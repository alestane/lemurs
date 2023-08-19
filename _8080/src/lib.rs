#![no_std]
#![feature(split_array)]
#![feature(exclusive_range_pattern)]

#[cfg(feature="std")]
mod foundation {
    extern crate std;
    pub use std::{boxed, vec, array};
}
#[cfg(not(feature="std"))]
mod foundation {
    extern crate alloc;
    pub use alloc::{boxed, vec};
    pub use core::array;
}
use foundation::*;
pub use boxed::Box;

mod chip;

pub use chip::{State, Debugger, Zone};
