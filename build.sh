#!/bin/bash

set -euo pipefail

cargo build --release -Zbuild-std=core -Zbuild-std-features=panic_immediate_abort
ELF=target/thumbv6m-none-eabi/release/flash-algo

llvm-objdump --disassemble $ELF > target/disassembly.s
llvm-objdump -x $ELF > target/dump.txt
llvm-nm $ELF -n > target/nm.txt

function bin {
    llvm-objcopy $ELF -O binary - | base64 -w0
}

function sym {
    echo $((0x$(llvm-nm $ELF | grep $1 | cut -d ' ' -f 1) + 1))
}

cat <<EOF
    instructions: $(bin)
    pc_init: $(sym _algo_init)
    pc_uninit: $(sym _algo_uninit)
    pc_program_page: $(sym _algo_program_page)
    pc_erase_sector: $(sym _algo_erase_sector)
    pc_erase_all: $(sym _algo_erase_all)
EOF