#!/bin/bash

set -euo pipefail

# macOS base64 doesn't take -w argument and defaults to a single line.
if [[ $(uname) = "Darwin" ]]; then
    BASE64_FLAGS=""
else
    BASE64_FLAGS="-w0"
fi


targets=(
    "thumbv6m-none-eabi"
    "thumbv8m.main-none-eabi"
    "riscv32imac-unknown-none-elf"
)

function bin {
    local elf=$1
    rust-objcopy $elf --remove-section DeviceData -O binary - | base64 $BASE64_FLAGS
}

function sym {
    local elf=$1
    local name=$2
    local hex
    hex=$(rust-nm "$elf" | grep -w "$name" | cut -d ' ' -f 1 || true)
    # if function not in elf, print missing instead of 0
    if [[ -z "$hex" ]]; then
        printf 'missing\n'
        return 0
    fi
    local addr=$((16#$hex))
    # add 1 for arm to set thumb mode bit
    if ! rust-objdump -f "$elf" | grep -q 'littleriscv'; then
        addr=$((addr + 1))
    fi
    printf '0x%x\n' "$addr"
}

for target in "${targets[@]}"; do
    cargo build --release --target $target
    ELF=target/$target/release/flash-algo
    rust-objdump --disassemble $ELF > target/$target/disassembly.s
    rust-objdump -x $ELF > target/$target/dump.txt
    rust-nm $ELF -n > target/$target/nm.txt

    cat <<EOF
Algo for $target:
        instructions: $(bin $ELF)
        pc_init: $(sym $ELF Init)
        pc_uninit: $(sym $ELF UnInit)
        pc_program_page: $(sym $ELF ProgramPage)
        pc_erase_sector: $(sym $ELF EraseSector)
        pc_verify: $(sym $ELF Verify)
EOF
done
