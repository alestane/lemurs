#![allow(dead_code)]

pub use execution::opcode;

use core::cell::UnsafeCell;

use crate::{Harness, ops::{Deref, DerefMut, Index, IndexMut}, Machine, raw, bits::*, num::Wrapping};

#[cfg(not(any(feature="open", debug_assertions)))]
pub(self) mod access;
#[cfg(any(feature="open", debug_assertions))]
pub mod access;
mod execution;

pub struct Socket(UnsafeCell<u8>);

impl Socket {
	pub fn new() -> Self {
		Self::default()
	}
}

impl Default for Socket {
	fn default() -> Self{
		Socket(UnsafeCell::default())
	}
}

impl Index<u16> for Socket {
	type Output = u8;
	fn index(&self, _index: u16) -> &Self::Output { let i = self.0.get(); unsafe {*i = Wrapping(0); &*i} }
}

impl IndexMut<u16> for Socket {
	fn index_mut(&mut self, _index: u16) -> &mut Self::Output { let i = self.0.get_mut(); *i = Wrapping(0); i }
}

impl Deref for Socket {
	type Target = [u8];
	fn deref(&self) -> &Self::Target {
		core::slice::from_ref(unsafe{self.0.get().as_ref().unwrap_unchecked()})
	}
}

impl Harness for Socket {
	fn read(&self, from: u16) -> u8 { let _ = from; Wrapping(0) }
	fn read_word(&self, from: u16) -> u16 { let _ = from; Wrapping(0) }
	fn input(&mut self, _port: raw::u8) -> u8 { Wrapping(0) }
	fn output(&mut self, _port: raw::u8, _value: u8) { }
}

#[cfg_attr(debug_assertions, disclose)]
pub struct State {
	pc: u16,
	sp: u16,
	register: [u8;7],
	c: bool, a: bool, p: bool, m: bool, z: bool,
	active: bool, interrupts: bool,
}

pub use access::Byte;

impl State {
	pub fn new() -> Self {
		Self {
			register: [Wrapping(0);7], 
			c: false, a: false, p: false, m: false, z: false, 
			active: true, interrupts: false, 
			pc: Wrapping(0), sp: Wrapping(0), 
		}
	}
}

#[cfg(debug_assertions)]
impl<H: Harness, C: DerefMut<Target = H>> Iterator for Machine<H, C> {
	type Item = Result<core::primitive::u8, crate::String>;	
	fn next(&mut self) -> Option<Self::Item> {
		let result = self.execute();
		match result {
			Ok(Some(cycles)) => Some(Ok(cycles.get())),
			Ok(None) => None,
			Err(e) => Some(Err(e)),
		}
	}
}

#[cfg(not(debug_assertions))]
impl<H: Harness, C: DerefMut<Target = H>> Iterator for Machine<H, C> {
	type Item = core::primitive::u8;
	fn next(&mut self) -> Option<Self::Item> {
		use core::num::NonZeroU8;
		self.execute().map(NonZeroU8::get)
	}
}

