#Esp32 movement detector for Loxone smart home

Problem rust-analyser can't find clang:
put this in vscode -> user settings (settings.json)
```
    "rust-analyzer.server.extraEnv": {
        "LIBCLANG_PATH": "/home/ronny/.espressif/tools/xtensa-esp32-elf-clang/esp-15.0.0-20221201-x86_64-unknown-linux-gnu/esp-clang/lib"
    },
```
see https://githubhelp.com/esp-rs/esp-idf-template/issues/49

Esp Book https://esp-rs.github.io/book/
##QEMU
build prerequisites see https://wiki.qemu.org/Hosts/Linux
```
sudo apt-get install git libglib2.0-dev libfdt-dev libpixman-1-dev zlib1g-dev ninja-build libgcrypt20 libgcrypt20-dev libuvdev-dev
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
##esp-idf-template
build prerequisites see https://github.com/esp-rs/esp-idf-template and 
https://docs.espressif.com/projects/esp-idf/en/v4.4.1/esp32/get-started/linux-setup.html \
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
build app.bin see https://esp-rs.github.io/book/tooling/simulating/qemu.html
```
cargo espflash save-image --features qemu --merge ESP32 app.bin --release
```
run bin in QEMU
```
~/Documents/code/rust/esp_move_detect/qemu/build/qemu-system-xtensa -nographic -machine esp32 -nic user,model=open_eth,id=lo0,hostfwd=udp:127.0.0.1:7888-:80 -drive file=app.bin,if=mtd,format=raw
```
DONE!






