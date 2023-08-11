#![cfg_attr(not(test), no_std)]
#![feature(lang_items)]

#[cfg(not(test))]
use core::panic::PanicInfo;

#[cfg(not(test))]
extern "C-unwind" {
	fn bail() -> !;
}

#[cfg(not(test))]
#[panic_handler]
pub fn throw(_panic: &PanicInfo<'_>) -> ! {
	unsafe { bail() }
}

#[cfg(not(test))]
#[lang="eh_personality"]
pub extern "C" fn eh_personality() {}
