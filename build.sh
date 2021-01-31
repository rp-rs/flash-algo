#!/bin/bash

set -euo pipefail

cargo build --release
ELF=target/thumbv6m-none-eabi/release/flash-algo

rust-objdump --disassemble $ELF > target/disassembly.s
rust-objdump -x $ELF > target/dump.txt
rust-nm $ELF -n > target/nm.txt

function bin {
    rust-objcopy $ELF -O binary - | base64 -w0
}

function sym {
    echo $((0x$(rust-nm $ELF | grep -w $1 | cut -d ' ' -f 1) + 1))
}

cat <<EOF
    instructions: $(bin)
    pc_init: $(sym Init)
    pc_uninit: $(sym UnInit)
    pc_program_page: $(sym ProgramPage)
    pc_erase_sector: $(sym EraseSector)
    pc_erase_all: $(sym EraseChip)
EOF
