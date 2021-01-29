use core::mem;

// A bootrom function table code.
pub type RomFnTableCode = [u8; 2];

const ROM_TABLE_LOOKUP_PTR: *const u16 = 0x18 as _;
const OPS_TABLE_PTR: *const u16 = 0x14 as _;

/// This function searches for (table)
type RomTableLookupFn<T> = unsafe extern "C" fn(*const u16, u32) -> T;

/// Given a rom_address pointer, convert a 16 bit pointer stored
/// at the given rom address into a 32 bit pointer
#[inline]
unsafe fn get_ptr_from_rom(rom_address: *const u16) -> *const u16 {
    *rom_address as *const u16
}

unsafe fn find_func<T>(
    rom_table_lookup: *const u16,
    func_table: *const u16,
    tag: RomFnTableCode,
) -> T {
    let rom_table_lookup: RomTableLookupFn<T> = mem::transmute(rom_table_lookup);
    rom_table_lookup(func_table, u16::from_le_bytes(tag) as u32)
}

pub struct ROMFuncs {
    pub connect_internal_flash: extern "C" fn(),
    pub flash_exit_xip: extern "C" fn(),
    pub flash_range_erase: extern "C" fn(addr: u32, count: u32, block_size: u32, block_cmd: u8),
    pub flash_range_program: extern "C" fn(addr: u32, data: *const u8, count: u32),
    pub flash_flush_cache: extern "C" fn(),
    pub flash_enter_cmd_xip: extern "C" fn(),
}

impl ROMFuncs {
    pub fn load() -> Self {
        unsafe {
            let rom_table_lookup = get_ptr_from_rom(ROM_TABLE_LOOKUP_PTR);
            let func_table = get_ptr_from_rom(OPS_TABLE_PTR);
            ROMFuncs {
                connect_internal_flash: find_func(rom_table_lookup, func_table, *b"IF"),
                flash_exit_xip: find_func(rom_table_lookup, func_table, *b"EX"),
                flash_range_erase: find_func(rom_table_lookup, func_table, *b"RE"),
                flash_range_program: find_func(rom_table_lookup, func_table, *b"RP"),
                flash_flush_cache: find_func(rom_table_lookup, func_table, *b"FC"),
                flash_enter_cmd_xip: find_func(rom_table_lookup, func_table, *b"CX"),
            }
        }
    }
}
