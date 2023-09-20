#![no_std]
#![feature(split_array)]
#![feature(exclusive_range_pattern)]
#![feature(trait_alias)]
#![feature(iter_intersperse)]
#![feature(generic_arg_infer)]
#![feature(error_in_core)]

#[macro_use]
extern crate disclose;

#[cfg(feature="std")]
mod foundation {
    extern crate std;
    pub use std::{boxed, vec, array, convert, num, string, result, ops, slice, any};
}
#[cfg(not(feature="std"))]
mod foundation {
    extern crate alloc;
    pub use alloc::{boxed, vec, string};
    pub use core::{array, convert, num, result, ops, slice, any};
}
pub use foundation::{string::String, num::Wrapping};
use foundation::*;
pub use boxed::Box;

mod chip;

#[cfg(feature="_cpp")]
mod cpp;

#[allow(non_camel_case_types)]
mod raw {
    pub type u8 = core::primitive::u8;
    pub type u16 = core::primitive::u16;
}

#[allow(non_camel_case_types)]
mod bits {
    use super::*;
    pub type u8 = crate::num::Wrapping<raw::u8>;
    pub type u16 = crate::num::Wrapping<raw::u16>;
}

use crate::ops::{Deref, DerefMut, Index, IndexMut, Range, RangeFull, RangeFrom, RangeTo, RangeToInclusive};
pub use core::result::Result;

#[cfg(debug_assertions)]
pub use chip::State;

#[cfg(feature="open")]
pub mod support {
    use super::*;
    pub use chip::Socket;
    pub use chip::access::*;
}
#[cfg(feature="open")]
pub mod op {
    pub use crate::chip::opcode::{Op::{self, *}, Flag::*, Test::*};
}
#[cfg(debug_assertions)]
pub use crate::chip::opcode::Op;

pub trait Harness {
    fn read(&self, from: bits::u16) -> bits::u8;
    fn read_word(&self, from: bits::u16) -> bits::u16 {
        Wrapping(u16::from_le_bytes([self.read(from).0, self.read(from + Wrapping(1)).0]))
    }
    fn write(&mut self, value: bits::u8, to: bits::u16) { let _ = (value, to); }
    fn write_word(&mut self, value: bits::u16, to: bits::u16) {
        for (index, byte) in value.0.to_le_bytes().into_iter().enumerate() {
            self.write(num::Wrapping(byte), to + num::Wrapping(index as u16))
        }
    }
	fn input(&mut self, port: u8) -> bits::u8;
	fn output(&mut self, port: u8, value: bits::u8);
    #[cfg(debug_assertions)]
    fn did_execute(&mut self, client: &State, did: Op) -> Result<Option<Op>, String> { let _ = (client, did); Ok( None ) }
    fn as_any(&self) -> Option<&dyn any::Any> { None }
}

#[repr(C)]
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
    fn read(&self, from: bits::u16) -> bits::u8 { num::Wrapping(self[from.0]) }
    fn read_word(&self, from: bits::u16) -> bits::u16 {
        Wrapping(u16::from_le_bytes([self.ram[from.0 as usize], self.ram[from.0 as usize + 1]]))
    }
    fn write(&mut self, value: bits::u8, to: bits::u16) { self.ram[to.0 as usize] = value.0; }
    fn write_word(&mut self, value: bits::u16, to: bits::u16) {
        [self.ram[to.0 as usize], self.ram[to.0.wrapping_add(1) as usize]] = value.0.to_le_bytes();
    }
	fn input(&mut self, port: u8) -> bits::u8 {
		Wrapping(self.port_in[port as usize])
	}
	fn output(&mut self, port: u8, value: bits::u8) {
		self.port_out[port as usize] = value.0
	}
    fn as_any(&self) -> Option<&dyn any::Any> {
        Some(self)
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

pub struct Machine<H: Harness + ?Sized, C: DerefMut<Target = H>> {
    board: C,
    chip: chip::State,
}

impl<H: Harness + ?Sized, C: DerefMut<Target = H>> Machine<H, C> {
    pub fn new(what: C) -> Self {
        Self { board: what, chip: chip::State::new() }
    }
}

impl<H: Harness + ?Sized, C: DerefMut<Target = H>> Deref for Machine<H, C> {
    type Target = H;
    fn deref(&self) -> &Self::Target { self.board.deref() }
}

impl<H: Harness + ?Sized, C: DerefMut<Target = H>> DerefMut for Machine<H, C> {
    fn deref_mut(&mut self) -> &mut Self::Target { self.board.deref_mut() }
}

#[cfg(debug_assertions)]
impl<H: Harness + ?Sized, C: DerefMut<Target = H>> AsRef<State> for Machine<H, C> {
    fn as_ref(&self) -> &State { &self.chip }
}

#[cfg(debug_assertions)]
impl<H: Harness + ?Sized, C: DerefMut<Target = H>> AsMut<State> for Machine<H, C> {
    fn as_mut(&mut self) -> &mut State { &mut self.chip }
}