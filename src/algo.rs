#![macro_use]


use core::num::NonZeroU32;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        asm!("udf #0");
        core::hint::unreachable_unchecked();
    }
}

pub const FUNCTION_ERASE: u32 = 1;
pub const FUNCTION_PROGRAM: u32 = 2;
pub const FUNCTION_VERIFY: u32 = 3;

pub type ErrorCode = NonZeroU32;

pub trait FlashAlgo: Sized + 'static {
    /// Initialize the flash algorithm.
    fn new(address: u32, clock: u32, function: u32) -> Result<Self, ErrorCode>;

    /// Erase entire chip. May only be called after init() with FUNCTION_ERASE
    fn erase_all(&mut self) -> Result<(), ErrorCode>;

    /// Erase sector. May only be called after init() with FUNCTION_ERASE
    fn erase_sector(&mut self, addr: u32) -> Result<(), ErrorCode>;

    /// Program bytes. May only be called after init() with FUNCTION_PROGRAM
    fn program_page(&mut self, addr: u32, size: usize, data: *const u8) -> Result<(), ErrorCode>;
}

#[macro_export]
macro_rules! algo {
    ($type:ty) => {
        static mut _IS_INIT: bool = false;
        static mut _ALGO_INSTANCE: MaybeUninit<$type> = MaybeUninit::uninit();

        #[no_mangle]
        #[link_section = ".entry"]
        pub unsafe extern "C" fn Init(addr: u32, clock: u32, function: u32) -> u32 {
            if _IS_INIT {
                UnInit();
            }
            _IS_INIT = true;
            match <$type as FlashAlgo>::new(addr, clock, function) {
                Ok(inst) => {
                    _ALGO_INSTANCE.as_mut_ptr().write(inst);
                    _IS_INIT = true;
                    0
                }
                Err(e) => e.get(),
            }
        }
        #[no_mangle]
        #[link_section = ".entry"]
        pub unsafe extern "C" fn UnInit() -> u32 {
            if !_IS_INIT {
                return 1;
            }
            _ALGO_INSTANCE.as_mut_ptr().drop_in_place();
            _IS_INIT = false;
            0
        }
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
        #[no_mangle]
        #[link_section = ".entry"]
        pub unsafe extern "C" fn ProgramPage(addr: u32, size: usize, data: *const u8) -> u32 {
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
