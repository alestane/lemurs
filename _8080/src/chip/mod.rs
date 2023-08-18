#![allow(dead_code)]

use crate::{boxed::Box, vec, vec::Vec};

pub(self) mod access;
mod execution;

pub struct State {
	ram: Vec<u8>,
	port_in: [u8;256],
	port_out: [u8;256],
	register: [u8;7],
	c: bool, a: bool, p: bool, m: bool, z: bool,
	active: bool, interrupts: bool,
	pc: u16,
	sp: u16,
	memory: u16,
	#[cfg(debug_assertions)]
	callbacks: Vec<unsafe extern "C" fn(&'static [u8;1], u16, u16, u8) -> bool>,
}

use core::{convert::TryFrom, ops::Deref};

pub use access::{Byte, Zone};

impl State {
	pub fn new_with_ram(memory: u16) -> Box<Self> {
		Box::new(State{
			ram: vec![0;memory as usize], 
			port_in: [0;256], 
			port_out: [0;256], 
			register: [0;7], 
			c: false, p: false, a: false, z: false, m: false, 
			active: true, interrupts: false, 
			pc: 0, sp: 0,
			memory: memory,
			#[cfg(debug_assertions)] callbacks: Vec::new(),
		})
	}
	pub fn new_with_rom(memory: &[u8]) -> Box<Self> {
		let length = u16::try_from(memory.len()).unwrap_or(0);
		Box::new(State{
			ram: Vec::from(memory), 
			port_in: [0;256], 
			port_out: [0;256], 
			register: [0;7], 
			c: false, a: false, p: false, m: false, z: false, 
			active: true, interrupts: false, 
			pc: 0, sp: 0,
			memory: length,
			#[cfg(debug_assertions)] callbacks: Vec::new(),
		})
	}
}

#[cfg(debug_assertions)]
impl State {
	pub fn add_callback(&mut self, op: extern "C" fn (&'static [u8;1], u16, u16, u8) -> bool) {
		self.callbacks.push(op);
	}
}

impl State {
	pub fn execute(&mut self) -> u8 {
		0
	}
}

impl Deref for State {
	type Target = [u8];
	fn deref(&self) -> &Self::Target {
		&self.ram[..]
	}
}
