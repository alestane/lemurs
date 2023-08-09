#![no_std]
#![feature(split_array)]

#[cfg(feature="std")]
mod foundation {
    extern crate std;
    pub use std::{boxed, vec};
}
#[cfg(not(feature="std"))]
mod foundation {
    extern crate alloc;
    pub use alloc::{boxed, vec};
}
use foundation::*;
pub use boxed::Box;

pub const VRAM_ADDRESS: usize = 9216;
pub const VRAM_SIZE: usize = 7168;

mod chip;

pub use chip::{State, Zone};
