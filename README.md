# RaspberryPi Pico RP2 flash algorithm

This is a flash algorithm for the RP2040 and RP2350 chips, used in the Raspberry Pi Pico board.
It implements the CMSIS-Pack ABI, so it's compatible with any tools that use it, including probe-rs.

## Dependencies

Run the following to install the requirements:

```bash
cargo install cargo-binutils
```

The `rust-toolchain` file will get you the targets and components you need.

## Building

Just run `build.sh`. It spits out the flash algo in the probe-rs YAML format:

```console
flash-algo$ ./build.sh 
    instructions: sLUUIACIGUoBRguI...wRwAgcEc=
    pc_init: 0x00000000
    pc_uninit: 0x0000007c
    pc_program_page: 0x00000088
    pc_erase_sector: 0x00000084
    pc_erase_all: 0x00000080
```

## Hacking

The `algo` module contains the FlashAlgo trait, and an `algo!` macro to generate
the glue functions for a given struct implementing it. This is generic for all chips, so feel free to reuse it!

`main.rs` has the actual implementation for RP2040 and RP2350.

## License

This thingy is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
