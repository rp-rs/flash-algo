#!/bin/bash

set -euo pipefail

cd "$(dirname "$0")"

cargo build --release -p flash-algo --target thumbv6m-none-eabi
cargo build --release -p flash-algo --target thumbv8m.main-none-eabi
cargo build --release -p flash-algo --target riscv32imac-unknown-none-elf

target-gen elf -n algo -u target/thumbv6m-none-eabi/release/flash-algo RP2040.yaml
target-gen elf -n algo -u target/thumbv8m.main-none-eabi/release/flash-algo RP235x.yaml
target-gen elf -n algo_riscv -u target/riscv32imac-unknown-none-elf/release/flash-algo RP235x.yaml
