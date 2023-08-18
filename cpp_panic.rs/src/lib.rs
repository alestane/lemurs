#![no_std]
#![feature(lang_items)]
#![feature(slice_take)]

#[cfg(not(test))]
mod disco {
	use core::{panic::PanicInfo, fmt::Write};

	extern "C-unwind" {
		fn bail(desc: *const u8) -> !;
	}

	struct ErrorBuffer<'a>(&'a mut [u8]);

	impl core::ops::Deref for ErrorBuffer<'_> {
		type Target = [u8];
		fn deref(&self) -> &Self::Target {
			&self.0[..]
		}
	}

	impl core::fmt::Write for ErrorBuffer<'_> {
		fn write_str(&mut self, text: &str) -> core::fmt::Result {
			let size = text.len();
			if self.0.len() < size { return Err(core::fmt::Error); }
			self.0[..size].copy_from_slice(text.as_bytes());
			let _ = self.0.take_mut(..size);
			Ok(())
		}
	}

	#[panic_handler]
	pub fn throw(panic: &PanicInfo<'_>) -> ! {
		static mut SCRATCHPAD: [u8;512] = [0;512];
		let backup_text = "Error in Rust Library (couldn't format full description";
		unsafe { 
			let mut buffer = ErrorBuffer(&mut SCRATCHPAD[..]);
			let error_text = if write!(&mut buffer, "Error in Rust Library:\n{panic}").is_ok() {
				SCRATCHPAD.as_ptr()
			} else {
				backup_text.as_ptr()
			};
			bail(error_text)
		}
	}

	#[lang="eh_personality"]
	pub extern "C" fn eh_personality() {}
}

#[cfg(not(test))]
pub use disco::*;