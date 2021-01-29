#![no_std]
#![no_main]
#![feature(asm)]

mod algo;
mod bootrom;

use core::mem::MaybeUninit;

use self::algo::*;
use self::bootrom::ROMFuncs;

struct RP2040Algo {
    funcs: ROMFuncs,
}

algo!(RP2040Algo);

const BLOCK_SIZE: u32 = 65536;
const SECTOR_SIZE: u32 = 4096;
const PAGE_SIZE: u32 = 256;
const BLOCK_ERASE_CMD: u8 = 0xd8;
const FLASH_BASE: u32 = 0x1000_0000;

impl FlashAlgo for RP2040Algo {
    fn new(_address: u32, _clock: u32, _function: u32) -> Result<Self, ErrorCode> {
        let funcs = ROMFuncs::load();

        (funcs.connect_internal_flash)();
        (funcs.flash_exit_xip)();
        Ok(Self { funcs })
    }

    fn erase_all(&mut self) -> Result<(), ErrorCode> {
        // todo
        Err(ErrorCode::new(0x70d0).unwrap())
    }

    fn erase_sector(&mut self, addr: u32) -> Result<(), ErrorCode> {
        (self.funcs.flash_range_erase)(addr - FLASH_BASE, SECTOR_SIZE, BLOCK_SIZE, BLOCK_ERASE_CMD);
        Ok(())
    }

    fn program_page(&mut self, addr: u32, size: u32, data: *const u8) -> Result<(), ErrorCode> {
        (self.funcs.flash_range_program)(addr - FLASH_BASE, data, size);
        Ok(())
    }
}

impl Drop for RP2040Algo {
    fn drop(&mut self) {
        (self.funcs.flash_flush_cache)();
        (self.funcs.flash_enter_cmd_xip)();
    }
}
