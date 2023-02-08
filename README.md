# Esp32 program for movement detecting and data sending
Esp32 program which uses a HW-416 passive infra red sensor (5V) to detect movement and sends the sensor state via udp to Loxone server.

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
## esp-idf-template
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
new project 
```
cargo generate https://github.com/esp-rs/esp-idf-template cargo
```
set toolchain to esp for the current project. not needed because of rust-toolchain.toml
```
rustup override set esp
```
load env vars (once per terminal session)
```
. $HOME/export-esp.sh
```
## Run with Qemu
build app.bin see https://esp-rs.github.io/book/tooling/simulating/qemu.html \
use `--features qemu` to switch from wifi to eth \
Build and run
```
./qemu.sh
```
Build
```
cargo espflash save-image --features qemu --merge ESP32 app.bin --release
```
run bin in QEMU
```
~/Documents/code/rust/esp_move_detect/qemu/build/qemu-system-xtensa -nographic -machine esp32 -nic user,model=open_eth,id=lo0,hostfwd=udp:127.0.0.1:7888-:80 -drive file=app.bin,if=mtd,format=raw
```
Error: can not set up eth connecting
```
E(21285) esp_eth: esp_eth_stop(288): driver not started yet
```
## Run on Esp
```
sudo chmod 666 /dev/ttyUSB0
```
build and flash and monitor
```
. $HOME/export-esp.sh
cargo espflash --release --monitor /dev/ttyUSB0
```
only monitor
```
espflash serial-monitor /dev/ttyUSB0 
```
## To Do
- send commands over tcp to esp (example: p for pause)
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




