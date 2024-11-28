#![no_std]
#![no_main]

use flash_algorithm::*;

extern "C" {
    /// Enables the redundancy coprocessor (RCP)
    ///
    /// If the RCP is already initialized, `init_rcp` will skip initialization
    /// as initializing it twice will cause a fault.
    fn init_rcp();
}

// Implementation adapted from the PicoSDK's crt0.S.
core::arch::global_asm!(
    r#"
    .syntax unified
    .cpu cortex-m33
    .thumb
    .global init_rcp
    init_rcp:
        // Enable the RCP.  To save space, it is assumed that no other
        // coprocessors are enabled.
        ldr r0, =0xe0000000 + 0x0000ed88  // PPB_BASE + M33_CPACR_OFFSET
        movs r1, 0x0000c000 // CPACR_CP7_BITS
        str r1, [r0]

        // Check that to see if the RCP is already initialized.
        //
        // Since this check requires passing `r15` to `mrc` and the inline
        // assembler will not allow this, we hard code the instruction here:
        //     `mrc p7, #1, r15, c0, c0, #0`
        .byte 0x30
        .byte 0xee
        .byte 0x10
        .byte 0xf7

        // Skip initialization if already initialized.
        bmi 2f

        // Initialize the RCP.
        mcrr p7, #8, r0, r0, c0
        mcrr p7, #8, r0, r0, c1

        // Signal other core.
        sev

        2:
        bx lr
    "#,
);

unsafe fn lookup_func_rp2040(tag: u32) -> usize {
    type RomTableLookupFn = unsafe extern "C" fn(table: *const u16, code: u32) -> usize;

    /// This location in ROM holds a 16-bit truncated pointer for the ROM lookup function for
    /// RP2040 ROMs.
    const ROM_TABLE_LOOKUP_PTR: *const u16 = 0x0000_0018 as _;

    /// This location in ROM holds a 16-bit truncated pointer for the ROM function table
    /// (there's also a ROM data table which we don't need)
    const FUNC_TABLE: *const u16 = 0x0000_0014 as _;

    let lookup_func = ROM_TABLE_LOOKUP_PTR.read() as usize;
    let lookup_func: RomTableLookupFn = core::mem::transmute(lookup_func);
    let table = FUNC_TABLE.read() as usize;
    lookup_func(table as *const u16, tag)
}

unsafe fn lookup_func_235x(tag: u32) -> usize {
    type RomTableLookupFn = unsafe extern "C" fn(code: u32, mask: u32) -> usize;

    /// This location in ROM holds a 16-bit truncated pointer for the ROM lookup function for
    /// RP235x ROMs.
    const ROM_TABLE_LOOKUP_PTR: *const u16 = 0x0000_0016 as _;

    /// The flash-algo needs to run in secure mode so we need too look up
    /// functions in that context
    const RT_FLAG_FUNC_ARM_SEC: u32 = 0x0004;

    // The RCP (redundancy coprocessor) must be enabled in order to call ROM functions.
    init_rcp();

    let lookup_func = ROM_TABLE_LOOKUP_PTR.read() as usize;
    let lookup_func: RomTableLookupFn = core::mem::transmute(lookup_func);
    lookup_func(tag, RT_FLAG_FUNC_ARM_SEC)
}

fn find_func<T>(tag: [u8; 2]) -> Result<T, ErrorCode> {
    let tag = u16::from_le_bytes(tag) as u32;

    /// This location in ROM holds a 3 byte magic value that confirms the validity of the
    /// ROM as well as identifying different interfaces for the RP2040 and RP235x.
    const BOOTROM_MAGIC: *const [u8; 3] = 0x0000_0010 as _;

    /// Magic value for RP2040 ROMs.
    const RP2040_BOOTROM_MAGIC: &[u8; 3] = b"Mu\x01";

    /// Magic value for RP235X ROMs.
    const RP235X_BOOTROM_MAGIC: &[u8; 3] = b"Mu\x02";

    unsafe {
        let result = match &*BOOTROM_MAGIC {
            RP2040_BOOTROM_MAGIC => lookup_func_rp2040(tag),
            RP235X_BOOTROM_MAGIC => lookup_func_235x(tag),
            _ => return Err(ErrorCode::new(0x1000_0000 | tag).unwrap()),
        };
        if result == 0 {
            return Err(ErrorCode::new(0x2000_0000 | tag).unwrap());
        }
        Ok(core::mem::transmute_copy(&result))
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
    fn load() -> Result<Self, ErrorCode> {
        Ok(ROMFuncs {
            connect_internal_flash: find_func(*b"IF")?,
            flash_exit_xip: find_func(*b"EX")?,
            flash_range_erase: find_func(*b"RE")?,
            flash_range_program: find_func(*b"RP")?,
            flash_flush_cache: find_func(*b"FC")?,
            flash_enter_cmd_xip: find_func(*b"CX")?,
        })
    }
}

struct RP2Algo {
    funcs: ROMFuncs,
}

algorithm!(RP2Algo, {
    device_name: "Raspberry Pi RP2",
    device_type: DeviceType::ExtSpi,
    flash_address: 0x1000_0000,
    flash_size: 0x0100_0000,
    page_size: 0x100,
    empty_value: 0xFF,
    program_time_out: 500, // 500 ms
    erase_time_out: 5000, // 5 s
    sectors: [{
        size: 0x1000,
        address: 0x10000000,
    }]
});

const BLOCK_SIZE: u32 = 65536;
const SECTOR_SIZE: u32 = 4096;
const BLOCK_ERASE_CMD: u8 = 0xd8;

impl FlashAlgorithm for RP2Algo {
    fn new(_address: u32, _clock: u32, _function: Function) -> Result<Self, ErrorCode> {
        let funcs = ROMFuncs::load()?;
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

impl Drop for RP2Algo {
    fn drop(&mut self) {
        (self.funcs.flash_flush_cache)();
        (self.funcs.flash_enter_cmd_xip)();
    }
}
