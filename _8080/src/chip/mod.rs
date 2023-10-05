use crate::prelude::*;

pub mod access;
mod execution;
pub use execution::opcode;

/// This struct stores the internal registers and flags of the 8080 CPU.
#[repr(C)]
#[cfg_attr(feature="open", disclose)]
pub struct State {
	pc: u16,
	sp: u16,
	register: [u8;7],
	c: bool, a: bool, p: bool, m: bool, z: bool,
	active: bool, interrupts: bool,
}

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
impl<H: Harness + ?Sized, C: BorrowMut<H>> Iterator for Machine<H, C> {
	type Item = Result<raw::u8, String>;
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
impl<H: Harness + ?Sized, C: BorrowMut<H>> Iterator for Machine<H, C> {
	type Item = core::primitive::u8;
	fn next(&mut self) -> Option<Self::Item> {
		use core::num::NonZeroU8;
		self.execute().map(NonZeroU8::get)
	}
}

