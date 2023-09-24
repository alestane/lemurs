pub use execution::opcode;

use core::{borrow::BorrowMut, num::Wrapping, ops::DerefMut};
use crate::{Harness, Machine, bits::*};

#[cfg(not(any(doc, feature="open")))]
pub(self) mod access;
#[cfg(any(doc, feature="open"))]
pub mod access;
mod execution;

/// This struct stores the internal registers and flags of the 8080 CPU.
#[repr(C)]
#[disclose(crate)]
pub struct State {
	pc: u16,
	sp: u16,
	register: [u8;7],
	c: bool, a: bool, p: bool, m: bool, z: bool,
	active: bool, interrupts: bool,
}

pub use access::Byte;

impl State {
	/// Creates a fresh state with the processor in an active state and all registers reset.
	pub fn new() -> Self {
		Self {
			register: [Wrapping(0);7],
			c: false, a: false, p: false, m: false, z: false,
			active: true, interrupts: false,
			pc: Wrapping(0), sp: Wrapping(0),
		}
	}
}

#[cfg(feature="open")]
impl<H: Harness + ?Sized, B: BorrowMut<H>> Iterator for Machine<H, B, B> {
	type Item = Result<core::primitive::u8, crate::string::String>;
	fn next(&mut self) -> Option<Self::Item> {
		let result = self.execute();
		match result {
			Ok(Some(cycles)) => Some(Ok(cycles.get())),
			Ok(None) => None,
			Err(e) => Some(Err(e)),
		}
	}
}

#[cfg(not(feature="open"))]
impl<H: Harness + ?Sized, B: BorrowMut<H>> Iterator for Machine<H, B, B> {
	type Item = core::primitive::u8;
	fn next(&mut self) -> Option<Self::Item> {
		use core::num::NonZeroU8;
		self.execute().map(NonZeroU8::get)
	}
}

