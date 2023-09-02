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
use foundation::{*, result::Result};
pub use boxed::Box;

mod chip;

use crate::ops::{Deref, Index, IndexMut, Range, RangeFull, RangeFrom, RangeTo, RangeToInclusive};

pub trait Harness : IndexMut<u16, Output = u8> + Deref<Target=[u8]> {
	fn input(&mut self, port: u8) -> u8;
	fn output(&mut self, port: u8, value: u8);
    #[cfg(debug_assertions)]
    fn did_execute(&mut self, _client: &State) -> Result<bool, String> { Ok( true )}
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
    fn deref(&self) -> &Self::Target {
        &self.ram[..]
    }
}

impl Harness for SimpleBoard {
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

#[cfg(debug_assertions)]
pub mod support {
    use super::*;
    pub use chip::Socket;
    pub use chip::access::*;
}

pub mod op {
    pub use super::chip::opcode::{Op::{self, *}, Flag::*, Test::*};
}

pub use chip::State;
