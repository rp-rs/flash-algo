#![no_std]
#![no_main]

mod algo;

use core::mem::MaybeUninit;

use self::algo::*;

fn find_func<T>(tag: [u8; 2]) -> Option<T> {
    let tag = u16::from_le_bytes(tag) as u32;
    type RomTableLookupFn = unsafe extern "C" fn(table: *const u16, code: u32) -> usize;
    /// This location in flash holds a 16-bit truncated pointer for the ROM lookup function
    const ROM_TABLE_LOOKUP_PTR: *const u16 = 0x0000_0018 as _;
    /// This location in flash holds a 16-bit truncated pointer for the ROM function table
    /// (there's also a ROM data table which we don't need)
    const FUNC_TABLE: *const u16 = 0x0000_0014 as _;
    unsafe {
        let lookup_func = ROM_TABLE_LOOKUP_PTR.read() as usize;
        let lookup_func: RomTableLookupFn = core::mem::transmute(lookup_func);
        let table = FUNC_TABLE.read() as usize;
        let result = lookup_func(table as *const u16, tag);
        if result == 0 {
            return None;
        }
        Some(core::mem::transmute_copy(&result))
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
    fn load() -> Option<Self> {
        Some(ROMFuncs {
            connect_internal_flash: find_func(*b"IF")?,
            flash_exit_xip: find_func(*b"EX")?,
            flash_range_erase: find_func(*b"RE")?,
            flash_range_program: find_func(*b"RP")?,
            flash_flush_cache: find_func(*b"FC")?,
            flash_enter_cmd_xip: find_func(*b"CX")?,
        })
    }
}

struct RP2040Algo {
    funcs: ROMFuncs,
}

algo!(RP2040Algo);

const BLOCK_SIZE: u32 = 65536;
const SECTOR_SIZE: u32 = 4096;
const BLOCK_ERASE_CMD: u8 = 0xd8;
const FLASH_BASE: u32 = 0x1000_0000;

impl FlashAlgo for RP2040Algo {
    fn new(_address: u32, _clock: u32, _function: u32) -> Result<Self, ErrorCode> {
        let Some(funcs) = ROMFuncs::load() else {
            return Err(ErrorCode::new(1).unwrap());
        };
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

    fn erase_range(&mut self, start_addr: u32, end_addr: u32) -> Result<(), ErrorCode> {
        let size = (end_addr - start_addr) + 1;
        (self.funcs.flash_range_erase)(start_addr - FLASH_BASE, size, BLOCK_SIZE, BLOCK_ERASE_CMD);
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
