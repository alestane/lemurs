use crate::array;
use super::{State, access::*};

pub enum Op {
    NOP,
    Call(u16),
}

use Op::*;

#[cfg(debug_assertions)]
fn check_listeners(chip: &mut State, addr: u16) -> bool {
    unsafe {
        let ram = array::from_ref(&chip.ram[0]) as *const [u8;1];
        let offset = Word::DE << chip;
        let switch = chip[Byte::C];
        chip.callbacks.iter().any(|op| op(&*ram, addr, offset, switch))
    }
}

impl Op {
    fn execute_on(self, chip: &mut State) -> u8 {
        match self {
            Call(addr) => {
                #[cfg(debug_assertions)]
                if check_listeners(chip, addr) { return 0; }
                chip.sp -= 2;
                *chip <<= (Word::Stack, chip.pc);
                chip.pc = addr;
                17
            }
            _nop => 0
        }
    }
}