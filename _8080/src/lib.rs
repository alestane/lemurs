#![no_std]
#![feature(split_array)]
#![feature(exclusive_range_pattern)]
#![feature(trait_alias)]
#![feature(iter_intersperse)]

#[cfg(feature="std")]
mod foundation {
    extern crate std;
    pub use std::{boxed, vec, array, convert, num, string, result, ops};
}
#[cfg(not(feature="std"))]
mod foundation {
    extern crate alloc;
    pub use alloc::{boxed, vec, string};
    pub use core::{array, convert, num, result, ops};
}
use foundation::{*, string::String, result::Result};
use foundation::vec::Vec;
pub use boxed::Box;

mod chip;

pub use chip::{State, Debugger, Zone};
