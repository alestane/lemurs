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

mod chip;

pub use chip::{State, Zone};
