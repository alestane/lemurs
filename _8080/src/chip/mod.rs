#![allow(dead_code)]

use crate::{*, boxed::Box, vec, vec::Vec};

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
	VRam,
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
	pub fn vram(&self) -> &[u8;VRAM_SIZE] {
		self.ram.rsplit_array_ref().1
	}
}

impl State {
	pub fn execute(&mut self) -> u8 {
		for (i, v) in self.ram.iter().take(self.port_out.len()).enumerate() {
			self.port_out[i] = *v;
		}
		1
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
			Zone::VRam => &self.ram[VRAM_ADDRESS..],
		}
	}
}

impl IndexMut<Zone> for State {
	fn index_mut(&mut self, z: Zone) -> &mut Self::Output {
		match z {
			Zone::In => &mut self.port_in[..],
			Zone::Out => &mut self.port_out[..0],
			Zone::RAM => &mut self.ram[..0],
			Zone::VRam => &mut self.ram[VRAM_ADDRESS..VRAM_ADDRESS],
		}		
	}
}

impl Deref for State {
	type Target = [u8];
	fn deref(&self) -> &Self::Target {
		&self.ram[..]
	}
}