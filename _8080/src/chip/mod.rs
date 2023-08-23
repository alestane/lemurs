#![allow(dead_code)]

use crate::{vec, vec::Vec, num::NonZeroU8};

pub(self) mod access;
mod execution;

pub trait Debugger = Fn(&[u8], u16, u16, u8) -> execution::Failure;

pub struct State {
	ram: Vec<u8>,
	port_in: [u8;256],
	port_out: [u8;256],
	register: [u8;7],
	c: bool, a: bool, p: bool, m: bool, z: bool,
	active: bool, interrupts: bool,
	pc: u16,
	sp: u16,
	#[cfg(debug_assertions)]
	callbacks: Vec<crate::Box<dyn Debugger>>,
}

use core::convert::From;

pub use access::{Byte, Zone};

impl State {
	pub fn new() -> Self {
		Self { 
			ram: Vec::new(), 
			port_in: [0;256],
			port_out: [0;256], 
			register: [0;7], 
			c: false, a: false, p: false, m: false, z: false, 
			active: true, interrupts: false, 
			pc: 0, sp: 0, 
			#[cfg(debug_assertions)]
			callbacks: Vec::new(),
		}
	}

	pub fn with_ram(memory: u16) -> Self {
		let memory = if memory == 0 { 0x010000 } else { memory as usize }; 
		Self {
			ram: vec![0;memory], 
			..Self::new()
		}
	}
}

impl Default for State {
	fn default() -> Self {
		Self::with_ram(0x0100)
	}
}

impl From<&[u8]> for State {
	fn from(memory: &[u8]) -> Self {
		State{
			ram: Vec::from(if memory.len() > 0x010000 {&memory[..0x010000]} else {memory}), 
			..Self::new()
		}
	}
}

impl Iterator for State {
	type Item = u8;
	fn next(&mut self) -> Option<Self::Item> {
		self.execute().into_iter().map(NonZeroU8::get).next()
	}
}

#[cfg(debug_assertions)]
impl State {
	pub fn add_callback<T: Debugger + 'static>(&mut self, op: T) {
		self.callbacks.push(crate::Box::new(op));
	}
}
