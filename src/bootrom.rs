use core::mem;

fn find_func(tag: [u8; 2]) -> u32 {
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
                return entry_addr as u32;
            }
        }
    }
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
            ROMFuncs {
                connect_internal_flash: mem::transmute(find_func(*b"IF")),
                flash_exit_xip: mem::transmute(find_func(*b"EX")),
                flash_range_erase: mem::transmute(find_func(*b"RE")),
                flash_range_program: mem::transmute(find_func(*b"RP")),
                flash_flush_cache: mem::transmute(find_func(*b"FC")),
                flash_enter_cmd_xip: mem::transmute(find_func(*b"CX")),
            }
        }
    }
}
