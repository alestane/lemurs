//! # Intel CPU Emulation
//!
//! This emulates early Intel 8-bit microprocessors (currently, only the 8080 is supported). 
//! It packages a fixed chip core with a "board" that can be user-defined (A very basic board 
//! is included and published in the crate, but in most cases, you will want to supply your own, 
//! for instance to emulate the non-CPU features of a historical arcade game).
//! 
//! Typically, you will implement the `lemurs-8080::Harness` trait on a type of your choice and 
//! then create a `Machine` instance that uses a value of that type. You can use any type that 
//! dereferences to a `mut Harness` of some kind, so you can give a machine sole or shared 
//! ownership of its Harness or just attach it to a reference.
//! 
//! You can then call the `.execute()` method on your `Machine` to execute one instruction or use
//! your `Machine` as an iterator. By keeping access to the contents of your Harness, you can 
//! examine the contents produced by the code and use them with another resource, such as printing 
//! text to a console or copying a pixel raster to a window.
//! 
//! By default, the package is built with the `"std"` feature under the expectation that you will 
//! use it in other Rust projects. You can also use this library in C++ projects, by using 
//! `--no-default-features` to deactivate `"std"`, and add the `"cpp"` (or `"_cpp"`) feature. (`"cpp"` 
//! includes a C++-based global allocator and panic handler from the `cruppers` crate; `"_cpp"` just 
//! turns on the C++ bridge code and requires you to supply your own memory and panic management.)
//! 
//! The package assumes that you will just use the core opaquely, but the `"open"` feature exposes 
//! several debug features so that you can examine what is happening with the execution directly. 

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
pub(crate) use foundation::num::Wrapping;
use foundation::*;

mod chip;

/// The cpp mod contains FFI exports to create and access Machine objects in C++.
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

#[cfg(any(feature="open", doc))]
pub use chip::State;

#[cfg(any(feature="open", doc))]
pub mod internal {
    use super::*;
    pub use chip::access::*;
    pub use crate::chip::opcode::{Op::{self, *}, Flag::*, Test::*};
}

/// The Harness trait is the core of using this package; the `Machine` struct will use a 
/// type of your choosing that the chip can use to read 8-bit or 16-bit values from 16-bit 
/// addresses and write 8-bit or 16-bit values to 16-bit addresses, as well as read and write 
/// 8-bit values on 8-bit port numbers.
pub trait Harness {
    /// This is the most critical element; you can't build a usable machine without being 
    /// able to supply opcodes to the core via this operation.
    /// 
    /// This method returns the 8-bit value associated with the supplied 16-bit memory 
    /// address. It should generally be consistent with any writes made to the same address.
    fn read(&self, from: bits::u16) -> bits::u8;

    /// This is a convenience method; it takes care of reading a 16-bit word in little-endian 
    /// format from the specified address (less-signficant byte) and the subsequent address 
    /// (more-significant byte). 
    /// 
    /// You don't have to supply this method; it defaults to simply calling the `read` method 
    /// on two consecutive addresses and concatenating them together. You can implement this 
    /// to provide an optimized read operation, such as if your memory values are stored in a
    /// byte slice and you can just read adjacent indices.
    fn read_word(&self, from: bits::u16) -> bits::u16 {
        Wrapping(u16::from_le_bytes([self.read(from).0, self.read(from + Wrapping(1)).0]))
    }

    /// This method takes care of writing a byte to the specified address. It defaults to 
    /// silently discarding the supplied byte, for emulating read-only memory, but you will 
    /// usually want to supply your own implementation to record at least some variables.
    fn write(&mut self, to: bits::u16, value: bits::u8) { let _ = (value, to); }

    /// This is a convenience method; it takes care of writing a 16-bit word in little-endian 
    /// format to the specified address (less-signficant byte) and the subsequent address 
    /// (more-significant byte). 
    /// 
    /// You don't have to supply this method; it defaults to simply calling the `write` method 
    /// on two consecutive addresses from the two bytes in the argument. You can implement this 
    /// to provide an optimized write operation, such as if your memory values are stored in a
    /// byte slice and you can just write adjacent indices.
    fn write_word(&mut self, to: bits::u16, value: bits::u16) {
        for (index, byte) in value.0.to_le_bytes().into_iter().enumerate() {
            self.write(to + num::Wrapping(index as u16), num::Wrapping(byte))
        }
    }

    /// This method handles input operations. The CPU core can request/accept inputs on any 
    /// 8-bit port number. What values are supplied via what ports is entirely application-specific.
	fn input(&mut self, port: u8) -> bits::u8;

    /// This method handles output operations. The CPU core can publish/transmit outputs on any 
    /// 8-bit port number. What values are carried via what ports is entirely application-specific.
    fn output(&mut self, port: u8, value: bits::u8);


    /// This method reports to the Harness after every operation executed by the CPU, detailing the 
    /// operation executed and providing access to the current state of the CPU's internal registers
    /// and flags. 
    #[cfg(any(feature="open", doc))]
    fn did_execute(&mut self, client: &State, did: chip::opcode::Op) -> Result<Option<chip::opcode::Op>, string::String> { let _ = (client, did); Ok( None ) }
    
    /// You don't usually need to implement this method; it enables downcasting in cases where a 
    /// Machine stores a `dyn Harness` trait object.
    fn as_any(&self) -> Option<&dyn any::Any> { None }
}

/// SimpleBoard is a minimal Harness designed to make it easy to start using the crate; 
/// it just stores a full 16k RAM space and byte arrays to store the input and output port values.
/// You can address the RAM space by indexing the SimpleBoard directly.
#[repr(C)]
pub struct SimpleBoard {
	ram: [u8; 65536],
	pub port_out: [u8; 256],
	pub port_in: [u8; 256]
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
    fn write(&mut self, to: bits::u16, value: bits::u8) { self.ram[to.0 as usize] = value.0; }
    fn write_word(&mut self, to: bits::u16, value: bits::u16) {
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

/// This is the main outward-facing type for actually executing instructions. It also 
/// accepts interrupt requests, including RST instructions. It can be used as an Iterator
/// to do processing in between operations. It also forwards the contained Harness object 
/// out to receive method requests.
pub struct Machine<H: Harness + ?Sized, C: DerefMut<Target = H>> {
    chip: chip::State,
    board: C,
}

impl<H: Harness + ?Sized, C: DerefMut<Target = H>> Machine<H, C> {
    /// This associated function generates a new Machine using the provided Harness reference.
    /// It can accept any value that can be dereferenced to a mut Harness, whether dynamic or 
    /// generic, making it fairly easy to supply a `&mut H`, a `Box<H>` or `Box<dyn Harness>`.
    /// 
    /// >>> Future: Provide ways to use `RefCell<H>` or `Mutex<H>`.
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

#[cfg(feature="open")]
impl<H: Harness + ?Sized, C: DerefMut<Target = H>> AsRef<State> for Machine<H, C> {
    fn as_ref(&self) -> &State { &self.chip }
}

#[cfg(feature="open")]
impl<H: Harness + ?Sized, C: DerefMut<Target = H>> AsMut<State> for Machine<H, C> {
    fn as_mut(&mut self) -> &mut State { &mut self.chip }
}