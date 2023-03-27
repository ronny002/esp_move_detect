# Rust program for Esp32 to detect movement via IR sensor
Esp32 program which uses a HW-416 passive infra red sensor (5V) to detect movement and sends the sensor state to Loxone server using udp. Also the Esp hosts a http server, so the user can change settings on the esp through a browser. Ota firmware flashing is implemented.

IR sensor: G17

## Set up QEMU to simulate Esp32 hardware
build prerequisites see https://wiki.qemu.org/Hosts/Linux
```
sudo apt-get install git libglib2.0-dev libfdt-dev libpixman-1-dev zlib1g-dev ninja-build libgcrypt20 libgcrypt20-dev libudev-dev
```
get Qemu
```
git clone https://github.com/espressif/qemu
```
configure see https://github.com/espressif/qemu/wiki
```
./configure --target-list=xtensa-softmmu \
    --enable-gcrypt \
    --enable-debug --enable-sanitizers \
    --disable-strip --disable-user \
    --disable-capstone --disable-vnc \
    --disable-sdl --disable-gtk
```
build
```
ninja -C build
```
## Set up toolchain and project
get toolchain see prerequisites at https://github.com/esp-rs/esp-idf-template#prerequisites and \
https://docs.espressif.com/projects/esp-idf/en/v4.4.1/esp32/get-started/linux-setup.html 
```
cargo install cargo-generate
cargo install ldproxy
cargo install espup
cargo install espflash
cargo install cargo-espflash
espup install
```
clone git repo
```
git clone https://github.com/ronny002/esp_move_detect
```
load env vars (once per terminal session) 
- Bash: 
```
source ~/export-esp.sh
```
- or if you use nushell
```
source export-esp.nu
```
## Run with Qemu
build app.bin see https://esp-rs.github.io/book/tooling/simulating/qemu.html \
use `--features qemu` to switch from wifi to eth (included in qemu.sh) \
Build and run
```
./qemu.sh
```
Build
```
cargo espflash save-image --features qemu --merge --release ESP32 app.bin 
```
run bin in QEMU
```
~/Documents/code/rust/esp_move_detect/qemu/build/qemu-system-xtensa -nographic -machine esp32 -nic user,model=open_eth,id=lo0,hostfwd=udp:127.0.0.1:7888-:80 -drive file=app.bin,if=mtd,format=raw
```
Networking \
see https://www.sbarjatiya.com/notes_wiki/index.php/Qemu_networking
- host (server) can be reached from guest (esp) with host ip
- to reach guest from host hostfw is needed
  hostfwd=tcp/upd:hostip:hostport-guestip:guestport
    - http server: 127.0.0.1:8080 forwarded to 10.0.2.15:80
- guest ip = 10.0.2.15
- host ip  = 192.168.1.?
## Run on Esp
```
sudo chmod 666 /dev/ttyUSB0
```
build, flash and monitor
```
cargo espflash --release --monitor /dev/ttyUSB0
```
only monitor
```
espflash serial-monitor /dev/ttyUSB0 
```
## ota
Do not forget
```
source ~/export-esp.sh or source export-esp.nu
```
build ota app
```
cargo espflash save-image ESP32 app_ota.bin --release
```
click on `ota` http command in browser \
use ota-downloader (`cargo run`) to download ota app to esp32 (set right path for app)\
hint to cargo espflash: qemu bin needs --merge, ota bin no --merge

## http server: html file
https://onecompiler.com/html/3z2tt824q

## NAPT Router
When esp is far away from wifi router, it's possible to use different esp closer to wifi as wifi extender. Udp packets comming from esp far out are forwarded over esp wifi extender to loxone server. Internet is also forwarded.

- to configure esp as wifi extender uncommend everything after `NAPT demo (router)` in `rust-bin/sdkconfig.defaults` 
- wifi extender hosts `esp32_presence_detector` wifi (referred as ap), set password in wifi_info_fill_out.rs 
- esp far out needs to change target wifi to `esp32_presence_detector`
- to access http server of esp far out connect to `esp32_presence_detector` wifi and use 192.168.71.2 in browser

## To Do
- monitor serial over wifi
- schematics
- movement detection sensitivity (connect pwm instead poti?)
- test and improve UX
- licences
- increase features of html website
- not working with high amps???
- qemu: hangs after esp_restart() so not possible to simulate ota flash
- low/high toggles with movement present when small follow-up time

## Solved Problems
Problem rust-analyser can't find clang: \
put this in vscode -> user settings (settings.json)
```
    "rust-analyzer.server.extraEnv": {
        "LIBCLANG_PATH": "/home/ronny/.espressif/tools/xtensa-esp32-elf-clang/esp-15.0.0-20221201-x86_64-unknown-linux-gnu/esp-clang/lib"
    },
```
see https://githubhelp.com/esp-rs/esp-idf-template/issues/49

## Resources
Esp Book https://esp-rs.github.io/book/ \
Nice example program https://github.com/ivmarkov/rust-esp32-std-demo \
IR sensor https://electropeak.com/learn/pir-motion-sensor-how-pir-work-how-use-with-arduino/
Browser Simulate https://wokwi.com/rust




