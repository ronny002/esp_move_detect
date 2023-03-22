#!/usr/bin/env nu
let-env LIBCLANG_PATH = "/home/ronny/.espressif/tools/xtensa-esp32-elf-clang/esp-15.0.0-20221201-x86_64-unknown-linux-gnu/esp-clang/lib"
let-env PATH = ($env.PATH | append /home/ronny/.espressif/tools/riscv32-esp-elf/esp-2021r2-patch5-8_4_0/riscv32-esp-elf/bin)
let-env PATH = ($env.PATH | append /home/ronny/.espressif/tools/riscv32-esp-elf/esp-2021r2-patch5-8_4_0/riscv32-esp-elf/bin)
let-env PATH = ($env.PATH | append /home/ronny/.espressif/tools/xtensa-esp32-elf/esp-2021r2-patch5-8_4_0/xtensa)
let-env PATH = ($env.PATH | append /home/ronny/.espressif/tools/xtensa-esp32s2-elf/esp-2021r2-patch5-8_4_0/xtensa-esp32s2-elf/bin)
let-env PATH = ($env.PATH | append /home/ronny/.espressif/tools/xtensa-esp32s3-elf/esp-2021r2-patch5-8_4_0/xtensa-esp32s3-elf/bin)

