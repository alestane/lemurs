#[cfg(not(test))]
extern crate cpp;

use crate::boxed::Box;
use crate::chip::opcode::Op;

use core::{borrow::{Borrow, BorrowMut}, marker::PhantomData, num::Wrapping};

#[repr(C)]
struct Harness (u8, PhantomData<dyn crate::Harness>);

mod safe {
    use super::*;
    pub (super) enum FreePtr {
        External(*mut dyn crate::Harness),
        Owned(Box<dyn crate::Harness>),
    }
    impl FreePtr {
        pub fn new_unowned(reference: &mut (dyn crate::Harness + 'static)) -> Self {
            Self::External(reference as *mut _)
        }
        pub fn new_owned(container: impl crate::Harness + 'static) -> Self {
            Self::Owned(Box::new(container))
        }
    }
    impl Borrow<dyn crate::Harness> for FreePtr {
        fn borrow(&self) -> &(dyn crate::Harness + 'static) {
            match self {
                Self::External(ptr) => unsafe { &**ptr },
                Self::Owned(ref ptr) => &**ptr, 
            }
        }
    }
    impl BorrowMut<dyn crate::Harness> for FreePtr {
        fn borrow_mut(&mut self) -> &mut (dyn crate::Harness + 'static) {
            match self {
                Self::External(ptr) => unsafe { &mut **ptr },
                Self::Owned(ref mut ptr) => &mut **ptr, 
            }
        }
    }
}

type Machine = crate::Machine<dyn crate::Harness, safe::FreePtr>;

#[no_mangle]
extern "C" fn create_machine<'a>(board: Option<&mut Harness>) -> *mut Machine {
    Box::into_raw(Box::new(match board {
        Some(ext) => Machine::new(safe::FreePtr::new_unowned(ext)),
        None => Machine::new(safe::FreePtr::new_owned(crate::SimpleBoard::default())),
    }))
}

#[no_mangle]
extern "C" fn request_default_impl(host: &Machine) -> Option<&crate::SimpleBoard> {
    host.as_any()?.downcast_ref()
}

#[no_mangle]
#[cfg(feature="open")]
extern "C" fn machine_state(host: &Machine) -> &crate::State {
    host.as_ref()
}

#[no_mangle]
#[cfg(not(feature="open"))]
extern "C-unwind" fn machine_execute(host: &mut Machine) -> u8 {
    match host.execute() {
        None => 0,
        Some(byte) => byte.get(),
    }
}

#[no_mangle]
#[cfg(feature="open")]
extern "C-unwind" fn machine_execute(host: &mut Machine) -> u8 {
    match host.execute() {
        Ok(None) => 0,
        Ok(Some(byte)) => byte.get(),
        Err(string) => panic!("{string}"),
    }
}

#[no_mangle]
extern "C-unwind" fn machine_interrupt(host:&mut Machine, code: u8) -> bool {
    unsafe { host.interrupt(Op::extract(core::iter::once(code)).expect("not a valid interrupt code.").0).unwrap_unchecked() }
}

#[no_mangle]
extern "C" fn discard_machine(state: *mut Machine) {
    drop(unsafe{Box::from_raw(state)});
}

extern "C-unwind" {
    fn read_harness(host: &Harness, address: Wrapping<u16>) -> Wrapping<u8>;
    fn read_word_harness(host: &Harness, address: Wrapping<u16>) -> Wrapping<u16>;
    fn write_harness(host: &mut Harness, address: Wrapping<u16>, value: Wrapping<u8>);
    fn write_word_harness(host: &mut Harness, address: Wrapping<u16>, value: Wrapping<u16>);
    fn input_harness(host: &Harness, port: u8) -> Wrapping<u8>;
    fn output_harness(host: &mut Harness, port: u8, value: Wrapping<u8>);
    #[cfg(feature="open")]
    fn did_execute_harness(host: &mut Harness, chip: &crate::State, op: u32) -> Option<&'static [u8;4]>;
}

impl crate::Harness for Harness {
    fn read(&self, from: Wrapping<u16>) -> Wrapping<u8> {
        unsafe { read_harness(self, from) }
    }
    fn read_word(&self, from: Wrapping<u16>) -> Wrapping<u16> {
        unsafe { read_word_harness(self, from) }
    }
    fn write(&mut self, to: Wrapping<u16>, value: Wrapping<u8>) {
        unsafe { write_harness(self, to, value) }
    }
    fn write_word(&mut self, to: Wrapping<u16>, value: Wrapping<u16>) {
        unsafe { write_word_harness(self, to, value) }
    }
    fn input(&mut self, port: u8) -> Wrapping<u8> {
        unsafe { input_harness(self, port) }
    }
    fn output(&mut self, port: u8, value: Wrapping<u8>) {
        unsafe { output_harness(self, port, value) }
    }
    #[cfg(feature="open")]
    fn did_execute(&mut self, client: &crate::State, did: Op) -> Result<Option<Op>, crate::string::String> {
        use core::{mem::transmute, ffi::CStr};
        unsafe {
            match did_execute_harness(self, client, u32::from_be_bytes(did.into())) {
                None => Ok(None),
                Some(code@&[0, ..]) => Ok(Some(Op::extract(code[1..].iter().copied()).or(Err(crate::string::String::from("Not a valid opcode")))?.0)),
                Some(bytes) => Err(CStr::from_ptr(transmute(bytes)).to_string_lossy().into_owned()),
            }
        }
    }
}
