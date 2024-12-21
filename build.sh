#!/bin/bash

set -euo pipefail

# macOS base64 doesn't take -w argument and defaults to a single line.
if [[ $(uname) = "Darwin" ]]; then
    BASE64_FLAGS=""
else
    BASE64_FLAGS="-w0"
fi

cargo build --release
ELF=target/thumbv6m-none-eabi/release/flash-algo

rust-objdump --disassemble $ELF > target/disassembly.s
rust-objdump -x $ELF > target/dump.txt
rust-nm $ELF -n > target/nm.txt
# printf "Algo binary size: `rust-objcopy $ELF --remove-section DeviceData -O binary - | wc -c` bytes\n"

function bin {
    rust-objcopy $ELF --remove-section DeviceData -O binary - | base64 $BASE64_FLAGS
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
EOF
