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
        if let Some(action) = self.board.did_execute(&self.chip, op)? {
            action.execute_on(&mut self.chip, self.board.deref_mut()).unwrap();
        }
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
            AddTo { value, carry } => {
                let carry_in = chip.c && carry;
                let accumulator = &mut chip[Register::A];
                let aux = *accumulator ^ value;
                let (value, carry) = accumulator.overflowing_add(value.wrapping_add(carry_in as u8));
                *accumulator = value;
                *chip.update_flags() = carry;
                chip.a = (value ^ aux) & 0x10 != 0;
                7
            }
            AndWith { value } => {
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
            CompareWith{value} => {
                let base = chip[Register::A];
                let (comparison, borrow) = base.overflowing_sub(value);
                *chip.update_flags_for(comparison) = borrow;
                7
            }
            ExchangeDoubleWithHilo => {
                (chip[Double::DE], chip[Double::HL]) = (chip[Double::HL], chip[Double::DE]);
                5
            }
            ExchangeTopWithHilo => {
                let out = chip[Double::HL];
                chip[Double::HL] = bus.read_word(chip.sp);
                bus.write_word(out, chip.sp);
                18
            }
            ExclusiveOrWith { value } => {
                chip[Register::A] ^= value;
                *chip.update_flags() = false;
                7
            }
            Halt => {
                chip.active = false;
                7
            }
            Jump{to} => {
                chip.pc = to;
                10
            }
            JumpIf(test, addr) => {
                if test.approves(chip) { chip.pc = addr; }
                10
            }
            LoadExtendedWith { to, value } => {
                chip[to] = value;
                10
            }
            Move{to, from} => {
                let (to, from) = (chip.resolve_byte(to), chip.resolve_byte(from));
                match (to, from) {
                    (Byte::Single(to), Byte::Single(from)) => {
                        chip[to] = chip[from];
                        5
                    }
                    (Byte::RAM(address), Byte::Single(from)) => {
                        bus.write(chip[from], address);
                        7
                    }
                    (Byte::Single(to), Byte::RAM(address)) => {
                        chip[to] = bus.read(address);
                        7
                    }
                    _ => unreachable!()
                }
            }
            MoveData { value, to } => {
                match chip.resolve_byte(to) {
                    Byte::Single(register) => { chip[register] = value; 7 },
                    Byte::RAM(address) => { bus.write(value, address); 10},
                    Byte::Indirect => unreachable!()
                }
            }
            OrWith{value} => {
                chip[Register::A] |= value;
                *chip.update_flags() = false;
                7
            }
            Pop(target) => {
                match target {
                    Word::OnBoard(internal) => chip[internal] = bus.read_word(chip.pop()),
                    Word::ProgramStatus => {
                        let [accumulator, status] = bus.read_word(chip.pop()).to_le_bytes();
                        chip[Register::A] = accumulator;
                        chip.extract_flags(status);
                    }
                    _ => unreachable!()
                };
                10
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
            Return => {
                chip.pc = bus.read_word(chip.pop());
                10
            }
            ReturnIf(test) => {
                if test.approves(chip) {
                    Return.execute_on(chip, bus)?;
                    11
                } else {
                    5
                }
            }
            RotateRightCarrying => {
                let accumulator = chip[Register::A];
                chip.c = accumulator & 0x01 != 0;
                chip[Register::A] = accumulator.rotate_right(1);
                4
            }
            SubtractBy{ value, carry } => {
                let carry = chip.c && carry;
                let accumulator = &mut chip[Register::A];
                let aux = *accumulator ^ value;
                let (value, carry) = accumulator.overflowing_sub(value.wrapping_add(carry as u8));
                *accumulator = value;
                *chip.update_flags() = carry;
                chip.a = (value ^ aux) & 0x10 == 0;
                7
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
        AddTo { value: 0x49, carry: false }.execute_on(&mut chip, &mut harness).unwrap();
        assert_eq!(chip.register[6], 0xBE);
        assert!(!chip.a, "aux carry was {}", chip.a);
        assert!(!chip.c, "carry was {}", chip.c);
        assert!(!chip.z, "zero bit was {}", chip.z);
        assert!(chip.m, "sign bit was {}", chip.m);
        assert!(chip.p, "parity bit was {}", chip.p);

        AddTo { value: 0x43, carry: false }.execute_on(&mut chip, &mut harness).unwrap();
        assert_eq!(chip.register[6], 0x01, "Sum was {}", chip.register[6]);
        assert!(chip.a, "aux carry was {}", chip.a);
        assert!(chip.c, "carry was {}", chip.c);
        assert!(!chip.z, "zero bit was {}", chip.z);
        assert!(!chip.m, "sign bit was {}", chip.m);
        assert!(!chip.p, "parity bit was {}", chip.p);

        AddTo { value: 0x7E, carry: true }.execute_on(&mut chip, &mut harness).unwrap();
        assert_eq!(chip.register[6], 0x80, "Sum was {}", chip.register[6]);
        assert!(chip.a, "aux carry was {}", chip.a);
        assert!(!chip.c, "carry was {}", chip.c);
        assert!(!chip.z, "zero bit was {}", chip.z);
        assert!(chip.m, "sign bit was {}", chip.m);
        assert!(!chip.p, "parity bit was {}", chip.p);
     }

     #[test]
     fn and() {
        let mut env = Socket::default();
        let mut chip = State::new();
        chip[Register::A] = 0b01011101;
        AndWith { value: 0b11011011 }.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip.register[6], 0b01011001);
        assert!(!chip.a, "aux carry was {}", chip.a);
        assert!(!chip.c, "carry was {}", chip.c);
        assert!(!chip.z, "zero bit was {}", chip.z);
        assert!(!chip.m, "sign bit was {}", chip.m);
        assert!(chip.p, "parity bit was {}", chip.p);

        AndWith { value: 0b10100100 }.execute_on(&mut chip, &mut env).unwrap();
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
        AddTo { value: 0x3C, carry: false }.execute_on(&mut chip, &mut env).unwrap();
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
    fn xthl() {
        let mut env = SimpleBoard::default();
        let mut chip = State::new();
        chip.sp = 0x7BE3;
        chip[HL] = 0x3472;
        [env[0x7BE3], env[0x7BE4]] = [0x43, 0x29];
        ExchangeTopWithHilo.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip.sp, 0x7BE3);
        assert_eq!(chip[L], 0x43);
        assert_eq!(chip[H], 0x29);
        assert_eq!(env.read_word(chip.sp), 0x3472);
    }

    #[test]
    fn xor() {
        let mut env = Socket::default();
        let mut chip = State::new();
        chip[Register::A] = 0b10011100;
        *chip.update_flags() = true;
        ExclusiveOrWith { value: 0b00111110 }.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip[Register::A], 0b10100010);
        assert!(!chip.c, "Carry flag set");
        assert!(!chip.z, "Zero flag set");
        assert!(chip.m, "minus flag reset");
        assert!(!chip.p, "parity flag even");
    }

    #[test]
    fn compare_i() {
        let mut env = Socket::default();
        let mut chip = State::new();
        chip[Register::A]  = 0b01011011;
        CompareWith { value: 0b10100011 }.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip[Register::A], 0x5B);
        assert!(!chip.a, "Auxilliary carry flag set");
        assert!(chip.c, "Carry flag cleared");
        assert!(!chip.z, "Zero flag set");
        assert!(chip.m, "Sign flag cleared");
        assert!(chip.p, "Parity flag odd");
    }

    #[test]
    fn halt() {
        let mut env = Socket::default();
        let mut chip = State::new();
        chip.pc = 0x2534;
        chip.pc += Halt.len() as u16;
        Halt.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip.pc, 0x2535);
        assert!(!chip.active, "Processor not stopped");
    }

    #[test]
    fn jump() {
        let mut env = Socket::default();
        let mut chip = State::new();
        Jump{ to: 0x0340 }.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip.pc, 0x0340);
        chip.register[6] = 0x90;
        AddTo { value: 0x73, carry: false }.execute_on(&mut chip, &mut env).unwrap();
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
        LoadExtendedWith { to: Internal::Wide(HL), value: 0x6472 }.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip.register[4], 0x72);
        assert_eq!(chip.register[5], 0x64);
    }

    #[test]
    fn move_() {
        let mut env = SimpleBoard::default();
        let mut chip = State::new();
        chip[Register::A] = 0x05;
        chip[Register::H] = 0x02;
        chip[Register::L] = 0xA4;
        chip[Register::B] = 0x32;
        env[0x02A4] = 0xD4;
        env[0x0205] = 0xB2;
        Move{to: Byte::Single(Register::L), from: Byte::Single(Register::A)}.execute_on(&mut chip, &mut env).unwrap();
        Move{to: Byte::Single(Register::B), from: Byte::Indirect}.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip[Register::B], 0xB2);
    }

    #[test]
    fn or() {
        let mut env = Socket::default();
        let mut chip = State::new();
        chip[Register::A] = 0b01010110;
        *chip.update_flags() = true;
        OrWith{value: 0b00010101}.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip[Register::A], 0b01010111);
        assert!(!chip.c, "carry flag not reset");
        assert!(!chip.z, "zero flag set");
        assert!(!chip.m, "minus flag set");
        assert!(!chip.p, "parity flag even");
    }

    #[test]
    fn pop() {
        let mut env = SimpleBoard::default();
        let mut chip = State::new();
        chip[BC] = 0x8372;
        chip[DE] = 0x4928;
        chip[HL] = 0x5B6E;
        chip.sp = 0x0238;
        [env[0x0238], env[0x0239]] = [0xB6, 0x4E];
        Pop(Word::OnBoard(Internal::Wide(BC))).execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip[BC], 0x4EB6);
        assert_eq!(chip.sp, 0x023A);
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
        AddTo { value: 0x73, carry: false }.execute_on(&mut chip, &mut env).unwrap();
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
    fn return_from() {
        let mut env = SimpleBoard::default();
        let mut chip = State::new();
        chip.pc = 0x02B6;
        chip.sp = 0x8EA5;
        [env[0x8EA5], env[0x8EA6]] = [0xFE,0x01];
        Return.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip.sp, 0x8EA7);
        assert_eq!(chip.pc, 0x01FE);
    }

    #[test]
    fn rotate() {
        let mut env = Socket::default();
        let mut chip = State::new();
        chip[Register::A] = 0b0111_0101;
        RotateRightCarrying.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip[Register::A], 0b1011_1010);
        assert!(chip.c, "Carry bit cleared");
        RotateRightCarrying.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip[Register::A], 0b0101_1101);
        assert!(!chip.c, "Carry bit set");
    }

    #[test]
    fn subtract() {
        let mut env = Socket::default();
        let mut chip = State::new();
        chip[Register::A] = 0b1001_0011;
        SubtractBy{value: 0b1011_0110, carry: false}.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip[Register::A], 0b1101_1101);
        assert!(!chip.a, "Auxilliary carry flag set");
        assert!(chip.c, "Carry flag cleared");
        assert!(!chip.z, "Zero flag set");
        assert!(chip.m, "Sign flag cleared");
        assert!(chip.p, "Parity flag odd");
        SubtractBy { value: 0b1101_1101, carry: false }.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip[Register::A], 0b0000_0000);
        assert!(chip.a, "Auxilliary carry flag clear");
        assert!(!chip.c, "Carry flag set");
        assert!(chip.z, "Zero flag cleared");
        assert!(!chip.m, "Sign flag set");
        assert!(chip.p, "Parity flag odd");
        chip.c = true;
        SubtractBy { value: 0b0011_1100, carry: true }.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(chip[Register::A], 0b1100_0011);
        assert!(chip.c, "carry flag reset");
        assert!(!chip.z, "zero flag set");
        assert!(chip.m, "sign flag reset");
        assert!(chip.p, "parity flag odd");
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
        MoveData { value: 0x02, to: Byte::Single(H) }.execute_on(&mut chip, &mut env).unwrap();
        MoveData { value: 0x72, to: Byte::Indirect }.execute_on(&mut chip, &mut env).unwrap();
        assert_eq!(env[0x0221], 0x72);
    }
}