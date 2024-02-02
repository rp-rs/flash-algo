#![macro_use]

use core::arch::asm;
use core::num::NonZeroU32;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        asm!("udf #0");
        core::hint::unreachable_unchecked();
    }
}

pub type ErrorCode = NonZeroU32;

pub trait FlashAlgo: Sized + 'static {
    /// Initialize the flash algorithm.
    fn new(address: u32, clock: u32, function: u32) -> Result<Self, ErrorCode>;

    /// Erase entire chip. May only be called after init() with FUNCTION_ERASE
    fn erase_all(&mut self) -> Result<(), ErrorCode>;

    /// Erase sector. May only be called after init() with FUNCTION_ERASE
    fn erase_sector(&mut self, addr: u32) -> Result<(), ErrorCode>;

    /// Erase a number of sectors. May only be called after init() with FUNCTION_ERASE
    fn erase_sectors(&mut self, addr: u32, qty: u32) -> Result<(), ErrorCode>;

    /// Program bytes. May only be called after init() with FUNCTION_PROGRAM
    fn program_page(&mut self, addr: u32, size: u32, data: *const u8) -> Result<(), ErrorCode>;
}

#[macro_export]
macro_rules! algo {
    ($type:ty) => {
        static mut _IS_INIT: bool = false;
        static mut _ALGO_INSTANCE: MaybeUninit<$type> = MaybeUninit::uninit();

        /// Initialise the Flash Algorithm
        ///
        /// # Safety
        ///
        /// Will disable execution from Flash. Ensure you are running from SRAM
        /// and do not call any flash-based based functions.
        #[no_mangle]
        #[link_section = ".entry"]
        pub unsafe extern "C" fn Init(addr: u32, clock: u32, function: u32) -> u32 {
            if _IS_INIT {
                UnInit();
            }
            match <$type as FlashAlgo>::new(addr, clock, function) {
                Ok(inst) => {
                    _ALGO_INSTANCE.as_mut_ptr().write(inst);
                    _IS_INIT = true;
                    0
                }
                Err(e) => e.get(),
            }
        }
        /// Uninitialise the Flash Algorithm
        #[no_mangle]
        #[link_section = ".entry"]
        pub extern "C" fn UnInit() -> u32 {
            unsafe {
                if !_IS_INIT {
                    return 1;
                }
                _ALGO_INSTANCE.as_mut_ptr().drop_in_place();
                _IS_INIT = false;
            }
            0
        }
        /// Erase the flash chip.
        ///
        /// # Safety
        ///
        /// Will erase the flash chip. Ensure you really want to erase the
        /// flash chip.
        #[no_mangle]
        #[link_section = ".entry"]
        pub unsafe extern "C" fn EraseChip() -> u32 {
            if !_IS_INIT {
                return 1;
            }
            let this = &mut *_ALGO_INSTANCE.as_mut_ptr();
            match <$type as FlashAlgo>::erase_all(this) {
                Ok(()) => 0,
                Err(e) => e.get(),
            }
        }
        /// Erase the a sector on the flash chip.
        ///
        /// # Safety
        ///
        /// Will erase the given sector. Pass a valid sector address.
        #[no_mangle]
        #[link_section = ".entry"]
        pub unsafe extern "C" fn EraseSector(addr: u32) -> u32 {
            if !_IS_INIT {
                return 1;
            }
            let this = &mut *_ALGO_INSTANCE.as_mut_ptr();
            match <$type as FlashAlgo>::erase_sector(this, addr) {
                Ok(()) => 0,
                Err(e) => e.get(),
            }
        }
        /// Erase a number of sectors on the flash chip.
        /// Will use block erase commands when possible to improve performance
        ///
        /// # Safety
        ///
        /// Will erase the given sectors. Pass a valid sector address and valid range of sectors.
        #[no_mangle]
        #[link_section = ".entry"]
        pub unsafe extern "C" fn EraseSectors(addr: u32, qty: u32) -> u32 {
            if !_IS_INIT {
                return 1;
            }
            let this = &mut *_ALGO_INSTANCE.as_mut_ptr();
            match <$type as FlashAlgo>::erase_sectors(this, addr, qty) {
                Ok(()) => 0,
                Err(e) => e.get(),
            }
        }
        /// Write to a page on the flash chip.
        ///
        /// # Safety
        ///
        /// Will program the given page. Pass a valid page address, and a
        /// valid pointer to at least `size` bytes of data.
        #[no_mangle]
        #[link_section = ".entry"]
        pub unsafe extern "C" fn ProgramPage(addr: u32, size: u32, data: *const u8) -> u32 {
            if !_IS_INIT {
                return 1;
            }
            let this = &mut *_ALGO_INSTANCE.as_mut_ptr();
            match <$type as FlashAlgo>::program_page(this, addr, size, data) {
                Ok(()) => 0,
                Err(e) => e.get(),
            }
        }
    };
}
