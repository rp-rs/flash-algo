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

struct RP2040Algo {
    funcs: ROMFuncs,
}

algo!(RP2040Algo);

const BLOCK_SIZE: u32 = 65536;
const SECTOR_SIZE: u32 = 4096;
const PAGE_SIZE: u32 = 256;
const BLOCK_ERASE_CMD: u8 = 0xd8;
const FLASH_BASE: u32 = 0x1000_0000;

#[repr(C)]
pub struct FlashDevice {
    pub version: u16,
    pub device_name: [u8; 128],
    pub device_type: u16,
    pub device_address: u32,
    pub size_device: u32,
    pub size_page: u32,
    pub reserved: u32,
    pub val_empty: u8,
    pub timeout_program: u32,
    pub timeout_erase: u32,
    pub sectors: [u32; 4],
}

#[cfg(feature = "device_description")]
#[no_mangle]
#[link_section = ".PrgData"]
pub static dummy: u32 = 0;

#[cfg(feature = "device_description")]
#[no_mangle]
#[link_section = ".DevDscr"]
pub static FlashDevice: FlashDevice = FlashDevice {
    version: 1, // Version 1.01
    device_name: [
        0x52, 0x61, 0x73, 0x70, 0x65, 0x72, 0x72, 0x79, 0x20, 0x50, 0x69, 0x20, 0x52, 0x50, 0x32,
        0x30, 0x34, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ], // "Rasperry Pi RP2040"
    device_type: 5, // External SPI
    device_address: FLASH_BASE, // Default device start address
    size_device: 16 * 1024 * 1024, // Total Size of device (16 MiB)
    size_page: PAGE_SIZE, // Programming page size
    reserved: 0, // Must be zero
    val_empty: 0xFF, // Content of erase memory
    timeout_program: 500, // 500 ms
    timeout_erase: 5000, // 5 s
    sectors: [SECTOR_SIZE, FLASH_BASE, 0xFFFFFFFF, 0xFFFFFFFF],
};

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
