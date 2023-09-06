use core::ops::DerefMut;
use crate::{num::NonZeroU8, String, Machine, Harness};
use super::{State, access::*};

pub mod opcode;
use opcode::{Op, Op::*};

pub type Failure = String;

#[cfg(debug_assertions)]
pub(super) type OpOutcome = Result<Option<NonZeroU8>, Failure>;
#[cfg(not(debug_assertions))]
pub(super) type OpOutcome = Option<NonZeroU8>;

impl<H: Harness, C: DerefMut<Target = H>> Machine<H, C> {
    #[cfg(debug_assertions)]
	pub fn execute(&mut self) -> OpOutcome {
		if !self.chip.active { return Ok(NonZeroU8::new(1)) };
        let (op, len) = Op::extract(self.board.deref_mut(), self.chip.pc)
            .map_err(|e| panic!("Couldn't extract opcode from {e:X} at {:#06X}", self.pc)).unwrap();
        self.chip.pc += len as u16; 
        let outcome = op.execute_on(&mut self.chip, self.board.deref_mut());
        match outcome {
            Ok(Some(_)) => (),
            _ => self.chip.active = false,
        };
        self.chip.active = self.board.did_execute(&self.chip)?;
        outcome
	}

    #[cfg(not(debug_assertions))]
	pub fn execute(&mut self) -> OpOutcome {
		if !self.active { return NonZeroU8::new(1) };
        let (op, len) = Op::extract(self.board.deref(), self.pc)
            .map_err(|e| panic!("Couldn't extract opcode from {e:X?}")).unwrap();
        self.pc += len as u16; 
        let elapsed = op.execute_on(&mut self.chip, self.board.deref_mut());
        if elapsed.is_none() { self. active = false; }
        elapsed
	}

    pub fn interrupt(&mut self, op: Op) -> Result<bool, opcode::Error> {
        if op.len() == 1 {
            Ok(self.interrupts && { 
                self.active = true; 
                self.interrupts = false; 
                let _ = op.execute_on(&mut self.chip, self.board.deref_mut()); 
                true 
            })
        } else {
            Err(opcode::Error::NotUsable(op))
        }
    }

    pub fn reset_to(&mut self, index: usize) -> Result<bool, opcode::OutOfRange> {
        match index {
            0..8 => Ok(self.interrupt(Reset{vector: index as u8}).ok().unwrap()),
            _ => Err(opcode::OutOfRange)
        }
    }
    }

impl Op {
    fn execute_on<H: Harness>(self, chip: &mut State, mut bus: impl DerefMut<Target = H>) -> OpOutcome {
        let cycles = match self {
            AddImmediate { value } => {
                let accumulator = &mut chip[Register::A];
                let aux = *accumulator ^ value;
                let (value, carry) = accumulator.overflowing_add(value);
                *accumulator = value;
                *chip.update_flags() = carry;
                chip.a = (value ^ aux) & 0x10 != 0;
                7
            }
            AndImmediate { value } => {
                chip[Register::A] &= value;
                *chip.update_flags() = false;
                7
            }
            Call{sub} => {
                bus.write_word(chip.pc, chip.push());
                chip.pc = sub;
                17
            }
            CallIf(test, sub) => if test.approves(chip) {
                Call{sub}.execute_on(chip, bus)?;
                17
            } else {
                11
            }
            ExchangeDoubleWithHilo => {
                let reg = &mut chip.register;
                (reg[2], reg[3], reg[4], reg[5]) = (reg[4], reg[5], reg[2], reg[3]);
                5
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
                chip[to] = value;
                10
            }
            MoveImmediate { value, to } => {
                match chip.resolve_byte(to) {
                    Byte::Single(register) => { chip[register] = value; 7 },
                    Byte::RAM(address) => { bus.write(value, address); 10},
                    Byte::Indirect => unreachable!()
                }
            }
            Push (source) => {
                let source = match source {
                    Word::OnBoard(internal) => chip[internal],
                    Word::ProgramStatus => chip.status(),
                    _ => unreachable!()
                };
                bus.write_word(source, chip.push());
                11
            }
            Reset{vector} => {
                bus.write_word(chip.pc, chip.push());
                chip.pc = vector as u16 * 8;
                11
            }
            ReturnIf(test) => {
                if test.approves(chip) {
                    chip.pc = bus.read_word(chip.pop());
                    11
                } else {
                    5
                }
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
    use access::{Double::*, Register::*};
    use crate::{chip::*, SimpleBoard};
    use opcode::{Test::*, Flag::*};
     #[test]
     fn add() {
        let mut harness = Socket::default();
        let mut chip = State::new();
        chip[Register::A] = 0x75;
        AddImmediate { value: 0x49 }.execute_on(&mut chip, &mut harness).unwrap();
        assert_eq!(chip.register[6], 0xBE);
        assert!(!chip.a, "aux carry was {}", chip.a);
        assert!(!chip.c, "carry was {}", chip.c);
        assert!(!chip.z, "zero bit was {}", chip.z);
        assert!(chip.m, "sign bit was {}", chip.m);
        assert!(chip.p, "parity bit was {}", chip.p);

        AddImmediate { value: 0x43 }.execute_on(&mut chip, &mut harness).unwrap();
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
        let mut chip = State::new();
        chip[Register::A] = 0b01011101;
        AndImmediate{ value: 0b11011011 }.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip.register[6], 0b01011001);
        assert!(!chip.a, "aux carry was {}", chip.a);
        assert!(!chip.c, "carry was {}", chip.c);
        assert!(!chip.z, "zero bit was {}", chip.z);
        assert!(!chip.m, "sign bit was {}", chip.m);
        assert!(chip.p, "parity bit was {}", chip.p);

        AndImmediate { value: 0b10100100 }.execute_on(&mut chip, &mut env).unwrap();
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
        let mut chip = State::new();
        chip.pc = 0x000C;
        chip.sp = 0x0100;
        let stack = chip.sp;
        env[stack] = 0x55;
        Call{sub: 0x00A2 }.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip.pc, 0x00A2);
        assert_eq!(chip.sp, 0x00FE);
        assert_eq!(env[0x00FE], 0x0C);
        assert_eq!(env[0x00FF], 0x00);
        assert_eq!(env[0x0100], 0x55);

        chip.register[6] = 0xC4;
        AddImmediate { value: 0x3C }.execute_on(&mut chip, &mut env).unwrap();
        CallIf(Not(Zero), 0x2000).execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip.pc, 0x00A2);
        assert_eq!(chip.sp, 0x00FE);
        assert_eq!(env[0x00FE], 0x0C);
        assert_eq!(env[0x00FF], 0x00);
        
        CallIf(Is(EvenParity), 0x1300).execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip.pc, 0x1300);
        assert_eq!(chip.sp, 0x00FC);
        assert_eq!(env[0x00FC], 0xA2);
        assert_eq!(env[0x00FD], 0x00);
    }

    #[test]
    fn jump() {
        let mut env = Socket::default();
        let mut chip = State::new();
        Jump{ to: 0x0340 }.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip.pc, 0x0340);
        chip.register[6] = 0x90;
        AddImmediate { value: 0x73 }.execute_on(&mut chip, &mut env).unwrap();
        JumpIf(Not(Carry), 0x1203).execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip.pc, 0x0340);
        assert!(!chip.m, "MINUS flag was {} after result {}", chip.m, chip.register[6]);
        JumpIf(Not(Negative), 0x5432).execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip.pc, 0x5432);
    }

    #[test]
    fn load_xi() {
        let mut env = Socket::default();
        let mut chip = State::new();
        LoadExtendedImmediate { to: Internal::Wide(HL), value: 0x6472 }.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip.register[4], 0x72);
        assert_eq!(chip.register[5], 0x64);
    }

    #[test]
    fn push() {
        let mut env = SimpleBoard::default();
        let mut chip = State::new();
        chip.sp = 0x4000;
        chip[BC] = 0x3256;
        chip[DE] = 0x2345;
        chip[HL] = 0x7654;
        Push(Word::OnBoard(Internal::Wide(HL))).execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(env[0x3FFE], 0x54);
        assert_eq!(env[0x3FFF], 0x76);
        assert_eq!(chip.sp, 0x3FFE);

        chip.register[6] = 0x90;
        AddImmediate { value: 0x73 }.execute_on(&mut chip, &mut env).unwrap();
        Push(Word::ProgramStatus).execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip.sp, 0x3FFC);
        assert_eq!(env[0x3FFC], 0x03);
        assert_eq!(env[0x3FFD], 0b00000111);
    }

    #[test]
    fn reset() {
        let mut env = SimpleBoard::default();
        let mut chip = State::new();
        chip.pc = 0x0391;
        chip.sp = 0x0200;
        Reset{vector: 0x05}.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip.pc, 0x0028);
        assert_eq!(chip.sp, 0x01FE);
        assert_eq!(env[0x01FE], 0x91);
        assert_eq!(env[0x01FF], 0x03);
    }

    #[test]
    fn exchange() {
        let mut env = Socket::default();
        let mut chip = State::new();
        chip[D] = 0x2B;
        chip[E] = 0x43;
        chip[H] = 0xD1;
        chip[L] = 0x6C;
        ExchangeDoubleWithHilo.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip[DE], 0xD16C);
        assert_eq!(chip[HL], 0x2B43);
    }

    #[test]
    fn move_i() {
        let mut env = SimpleBoard::default();
        let mut chip = State::new();
        chip[HL] = 0x0421;
        MoveImmediate { value: 0x02, to: Byte::Single(H) }.execute_on(&mut chip, &mut env).unwrap();
        MoveImmediate { value: 0x72, to: Byte::Indirect }.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(env[0x0221], 0x72);
    }
}