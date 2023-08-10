#![no_std]

use _8080::{State, Box};

use core::convert::TryFrom;

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
pub unsafe extern "C" fn state_outputs(state: *const State) -> *const u8 {
    &(*state)[Zone::Out][0] as *const u8
}

#[no_mangle]
pub unsafe extern "C" fn state_inputs(state: *mut State) -> *mut u8 {
    &mut (*state)[Zone::In][0] as *mut u8
}

#[no_mangle]
pub unsafe extern "C" fn state_ram(state: *const State) -> *const u8 {
    &(*state)[Zone::RAM][0] as *const u8
}

#[no_mangle]
pub unsafe extern "C" fn state_execute(state: *mut State) -> u8 {
    state.execute()
}