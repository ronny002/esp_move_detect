#!/bin/sh
. $HOME/export-esp.sh
cargo espflash save-image --features qemu --merge ESP32 app.bin --release

~/Documents/code/rust/esp_move_detect/qemu/build/qemu-system-xtensa -nographic -machine esp32 -nic user,model=open_eth,id=lo0,hostfwd=udp:127.0.0.1:1236-:1235 -drive file=app.bin,if=mtd,format=raw
#hostfwd=hostip:hostport-guestip:guestport