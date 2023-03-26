#!/bin/sh
. $HOME/export-esp.sh
cargo espflash save-image --features qemu --merge ESP32 app.bin --release
cargo espflash save-image --features qemu ESP32 app_ota.bin --release
~/Documents/code/rust/esp_move_detect/qemu/build/qemu-system-xtensa -nographic -machine esp32 -nic user,model=open_eth,id=lo0,hostfwd=tcp:127.0.0.1:8080-10.0.2.15:80,hostfwd=tcp:127.0.0.1:5003-10.0.2.15:5003 -drive file=app.bin,if=mtd,format=raw
#hostfwd=tcp/upd:hostip:hostport-guestip:guestport