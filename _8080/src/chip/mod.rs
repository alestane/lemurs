#![allow(dead_code)]

use crate::{boxed::Box, vec, vec::Vec};

pub struct State {
	ram: Vec<u8>,
	port_in: [u8;256],
	port_out: [u8;256],
	register: [u8;7],
	c: bool, a: bool, p: bool, m: bool, z: bool,
	active: bool, interrupts: bool,
	memory: u16,
}

use core::{convert::TryFrom, ops::{Index, IndexMut, Deref}};

pub enum Byte {
	In(u8),
	Out(u8),
}

pub enum Zone {
	In, 
	Out,
	RAM,
}

impl State {
	pub fn new_with_ram(memory: u16) -> Box<Self> {
		Box::new(State{
			ram: vec![0;memory as usize], 
			port_in: [0;256], 
			port_out: [0;256], 
			register: [0;7], 
			c: false, a: false, p: false, m: false, z: false, 
			active: true, interrupts: false, 
			memory: memory
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
			memory: length
		})
	}
}

impl State {
	pub fn execute(&mut self) -> u8 {
		let mut cycles = 0u8;
		let mut ram_pos = 255usize;
		let mut port_idx = 0usize;
		loop {
			let (ram_val, in_value) = (self.ram[ram_pos], self.port_in[port_idx]);
			(self.port_out[255 - ram_pos], self.ram[ram_pos]) = (ram_val, in_value);
			let progress = 
				(if ram_val != 0 { ram_pos -= 1; true } else { false }) ||
				(if in_value != 0 { port_idx += 1; true } else { false });
			if progress > 0 {
				cycles += 1;
			} else {
				break cycles;
			}
		}
	}
}

impl Index<Byte> for State {
	type Output = u8;
	fn index(&self, i: Byte) -> &Self::Output {
		match i {
			Byte::In(port) => &self.port_in[port as usize],
			Byte::Out(port) => &self.port_out[port as usize],
		}
	}
}

impl Index<Zone> for State {
	type Output = [u8];
	fn index(&self, z: Zone) -> &Self::Output {
		match z {
			Zone::In => &self.port_in[..],
			Zone::Out => &self.port_out[..],
			Zone::RAM => &self.ram[..],
		}
	}
}

impl IndexMut<Zone> for State {
	fn index_mut(&mut self, z: Zone) -> &mut Self::Output {
		match z {
			Zone::In => &mut self.port_in[..],
			Zone::Out => &mut self.port_out[..0],
			Zone::RAM => &mut self.ram[..0],
		}		
	}
}

impl Deref for State {
	type Target = [u8];
	fn deref(&self) -> &Self::Target {
		&self.ram[..]
	}
}