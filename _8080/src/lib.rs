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
    pub use std::{any, array, borrow, boxed, convert, num, ops, rc, result, slice, string, sync, vec};
}
#[cfg(not(feature="std"))]
mod foundation {
    extern crate alloc;
    pub use alloc::{boxed, rc, string, vec};
    pub use core::{array, borrow, convert, num, result, ops, slice, any};
}
use foundation::num::Wrapping;
use foundation::*;
#[allow(unused_imports)]
use self::boxed::Box;
use self::rc::Rc;

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
    fn did_execute(&mut self, client: &chip::State, did: chip::opcode::Op) -> Result<Option<chip::opcode::Op>, string::String> { let _ = (client, did); Ok( None ) }

    /// You don't usually need to implement this method; it enables downcasting in cases where a
    /// Machine stores a `dyn Harness` trait object.
    fn as_any(&self) -> Option<&dyn any::Any> { None }
}

use crate::sync::{Arc, Mutex};
use core::{borrow::BorrowMut, cell::RefCell, marker::PhantomData, ops::{Deref, DerefMut}};

type Shared<H, C> = Rc<RefCell<(C, PhantomData<H>)>>;

impl<H: Harness + ?Sized, C: BorrowMut<H>> Harness for Shared<H, C> {
	fn read(&self, address: bits::u16) -> bits::u8 { self.deref().borrow().0.borrow().read(address) }
	fn read_word(&self, address: bits::u16) -> bits::u16 { self.deref().borrow().0.borrow().read_word(address) }
	fn write(&mut self, address: bits::u16, value: bits::u8) { (**self).borrow_mut().0.borrow_mut().write(address, value) }
	fn write_word(&mut self, address: bits::u16, value: bits::u16) { (**self).borrow_mut().0.borrow_mut().write_word(address, value) }
	fn input(&mut self, port: u8) -> bits::u8 { (**self).borrow_mut().0.borrow_mut().input(port) }
	fn output(&mut self, port: u8, value: bits::u8) { (**self).borrow_mut().0.borrow_mut().output(port, value) }
	#[cfg(feature="cfg")]
	fn did_execute(&mut self, client: &chip::State, did: chip::opcode::Op) -> Result<Option<chip::opcode::Op>, string::String> {
		(**self).borrow_mut().0.borrow_mut().did_execute(client, did)
	}
}

type Synced<H, C> = Arc<Mutex<(C, PhantomData<H>)>>;

impl<H: Harness + ?Sized, C: BorrowMut<H>> Harness for Synced<H, C> {
	fn read(&self, address: bits::u16) -> bits::u8 { self.deref().lock().unwrap().0.borrow().read(address) }
	fn read_word(&self, address: bits::u16) -> bits::u16 { self.deref().lock().unwrap().0.borrow().read_word(address) }
	fn write(&mut self, address: bits::u16, value: bits::u8) { (**self).lock().unwrap().0.borrow_mut().write(address, value) }
	fn write_word(&mut self, address: bits::u16, value: bits::u16) { (**self).lock().unwrap().0.borrow_mut().write_word(address, value) }
	fn input(&mut self, port: u8) -> bits::u8 { (**self).lock().unwrap().0.borrow_mut().input(port) }
	fn output(&mut self, port: u8, value: bits::u8) { (**self).lock().unwrap().0.borrow_mut().output(port, value) }
	#[cfg(feature="cfg")]
	fn did_execute(&mut self, client: &chip::State, did: chip::opcode::Op) -> Result<Option<chip::opcode::Op>, string::String> {
		(**self).lock().unwrap().0.borrow_mut().did_execute(client, did)
	}
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

mod simple;

/// This is the main outward-facing type for actually executing instructions. It also
/// accepts interrupt requests, including RST instructions. It can be used as an Iterator
/// to do processing in between operations. It also forwards the contained Harness object
/// out to receive method requests.
pub struct Machine<H: Harness + ?Sized, C: BorrowMut<H>> {
    chip: chip::State,
    board: C,
    _grammar: PhantomData<H>,
}

impl<H: Harness + ?Sized, C: BorrowMut<H>> Machine<H, C> {
	pub fn new(board: C) -> Self {
		Self { board, chip: chip::State::new(), _grammar: PhantomData::default() }
	}

	fn split_mut(&mut self) -> (&mut chip::State, &mut H) { (&mut self.chip, self.board.borrow_mut() )}
}

impl<H: Harness + ?Sized, C: BorrowMut<H>> Deref for Machine<H, C> {
	type Target = H;
	fn deref(&self) -> &Self::Target { self.board.borrow() }
}

impl<H: Harness + ?Sized, C: BorrowMut<H>> DerefMut for Machine<H, C> {
	fn deref_mut(&mut self) -> &mut Self::Target { self.board.borrow_mut() }
}

pub struct Install<H: Harness + ?Sized>(PhantomData<H>);

impl<H: Harness + ?Sized> Install<H> {
    /// This associated function generates a new Machine using the provided Harness reference.
    /// It can accept any value that can be dereferenced to a mut Harness, whether dynamic or
    /// generic, making it fairly easy to supply a `&mut H`, a `Box<H>` or `Box<dyn Harness>`.
    ///
    /// >>> Future: Provide ways to use `RefCell<H>` or `Mutex<H>`.
    pub fn new<C: BorrowMut<H>>(board: C) -> Machine<H, C> {
        Machine::new( board )
    }

    pub fn new_shared<C: BorrowMut<H>>(board: C) -> Machine<Shared<H, C>, Shared<H, C>> {
    	Machine::new(Rc::new(RefCell::new( (board, PhantomData::default()) )))
    }

    pub fn new_synced<C: BorrowMut<H>>(board: C) -> Machine<Synced<H, C>, Synced<H, C>> {
    	Machine::new(Arc::new(Mutex::new( (board, PhantomData::default()) )))
    }
}

#[cfg(feature="open")]
impl<H: Harness + ?Sized, C: BorrowMut<H>> AsRef<chip::State> for Machine<H, C> {
    fn as_ref(&self) -> &chip::State { &self.chip }
}

#[cfg(feature="open")]
impl<H: Harness + ?Sized, C: BorrowMut<H>> AsMut<chip::State> for Machine<H, C> {
    fn as_mut(&mut self) -> &mut chip::State { &mut self.chip }
}
