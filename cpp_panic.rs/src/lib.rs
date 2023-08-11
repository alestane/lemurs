#![no_std]
#![feature(lang_items)]

#[cfg(not(test))]
mod disco {
	use core::panic::PanicInfo;

	extern "C-unwind" {
		fn bail() -> !;
	}

	#[panic_handler]
	pub fn throw(_panic: &PanicInfo<'_>) -> ! {
		unsafe { bail() }
	}

	#[lang="eh_personality"]
	pub extern "C" fn eh_personality() {}
}

#[cfg(not(test))]
pub use disco::*;