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
            .map_err(|e| panic!("Couldn't extract opcode from {e:X} at {:#06X}", self.pc)).unwrap();
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
            .map_err(|e| panic!("Couldn't extract opcode from {e:X?}")).unwrap();
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
            AddImmediate { value } => {
                let accumulator = &mut chip[Byte::Single(Register::A)];
                let aux = (*accumulator & 0x0F) + (value & 0x0F) > 0x0F;
                let sign = (*accumulator | value) & 0x80;
                *accumulator = accumulator.wrapping_add(value);
                *chip.update_flags() = (sign != 0) && *accumulator & 0x80 == 0;
                chip.a = aux;
                7
            }
            AndImmediate { value } => {
                chip[Byte::Single(Register::A)] &= value;
                *chip.update_flags() = false;
                7
            }
            Call{sub} => {
                *chip <<= (Word::Stack, chip.pc);
                chip.pc = sub;
                17
            }
            CallIf(test, sub) => if test.approves(chip) {
                Call{sub}.execute_on(chip)?;
                17
            } else {
                11
            }
            Jump{to} => {
                chip.pc = to;
                10
            }
            JumpIf(test, addr) => {
                if test.approves(chip) { chip.pc = addr; }
                10
            }
            LoadExtendedImmediate { to, value } => {
                *chip <<= (to, value);
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{chip::*, SimpleBoard};
    use opcode::{Test::*, Flag::*};
     #[test]
     fn add() {
        let mut harness = Socket::default();
        let mut chip = State::with(&mut harness);
        chip[Byte::Single(Register::A)] = 0x75;
        AddImmediate { value: 0x49 }.execute_on(&mut chip).unwrap();
        assert_eq!(chip.register[6], 0xBE);
        assert!(!chip.a, "aux carry was {}", chip.a);
        assert!(!chip.c, "carry was {}", chip.c);
        assert!(!chip.z, "zero bit was {}", chip.z);
        assert!(chip.m, "sign bit was {}", chip.m);
        assert!(chip.p, "parity bit was {}", chip.p);

        AddImmediate { value: 0x43 }.execute_on(&mut chip).unwrap();
        assert_eq!(chip.register[6], 0x01, "Sum was {}", chip.register[6]);
        assert!(chip.a, "aux carry was {}", chip.a);
        assert!(chip.c, "carry was {}", chip.c);
        assert!(!chip.z, "zero bit was {}", chip.z);
        assert!(!chip.m, "sign bit was {}", chip.m);
        assert!(!chip.p, "parity bit was {}", chip.p);
     }

     #[test]
     fn and() {
        let mut env = Socket::default();
        let mut chip = State::with(&mut env);
        chip[Byte::Single(Register::A)] = 0b01011101;
        AndImmediate{ value: 0b11011011 }.execute_on(&mut chip).unwrap();
        assert_eq!(chip.register[6], 0b01011001);
        assert!(!chip.a, "aux carry was {}", chip.a);
        assert!(!chip.c, "carry was {}", chip.c);
        assert!(!chip.z, "zero bit was {}", chip.z);
        assert!(!chip.m, "sign bit was {}", chip.m);
        assert!(chip.p, "parity bit was {}", chip.p);

        AndImmediate { value: 0b10100100 }.execute_on(&mut chip).unwrap();
        assert_eq!(chip.register[6], 0b00000000);
        assert!(!chip.a, "aux carry was {}", chip.a);
        assert!(!chip.c, "carry was {}", chip.c);
        assert!(chip.z, "zero bit was {}", chip.z);
        assert!(!chip.m, "sign bit was {}", chip.m);
        assert!(chip.p, "parity bit was {}", chip.p);
     }

     #[test]
    fn call() {
        let mut env = SimpleBoard::default();
        let mut chip = State::with(&mut env);
        chip.pc = 0x000C;
        chip.sp = 0x0100;
        let stack = chip.sp;
        chip[stack] = 0x55;
        Call{sub: 0x00A2 }.execute_on(&mut chip).unwrap();
        assert_eq!(chip.pc, 0x00A2);
        assert_eq!(chip.sp, 0x00FE);
        assert_eq!(chip[0x00FE], 0x0C);
        assert_eq!(chip[0x00FF], 0x00);
        assert_eq!(chip[0x0100], 0x55);

        chip.register[6] = 0xC4;
        AddImmediate { value: 0x3C }.execute_on(&mut chip).unwrap();
        CallIf(Not(Zero), 0x2000).execute_on(&mut chip).unwrap();
        assert_eq!(chip.pc, 0x00A2);
        assert_eq!(chip.sp, 0x00FE);
        assert_eq!(chip[0x00FE], 0x0C);
        assert_eq!(chip[0x00FF], 0x00);
        
        CallIf(Is(EvenParity), 0x1300).execute_on(&mut chip).unwrap();
        assert_eq!(chip.pc, 0x1300);
        assert_eq!(chip.sp, 0x00FC);
        assert_eq!(chip[0x00FC], 0xA2);
        assert_eq!(chip[0x00FD], 0x00);
    }

    #[test]
    fn jump() {
        let mut env = Socket::default();
        let mut chip = State::with(&mut env);
        Jump{ to: 0x0340 }.execute_on(&mut chip).unwrap();
        assert_eq!(chip.pc, 0x0340);
        chip.register[6] = 0x90;
        AddImmediate { value: 0x73 }.execute_on(&mut chip).unwrap();
        JumpIf(Not(Carry), 0x1203).execute_on(&mut chip).unwrap();
        assert_eq!(chip.pc, 0x0340);
        assert!(!chip.m, "MINUS flag was {} after result {}", chip.m, chip.register[6]);
        JumpIf(Not(Negative), 0x5432).execute_on(&mut chip).unwrap();
        assert_eq!(chip.pc, 0x5432);
    }

    #[test]
    fn load_xi() {
        let mut env = Socket::default();
        let mut chip = State::with(&mut env);
        LoadExtendedImmediate { to: Word::Wide(Double::HL), value: 0x6472 }.execute_on(&mut chip).unwrap();
        assert_eq!(chip.register[4], 0x72);
        assert_eq!(chip.register[5], 0x64);
    }
}