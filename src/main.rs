#![no_std]
#![no_main]

use flash_algorithm::*;

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

algorithm!(RP2040Algo, {
    flash_address: 0x1000_0000,
    flash_size: 0x0100_0000,
    page_size: 0x100,
    empty_value: 0xFF,
    sectors: [{
        size: 0x1000,
        address: 0x10000000,
    }]
});

const BLOCK_SIZE: u32 = 65536;
const SECTOR_SIZE: u32 = 4096;
const BLOCK_ERASE_CMD: u8 = 0xd8;

impl FlashAlgorithm for RP2040Algo {
    fn new(_address: u32, _clock: u32, _function: Function) -> Result<Self, ErrorCode> {
        let Some(funcs) = ROMFuncs::load() else {
            return Err(ErrorCode::new(1).unwrap());
        };
        (funcs.connect_internal_flash)();
        (funcs.flash_exit_xip)();
        Ok(Self { funcs })
    }

    fn erase_sector(&mut self, addr: u32) -> Result<(), ErrorCode> {
        (self.funcs.flash_range_erase)(
            addr - FlashDevice.dev_addr,
            SECTOR_SIZE,
            BLOCK_SIZE,
            BLOCK_ERASE_CMD,
        );
        Ok(())
    }

    fn program_page(&mut self, addr: u32, data: &[u8]) -> Result<(), ErrorCode> {
        (self.funcs.flash_range_program)(
            addr - FlashDevice.dev_addr,
            data.as_ptr(),
            data.len() as u32,
        );
        Ok(())
    }
}

impl Drop for RP2040Algo {
    fn drop(&mut self) {
        (self.funcs.flash_flush_cache)();
        (self.funcs.flash_enter_cmd_xip)();
    }
}
