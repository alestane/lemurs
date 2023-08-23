#![no_std]

#[cfg(all(not(test), any(feature="cpp_panic",feature="cpp_alloc",feature="cpp_fmt")))]
extern crate cpp;

use _8080::{State, Box, Zone};

use core::{array, convert::TryFrom};

#[no_mangle]
pub unsafe extern "C" fn entrust_blank_state(memory: usize) -> *mut State {
    Box::into_raw(Box::new(State::with_ram(u16::try_from(memory).unwrap_or(0))))
}

#[no_mangle]
pub unsafe extern "C" fn entrust_state_from(memory: usize, source: *const u8) -> *mut State {
    Box::into_raw(Box::new(State::from(core::slice::from_raw_parts(source, memory))))
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
pub unsafe extern "C" fn state_register_debug(state: *mut State, op: extern "C" fn(*const [u8;1], u16, u16, u8) -> bool) {
    if let Some(state) = state.as_mut() {
        state.add_callback(move |ram, addr, offset, switch| op(array::from_ref(&ram[0]) as *const _, addr, offset, switch)) 
    };
}

#[no_mangle]
pub unsafe extern "C-unwind" fn state_execute(state: *mut State) -> Option<core::num::NonZeroU8> {
    for cycles in state.as_mut()
        .expect("null or invalid state pointer")
        .execute() 
    {
        return Some(cycles);
    }
    None
}