use core::ops::DerefMut;
use crate::{num::NonZeroU8, String, Machine, Harness};
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

fn subtract(base: u8, by: u8) -> (u8, bool, bool) {
    let value = (!by).wrapping_add(1);
    let aux = base ^ value;
    let (value, carry) = base.overflowing_add(value);
    (value, by != 0 && !carry, (value ^ aux) & 0x10 != 0)
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
                let (value, carry) = accumulator.overflowing_add(value.wrapping_add(carry_in as u8));
                *accumulator = value;
                *chip.update_flags() = carry;
                chip.a = (value ^ aux) & 0x10 != 0;
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
                let aux = if chip[A]& 0x0F > 0x09 {
                    chip[A] += 0x06;
                    true
                } else {
                    if chip.a { chip[A] = chip[A].wrapping_add(6); }
                    false
                };
                let carry = if chip[A] >> 4 > 0x09 {
                    chip[A] = chip[A].wrapping_add(0x06 << 4);
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
                    Single(reg) => { chip[reg] = chip[reg].wrapping_sub(1); (chip[reg], 5)}
                    Byte::RAM(address) => { 
                        let value = bus.read(address).wrapping_sub(1); 
                        bus.write(value, address);
                        (value, 10)
                    }
                    _ => unreachable!()
                };
                *chip.update_flags_for(value) = false;
                chip.a = (value ^ value.wrapping_add(1)) & 0x10 != 0;
                time
            }
            DecrementWord{register} => {
                chip[register] = chip[register].wrapping_sub(1);
                5
            }
            DoubleAdd { register } => {
                (chip[HL], chip.c) = chip[HL].overflowing_add(chip[register]);
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
                    Single(reg) => { chip[reg] = chip[reg].wrapping_add(1); (chip[reg], 5)}
                    Byte::RAM(address) => { 
                        let value = bus.read(address).wrapping_add(1); 
                        bus.write(value, address);
                        (value, 10)
                    }
                    _ => unreachable!()
                };
                *chip.update_flags_for(value) = false;
                chip.a = (value ^ value.wrapping_sub(1)) & 0x10 != 0;
                time
            }
            IncrementWord { register } => {
                chip[register] = chip[register].wrapping_add(1);
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
                        let [accumulator, status] = bus.read_word(chip.pop()).to_le_bytes();
                        chip[A] = accumulator;
                        chip.extract_flags(status);
                    }
                    _ => unreachable!()
                };
                10
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
                let accumulator = chip[A];
                chip.c = accumulator & 0x01 != 0;
                chip[A] = accumulator.rotate_right(1);
                4
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
                let (value, carry, aux) = subtract(chip[A], value.wrapping_add((chip.c && carry) as u8));
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