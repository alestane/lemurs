#![no_std]
#![feature(split_array)]
#![feature(exclusive_range_pattern)]
#![feature(trait_alias)]
#![feature(iter_intersperse)]
#![feature(generic_arg_infer)]

#[macro_use]
extern crate disclose;

#[cfg(feature="std")]
mod foundation {
    extern crate std;
    pub use std::{boxed, vec, array, convert, num, string, result, ops, slice};
}
#[cfg(not(feature="std"))]
mod foundation {
    extern crate alloc;
    pub use alloc::{boxed, vec, string};
    pub use core::{array, convert, num, result, ops, slice};
}
pub use foundation::string::String;
use foundation::*;
pub use boxed::Box;

mod chip;

use crate::ops::{Deref, DerefMut, Index, IndexMut, Range, RangeFull, RangeFrom, RangeTo, RangeToInclusive};
pub use core::result::Result;

pub use chip::State;

#[cfg(debug_assertions)]
pub mod support {
    use super::*;
    pub use chip::Socket;
    pub use chip::access::*;
}

pub mod op {
    pub use super::chip::opcode::{Op::{self, *}, Flag::*, Test::*};
}

pub trait Harness {
    fn read(&self, from: u16) -> u8;
    fn read_word(&self, from: u16) -> u16 {
        u16::from_le_bytes([self.read(from), self.read(from + 1)])
    }
    fn write(&mut self, value: u8, to: u16) { let _ = (value, to); }
    fn write_word(&mut self, value: u16, to: u16) {
        for (index, byte) in value.to_le_bytes().into_iter().enumerate() {
            self.write(byte, to + index as u16)
        }
    }
	fn input(&mut self, port: u8) -> u8;
	fn output(&mut self, port: u8, value: u8);
    #[cfg(debug_assertions)]
    fn did_execute(&mut self, _client: &State) -> Result<Option<op::Op>, String> { Ok( None )}
}

pub struct SimpleBoard {
	ram: [u8; 65536],
	port_out: [u8; 256],
	port_in: [u8; 256]
}

impl Default for SimpleBoard {
    fn default() -> Self {
        Self {
            ram: [0; _],
            port_out: [0; _],
            port_in: [0; _],
        }
    }
}

impl Deref for SimpleBoard {
    type Target = [u8];
    fn deref(&self) -> &Self::Target { &self.ram[..] }
}

impl Harness for SimpleBoard {
    fn read(&self, from: u16) -> u8 { self[from] }
    fn read_word(&self, from: u16) -> u16 {
        u16::from_le_bytes([self.ram[from as usize], self.ram[from as usize + 1]])
    }
    fn write(&mut self, value: u8, to: u16) { self.ram[to as usize] = value; }
    fn write_word(&mut self, value: u16, to: u16) {
        [self.ram[to as usize], self.ram[to.wrapping_add(1) as usize]] = value.to_le_bytes();
    }
	fn input(&mut self, port: u8) -> u8 {
		self.port_in[port as usize]
	}
	fn output(&mut self, port: u8, value: u8) {
		self.port_out[port as usize] = value
	}
}

impl Index<u16> for SimpleBoard {
    type Output = u8;
    fn index(&self, index: u16) -> &Self::Output { &self.ram[index as usize] }
}

impl Index<Range<u16>> for SimpleBoard {
    type Output = [u8];
    fn index(&self, index: Range<u16>) -> &Self::Output { &self.ram[index.start as usize..index.end as usize] }
}

impl Index<RangeFrom<u16>> for SimpleBoard {
    type Output = [u8];
    fn index(&self, index: RangeFrom<u16>) -> &Self::Output { &self.ram[index.start as usize..] }
}

impl Index<RangeTo<u16>> for SimpleBoard {
    type Output = [u8];
    fn index(&self, index: RangeTo<u16>) -> &Self::Output { &self.ram[..index.end as usize] }
}

impl Index<RangeToInclusive<u16>> for SimpleBoard {
    type Output = [u8];
    fn index(&self, index: RangeToInclusive<u16>) -> &Self::Output { &self.ram[..=index.end as usize] }
}

impl Index<RangeFull> for SimpleBoard {
    type Output = [u8];
    fn index(&self, _index: RangeFull) -> &Self::Output { &self.ram[..] }
}

impl IndexMut<u16> for SimpleBoard {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output { &mut self.ram[index as usize] }
}

impl IndexMut<Range<u16>> for SimpleBoard {
    fn index_mut(&mut self, index: Range<u16>) -> &mut Self::Output { &mut self.ram[index.start as usize..index.end as usize] }
}

impl IndexMut<RangeFrom<u16>> for SimpleBoard {
    fn index_mut(&mut self, index: RangeFrom<u16>) -> &mut Self::Output { &mut self.ram[index.start as usize..] }
}

impl IndexMut<RangeTo<u16>> for SimpleBoard {
    fn index_mut(&mut self, index: RangeTo<u16>) -> &mut Self::Output { &mut self.ram[..index.end as usize] }
}

impl IndexMut<RangeToInclusive<u16>> for SimpleBoard {
    fn index_mut(&mut self, index: RangeToInclusive<u16>) -> &mut Self::Output { &mut self.ram[..=index.end as usize] }
}

impl IndexMut<RangeFull> for SimpleBoard {
    fn index_mut(&mut self, _index: RangeFull) -> &mut Self::Output { &mut self.ram[..] }
}

pub struct Machine<H: Harness, C: DerefMut<Target = H>> {
    board: C,
    chip: State,
}

impl<H: Harness, C: DerefMut<Target = H>> Machine<H, C> {
    pub fn new(what: C) -> Self {
        Self { board: what, chip: State::new() }
    }
}

impl<H: Harness, C: DerefMut<Target = H>> Deref for Machine<H, C> {
    type Target = State;
    fn deref(&self) -> &Self::Target { &self.chip }
}

impl<H: Harness, C: DerefMut<Target = H>> DerefMut for Machine<H, C> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.chip }
}
