use core::ops::DerefMut;
use crate::{raw, bits::*, num::{NonZeroU8, Wrapping}, String, Machine, Harness};
use super::{State, access::{*, Register::*, Byte::*, Double::*, Internal::*, Word::*}};

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
        self.chip.pc += Wrapping(len as raw::u16); 
        let outcome = op.execute_on(&mut self.chip, self.board.deref_mut());
        match outcome {
            Ok(Some(_)) => (),
            _ => self.chip.active = false,
        };
        if let Some(action) = self.board.did_execute(&self.chip, op)? {
            action.execute_on(&mut self.chip, self.board.deref_mut()).unwrap();
            if action == Halt { return Ok(None); }
        }
        outcome
	}

    #[cfg(not(debug_assertions))]
	pub fn execute(&mut self) -> OpOutcome {
		if !self.active { return NonZeroU8::new(1) };
        let (op, len) = Op::extract(self.board.deref(), self.pc)
            .map_err(|e| panic!("Couldn't extract opcode from {e:X?}")).unwrap();
        self.pc += Wrapping(len as raw::u16); 
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
            0..8 => Ok(self.interrupt(Reset{vector: index as raw::u8}).ok().unwrap()),
            _ => Err(opcode::OutOfRange)
        }
    }
    }

fn subtract(base: u8, by: u8) -> (u8, bool, bool) {
    let value = (!by) + Wrapping(1);
    let aux = base ^ value;
    let (value, carry) = base.0.overflowing_add(value.0);
    (Wrapping(value), by.0 != 0 && !carry, (value ^ aux.0) & 0x10 != 0)
}

macro_rules! byte {
    {$chip:expr, $from:ident, $bus:expr, $onboard: expr, $external: expr} => {
        match $chip.resolve_byte($from) {
            Single(register) => ($chip[register], $onboard),
            Byte::RAM(address) => ($bus.read(address), $external),
            _ => unreachable!()
        }
    };
}

impl Op {
    fn execute_on<H: Harness>(self, chip: &mut State, mut bus: impl DerefMut<Target = H>) -> OpOutcome {
        let cycles = match self {
            Add { from, carry } => {
                let (value, time) = byte!{chip, from, bus, 4, 7};
                AddTo{value, carry}.execute_on(chip, bus)?;
                time
            }
            AddTo { value, carry } => {
                let carry_in = chip.c && carry;
                let accumulator = &mut chip[A];
                let aux = *accumulator ^ value;
                let (value, carry) = accumulator.0.overflowing_add(value.0.wrapping_add(carry_in as raw::u8));
                let value = Wrapping(value);
                *accumulator = value;
                *chip.update_flags() = carry;
                chip.a = (value ^ aux).0 & 0x10 != 0;
                7
            }
            And{from} => {
                let (value, time) = byte!{chip, from, bus, 4, 7};
                AndWith{value}.execute_on(chip, bus)?;
                time
            }
            AndWith { value } => {
                chip[A] &= value;
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
            CarryFlag(set) => {
                chip.c = set || !chip.c;
                4
            }
            Compare{from} => {
                let (value, time) = byte!{chip, from, bus, 4, 7};
                CompareWith { value }.execute_on(chip, bus)?;
                time
            }
            CompareWith{value} => {
                let (value, carry, aux) = subtract(chip[A], value);
                *chip.update_flags_for(value) = carry;
                chip.a = aux;
                7
            }
            ComplementAccumulator => {
                chip[A] = !chip[A];
                4
            }
            DecimalAddAdjust => {
                let aux = if chip[A].0  & 0x0F > 0x09 {
                    chip[A] += 0x06;
                    true
                } else {
                    if chip.a { chip[A] = chip[A] + Wrapping(6); }
                    false
                };
                let carry = if chip[A] >> 4 > Wrapping(0x09) {
                    chip[A] += 0x06 << 4;
                    true
                } else {
                    if chip.c { chip[A] += 0x06 << 4; }
                    false
                };
                *chip.update_flags() = carry;
                chip.a = aux;
                4
            }
            DecrementByte { register } => {
                let (value, time) = match chip.resolve_byte(register) {
                    Single(reg) => { chip[reg] -= 1; (chip[reg], 5)}
                    Byte::RAM(address) => { 
                        let value = bus.read(address) - Wrapping(1); 
                        bus.write(value, address);
                        (value, 10)
                    }
                    _ => unreachable!()
                };
                *chip.update_flags_for(value) = false;
                chip.a = (value ^ (value + Wrapping(1))).0 & 0x10 != 0;
                time
            }
            DecrementWord{register} => {
                chip[register] -= 1;
                5
            }
            DoubleAdd { register } => {
                let (value, carry) = chip[HL].0.overflowing_add(chip[register].0);
                (chip[HL], chip.c) = (Wrapping(value), carry);
                10
            }
            ExchangeDoubleWithHilo => {
                (chip[DE], chip[HL]) = (chip[HL], chip[DE]);
                5
            }
            ExchangeTopWithHilo => {
                let out = chip[HL];
                chip[HL] = bus.read_word(chip.sp);
                bus.write_word(out, chip.sp);
                18
            }
            ExclusiveOr { from } => {
                let (value, time) = byte!(chip, from, bus, 4, 7);
                ExclusiveOrWith{value}.execute_on(chip, bus)?;
                time
            }
            ExclusiveOrWith { value } => {
                chip[A] ^= value;
                *chip.update_flags() = false;
                7
            }
            Halt => {
                chip.active = false;
                7
            }
            IncrementByte { register } => {
                let (value, time) = match chip.resolve_byte(register) {
                    Single(reg) => { chip[reg] += 1; (chip[reg], 5)}
                    Byte::RAM(address) => { 
                        let value = bus.read(address) + Wrapping(1); 
                        bus.write(value, address);
                        (value, 10)
                    }
                    _ => unreachable!()
                };
                *chip.update_flags_for(value) = false;
                chip.a = (value ^ (value - Wrapping(1))).0 & 0x10 != 0;
                time
            }
            IncrementWord { register } => {
                chip[register] += 1;
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
            LoadAccumulator{address} => {
                chip[A] = bus.read(address);
                13
            }
            LoadAccumulatorIndirect { register } => {
                chip[A] = bus.read(chip[register]);
                7
            }
            LoadExtendedWith { to, value } => {
                chip[to] = value;
                10
            }
            LoadHilo{address} => {
                chip[HL] = bus.read_word(address);
                16
            }
            Move{to, from} => {
                let (to, from) = (chip.resolve_byte(to), chip.resolve_byte(from));
                match (to, from) {
                    (Single(to), Single(from)) => {
                        chip[to] = chip[from];
                        5
                    }
                    (Byte::RAM(address), Single(from)) => {
                        bus.write(chip[from], address);
                        7
                    }
                    (Single(to), Byte::RAM(address)) => {
                        chip[to] = bus.read(address);
                        7
                    }
                    _ => unreachable!()
                }
            }
            MoveData { value, to } => {
                match chip.resolve_byte(to) {
                    Single(register) => { chip[register] = value; 7 },
                    Byte::RAM(address) => { bus.write(value, address); 10},
                    _ => unreachable!()
                }
            }
            Or{from} => {
                let (value, time) = byte!{chip, from, bus, 4, 7};
                OrWith{value}.execute_on(chip, bus)?;
                time
            }
            OrWith{value} => {
                chip[A] |= value;
                *chip.update_flags() = false;
                7
            }
            Pop(target) => {
                match target {
                    OnBoard(internal) => chip[internal] = bus.read_word(chip.pop()),
                    ProgramStatus => {
                        let [accumulator, status] = bus.read_word(chip.pop()).0.to_le_bytes();
                        chip[A] = Wrapping(accumulator);
                        chip.extract_flags(status);
                    }
                    _ => unreachable!()
                };
                10
            }
            ProgramCounterFromHilo => {
                chip[ProgramCounter] = chip[HL];
                5
            }
            Push (source) => {
                let source = match source {
                    OnBoard(internal) => chip[internal],
                    ProgramStatus => chip.status(),
                    _ => unreachable!()
                };
                bus.write_word(source, chip.push());
                11
            }
            Reset{vector} => {
                bus.write_word(chip.pc, chip.push());
                chip.pc = Wrapping(vector as raw::u16 * 8);
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
            RotateAccumulatorLeft => {
                let bits = chip[A].0 as raw::u16 | if chip.c { 0x8000 } else { 0x0000 };
                let [bits, carry] = bits.rotate_left(1).to_le_bytes();
                chip.c = carry != 0;
                chip[A] = Wrapping(bits);
                4
            }
            RotateAccumulatorRight => {
                let bits = chip[A].0 as raw::u16 | if chip.c { 0x0100 } else { 0x0000 };
                let [bits, carry] = bits.rotate_right(1).to_le_bytes();
                chip.c = carry != 0;
                chip[A] = Wrapping(bits);
                4
            }
            RotateLeftCarrying => {
                let accumulator = chip[A].0;
                chip.c = accumulator & 0x80 != 0;
                chip[A] = Wrapping(accumulator.rotate_left(1));
                4
            }
            RotateRightCarrying => {
                let accumulator = chip[A].0;
                chip.c = accumulator & 0x01 != 0;
                chip[A] = Wrapping(accumulator.rotate_right(1));
                4
            }
            StackPointerFromHilo => {
                chip[StackPointer] = chip[HL];
                5
            }
            StoreAccumulator { address } => {
                bus.write(chip[A], address);
                13
            }
            StoreAccumulatorIndirect { register } => {
                bus.write(chip[A], chip[register]);
                7
            }
            StoreHilo{ address } => {
                bus.write_word(chip[HL], address);
                16
            }
            Subtract { from, carry } => {
                let (value, time) = byte!{chip, from, bus, 4, 7};
                SubtractBy{value, carry}.execute_on(chip, bus)?;
                time
            }
            SubtractBy{ value, carry } => {
                let (value, carry, aux) = subtract(chip[A], value + Wrapping((chip.c && carry) as raw::u8));
                chip[A] = value;
                *chip.update_flags() = carry;
                chip.a = aux;
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
mod tests;