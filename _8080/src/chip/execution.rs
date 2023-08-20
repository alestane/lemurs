use crate::array;
use super::{State, access::*};
use core::{convert::TryFrom, num::NonZeroU8};

pub enum Op {
    NOP(u8),
    Call{sub: u16},
    Reset{vector: u8},
}

impl Op {
    fn len(&self) -> u8 {
        match self {
            Call{..} => 3,
            _ => 1,
        }
    }
}

#[repr(u8)]
enum B11_000_111 {
    Reset = 0b11_000_111,
}

impl TryFrom<u8> for B11_000_111 {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value & 0b11_000_111 {
            n if n == Self::Reset as u8 => Ok(Self::Reset),
            _ => Err(value),
        }
    }
}

pub struct OutOfRange;
pub enum BadOpcode {
    NotUsable(Op),
    Mismatch(Op, u8),
    Invalid([u8;1]),
    InvalidPair([u8;2]),
    InvalidTriple([u8;3]),
}

impl Op {
    fn extract(value: &[u8]) -> Result<(Self, usize), BadOpcode> {
        Err(BadOpcode::Invalid([0]))
    }
}

use Op::*;

impl State {
	pub fn execute(&mut self) -> Option<NonZeroU8> {
		if !self.active { return None };
        let (op, len) = Op::extract(&self.ram[self.pc as usize..]).ok()?;
        self.pc += len as u16; 
        let elapsed = op.execute_on(self);
        if self.pc as usize >= self.ram.len() { self.active = false };
        elapsed
	}

    pub fn interrupt(&mut self, op: Op) -> Result<bool, BadOpcode> {
        if op.len() == 1 {
            Ok(self.interrupts && { 
                self.active = true; 
                self.interrupts = false; 
                op.execute_on(self); 
                true 
            })
        } else {
            Err(BadOpcode::NotUsable(op))
        }
    }

    pub fn reset_to(&mut self, index: usize) -> Result<bool, OutOfRange> {
        match index {
            0..8 => Ok(self.interrupt(Op::Reset{vector: index as u8}).ok().unwrap()),
            _ => Err(OutOfRange)
        }
    }
}

#[cfg(debug_assertions)]
fn check_listeners(chip: &mut State, addr: u16) -> bool {
    unsafe {
        let ram = array::from_ref(&chip.ram[0]) as *const [u8;1];
        let offset = Double::DE << &*chip;
        let switch = chip[Register::C];
        chip.callbacks.iter().copied().any(|op| op(&*ram, addr, offset, switch))
    }
}

impl Op {
    fn execute_on(self, chip: &mut State) -> Option<NonZeroU8> {
        let cycles = match self {
            Call{sub} => {
                #[cfg(debug_assertions)]
                if check_listeners(chip, sub) { return None; }
                *chip <<= (Word::Stack, chip.pc);
                chip.pc = sub;
                17
            }
            Reset{vector} => {
                *chip <<= (Word::Stack, chip.pc);
                chip.pc = vector as u16 * 8;
                11
            }
            NOP(n) => n,
        };
        NonZeroU8::new(cycles)
    }
}