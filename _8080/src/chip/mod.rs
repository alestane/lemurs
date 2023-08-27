#![allow(dead_code)]

pub use execution::opcode;

use core::cell::UnsafeCell;

use crate::{String, Harness, ops::{Deref, Index,IndexMut}};

#[cfg(not(debug_assertions))]
pub(self) mod access;
#[cfg(debug_assertions)]
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
	fn index(&self, _index: u16) -> &Self::Output { let i = self.0.get(); unsafe {*i = 0; &*i} }
}

impl IndexMut<u16> for Socket {
	fn index_mut(&mut self, _index: u16) -> &mut Self::Output { let i = self.0.get_mut(); *i = 0; i }
}

impl Deref for Socket {
	type Target = [u8];
	fn deref(&self) -> &Self::Target {
		core::slice::from_ref(unsafe{self.0.get().as_ref().unwrap_unchecked()})
	}
}

impl Harness for Socket {
	fn input(&mut self, _port: u8) -> u8 { 0 }
	fn output(&mut self, _port: u8, _value: u8) { }
}

#[cfg_attr(debug_assertions, disclose)]
pub struct State<'a> {
	board: &'a mut dyn Harness,
	register: [u8;7],
	c: bool, a: bool, p: bool, m: bool, z: bool,
	active: bool, interrupts: bool,
	pc: u16,
	sp: u16,
}

pub use access::Byte;

impl<'a> State<'a> {
	pub fn with(board: &'a mut dyn Harness) -> Self {
		Self {
			board: board, 
			register: [0;7], 
			c: false, a: false, p: false, m: false, z: false, 
			active: true, interrupts: false, 
			pc: 0, sp: 0, 
		}
	}

	pub fn embed(&mut self, board: &'a mut dyn Harness) {
		self.board = board;
	}
}

#[cfg(debug_assertions)]
impl Iterator for State<'_> {
	type Item = Result<u8, Result<String, String>>;	
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
impl Iterator for State<'_> {
	type Item = u8;
	fn next(&mut self) -> Option<Self::Item> {
		self.execute().map(NonZeroU8::get)
	}
}

