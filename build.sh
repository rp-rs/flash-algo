#!/bin/bash

set -euo pipefail

cargo build --release
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
    pc_init: $(sym Init)
    pc_uninit: $(sym UnInit)
    pc_program_page: $(sym ProgramPage)
    pc_erase_sector: $(sym EraseSector)
    pc_erase_all: $(sym EraseChip)
EOF
