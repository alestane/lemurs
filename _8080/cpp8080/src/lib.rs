#![no_std]

extern crate cpp_panic;
extern crate cpp_alloc;

use _8080::{State, Box, Zone};

use core::{array, convert::TryFrom};

#[no_mangle]
pub unsafe extern "C" fn new_empty_state(memory: usize) -> *mut State {
    Box::into_raw(State::new_with_ram(u16::try_from(memory).unwrap_or(0)))
}

#[no_mangle]
pub unsafe extern "C" fn new_state_with(memory: usize, source: *const u8) -> *mut State {
    Box::into_raw(State::new_with_rom(core::slice::from_raw_parts(source, memory)))
}

#[no_mangle]
pub unsafe extern "C" fn discard_state(state: *mut State) {
    drop(Box::from_raw(state));
}

#[no_mangle]
pub unsafe extern "C" fn state_outputs(state: *const State) -> Option<&'static [u8;1]> {
    state.as_ref()
        .map(|state| array::from_ref(&state[Zone::Out][0]))
}

#[no_mangle]
pub unsafe extern "C" fn state_inputs(state: *mut State) -> Option<&'static mut [u8;1]> {
    state.as_mut()
        .map(|state|array::from_mut(&mut state[Zone::In][0]))
}

#[no_mangle]
pub unsafe extern "C" fn state_ram(state: *const State) -> Option<&'static [u8;1]> {
    state.as_ref()
        .map(|state| array::from_ref(&state[Zone::RAM][0]))
}

#[cfg(debug_assertions)]
#[no_mangle]
pub unsafe extern "C" fn state_register_debug(state: *mut State, op: extern "C" fn(&'static [u8;1], u16, u16, u8) -> bool) {
    if let Some(state) = state.as_mut() {
        state.register(op)
    };
}

#[no_mangle]
pub unsafe extern "C-unwind" fn state_execute(state: *mut State) -> u8 {
    state.as_mut()
    .expect("null or invalid state pointer")
    .execute()
}