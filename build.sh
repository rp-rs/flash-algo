#!/bin/bash

set -euo pipefail

# macOS base64 doesn't take -w argument and defaults to a single line.
if [[ $(uname) = "Darwin" ]]; then
    BASE64_FLAGS=""
else
    BASE64_FLAGS="-w0"
fi


target="thumbv6m-none-eabi"

function bin {
    rust-objcopy $1 --remove-section DeviceData -O binary - | base64 $BASE64_FLAGS
}

function sym {
    echo $((0x$(rust-nm $1 | grep -w $2 | cut -d ' ' -f 1) + 1))
}

cargo build --release --target $target
ELF=target/$target/release/flash-algo
rust-objdump --disassemble $ELF > target/$target/disassembly.s
rust-objdump -x $ELF > target/$target/dump.txt
rust-nm $ELF -n > target/$target/nm.txt

cat <<EOF
    instructions: $(bin $ELF)
    pc_init: $(sym $ELF Init)
    pc_uninit: $(sym $ELF UnInit)
    pc_program_page: $(sym $ELF ProgramPage)
    pc_erase_sector: $(sym $ELF EraseSector)
    pc_verify: $(sym $ELF Verify)
EOF
