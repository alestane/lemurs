use crate::{num::NonZeroU8, String};
use super::{State, access::*};

pub mod opcode;
use opcode::{Op, Op::*};

pub type Failure = Result<String, String>;

#[cfg(debug_assertions)]
pub(super) type OpOutcome = Result<Option<NonZeroU8>, Failure>;
#[cfg(not(debug_assertions))]
pub(super) type OpOutcome = Option<NonZeroU8>;

impl State<'_> {
    #[cfg(debug_assertions)]
	pub fn execute(&mut self) -> OpOutcome {
		if !self.active { return Ok(NonZeroU8::new(1)) };
        let (op, len) = Op::extract(&self.board.deref()[self.pc as usize..])
            .map_err(|e| panic!("Couldn't extract opcode from {e:?}")).unwrap();
        self.pc += len as u16; 
        let outcome = op.execute_on(self);
        match outcome {
            Ok(Some(_)) => (),
            _ => self.active = false,
        }
        self.board.did_execute(self)?;
        outcome
	}

    #[cfg(not(debug_assertions))]
	pub fn execute(&mut self) -> OpOutcome {
		if !self.active { return NonZeroU8::new(1) };
        let (op, len) = Op::extract(&self.board.deref()[self.pc as usize..])
            .map_err(|e| panic!("Couldn't extract opcode from {e:?}")).unwrap();
        self.pc += len as u16; 
        let elapsed = op.execute_on(self);
        if elapsed.is_none() { self. active = false; }
        elapsed
	}

    pub fn interrupt(&mut self, op: Op) -> Result<bool, opcode::Error> {
        if op.len() == 1 {
            Ok(self.interrupts && { 
                self.active = true; 
                self.interrupts = false; 
                let _ = op.execute_on(self); 
                true 
            })
        } else {
            Err(opcode::Error::NotUsable(op))
        }
    }

    pub fn reset_to(&mut self, index: usize) -> Result<bool, opcode::OutOfRange> {
        match index {
            0..8 => Ok(self.interrupt(Op::Reset{vector: index as u8}).ok().unwrap()),
            _ => Err(opcode::OutOfRange)
        }
    }
}

impl Op {
    fn execute_on(self, chip: &mut State) -> OpOutcome {
        let cycles = match self {
            Call{sub} => {
                *chip <<= (Word::Stack, chip.pc);
                chip.pc = sub;
                17
            }
            Jump{to} => {
                chip.pc = to;
                10
            }
            Reset{vector} => {
                *chip <<= (Word::Stack, chip.pc);
                chip.pc = vector as u16 * 8;
                11
            }
            NOP(n) => n,
            #[cfg(debug_assertions)]
            _ => unimplemented!("Op {self:?} not implemented yet")
        };
        let cycles = NonZeroU8::new(cycles);
        #[cfg(debug_assertions)]
        let cycles = Ok(cycles);
        cycles
    }
}