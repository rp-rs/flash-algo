#![no_std]
#![no_main]
#![feature(asm)]

mod algo;

use core::mem;
use core::mem::MaybeUninit;

use self::algo::*;

fn find_func<T>(tag: [u8; 2]) -> T {
    let tag = u16::from_le_bytes(tag);

    unsafe {
        let mut entry = *(0x00000014 as *const u16) as *const u16;
        loop {
            let entry_tag = entry.read();
            if entry_tag == 0 {
                panic!("Func not found");
            }
            entry = entry.add(1);
            let entry_addr = entry.read();
            entry = entry.add(1);
            if entry_tag == tag {
                return mem::transmute_copy(&(entry_addr as u32));
            }
        }
    }
}

struct ROMFuncs {
    connect_internal_flash: extern "C" fn(),
    flash_exit_xip: extern "C" fn(),
    flash_range_erase: extern "C" fn(addr: u32, count: u32, block_size: u32, block_cmd: u8),
    flash_range_program: extern "C" fn(addr: u32, data: *const u8, count: u32),
    flash_flush_cache: extern "C" fn(),
    flash_enter_cmd_xip: extern "C" fn(),
}

impl ROMFuncs {
    fn load() -> Self {
        unsafe {
            ROMFuncs {
                connect_internal_flash: find_func(*b"IF"),
                flash_exit_xip: find_func(*b"EX"),
                flash_range_erase: find_func(*b"RE"),
                flash_range_program: find_func(*b"RP"),
                flash_flush_cache: find_func(*b"FC"),
                flash_enter_cmd_xip: find_func(*b"CX"),
            }
        }
    }
}

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
        (self.funcs.flash_range_erase)(addr - FLASH_BASE, 4096, BLOCK_SIZE, BLOCK_ERASE_CMD);
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
