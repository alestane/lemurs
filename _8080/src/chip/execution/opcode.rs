use crate::{convert::TryFrom, chip::access::{Word, Double}};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    NOP(u8),
    AndImmediate{value: u8},
    Call{sub: u16},
    Jump{to: u16},
    JumpIf(Test, u16),
    LoadExtendedImmediate{to: Word, value: u16 },
    Reset{vector: u8},
    ReturnIf(Test),
}
use Op::*;

impl From<u8> for Word {
    fn from(value: u8) -> Self {
        match value & 0b00_11_0000 {
            0b00_00_0000 => Word::Wide(Double::BC),
            0b00_01_0000 => Word::Wide(Double::DE),
            0b00_10_0000 => Word::Wide(Double::HL),
            0b00_11_0000 => Word::SP,
            _ => unreachable!(),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Flag {
    Zero,
    Carry,
    Parity,
    Sign,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Test {
    Not(Flag),
    Is(Flag),
}

use Flag::*;
use Test::*;

impl Test {
    pub fn approves(self, env: &super::State) -> bool {
        match self {
            Not(Zero) => !env.z,
            Is(Zero) => env.z,
            Not(Carry) => !env.c,
            Is(Carry) => env.c,
            Not(Parity) => !env.p,
            Is(Parity) => env.p,
            Not(Sign) => !env.m,
            Is(Sign) => env.m,
        }
    }
}

impl From<u8> for Test {
fn from(value: u8) -> Self {
        let test = match (value & 0b00_11_0_000) >> 4 {
            0b00 => Zero,
            0b01 => Carry,
            0b10 => Parity,
            0b11 => Sign,
            _ => unreachable!()
        };
        match (value & 0b00_00_1_000) >> 3  {
            0b0 => Not(test),
            0b1 => Is(test),
            _ => unreachable!()
        }
    }
}

#[disclose]
#[allow(non_upper_case_globals)]
mod b11111111 {
    const NoOp: u8 = 0b00000000;
    const RotateLeftCarrying: u8        = 0b00000111;
    const RotateRightCarrying: u8       = 0b00001111;
    const RotateAccumulatorLeft: u8     = 0b00010111;
    const RotateAccumulatorRight: u8    = 0b00011111;

    const DecimalAddAdjust: u8      = 0b00100111;
    const ComplementAccumulator: u8 = 0b00101111;

    const SetCarry: u8          = 0b00110111;
    const ComplementCarry: u8   = 0b00111111;

    const Halt: u8      = 0b01110110;
    const Return: u8    = 0b11001001;

    const Output: u8    = 0b11010011;
    const Input: u8     = 0b11011011;

    const ExchangeTopWithHilo: u8       = 0b11100011;
    const ProgramCounterFromHilo: u8    = 0b11101001;
    const ExchangeDoubleWithHilo: u8    = 0b11101011;
    const StackPointerFromHilo: u8      = 0b11111001;

    const AndImmediate: u8  = 0b11100110;

    const StoreHiLoDirect: u8   = 0b00100010;
    const Jump: u8  = 0b11000011;
}

#[disclose]
#[allow(non_upper_case_globals)]
mod b11_00_1111 {
    const LoadExtendedImmediate: u8 = 0x01;
}

#[disclose]
#[allow(non_upper_case_globals)]
mod b11_000_111 {
    const JumpIf: u8 = 0b11_000_010;
    const Reset: u8 = 0b11_000_111;
    const ReturnIf: u8 = 0b11_000_000;
    const CallIf: u8 = 0b11_000_100;
}

pub struct OutOfRange;
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
    NotUsable(Op),
    Mismatch(Op, u8),
    Invalid([u8;1]),
    InvalidPair([u8;2]),
    InvalidTriple([u8;3]),
    NoData,
}

mod build {
    pub type Result = crate::Result<(super::Op, usize), super::Error>;
}

impl TryFrom<[u8;1]> for Op {
    type Error = [u8;1];
    fn try_from(value: [u8;1]) -> Result<Self, Self::Error> {
        {
            let value = value[0];
            let value = match value & 0b11111111 {
                b11111111::NoOp => return Ok(NOP(4)),
                _ => value
            };
            let _value = match value & 0b11_000_111 {
                b11_000_111::Reset => return Ok(Reset{vector: value >> 3 & 0x07}),
                b11_000_111::ReturnIf => return Ok(ReturnIf(Test::from(value))),
                _ => value,
            };
        }
        Err(value)
    }
}

impl TryFrom<[u8;2]> for Op {
    type Error = [u8;2];
    fn try_from(value: [u8;2]) -> Result<Self, Self::Error> {
        let [action, data] = value;
        let _action = match action {
            b11111111::AndImmediate => return Ok(AndImmediate { value: data }),
            next => next,
        };
        Err(value)
    }
}

impl TryFrom<[u8;3]> for Op {
    type Error = [u8;3];
    fn try_from(value: [u8;3]) -> Result<Self, Self::Error> {
        let action = value[0];
        let data = u16::from_le_bytes([value[1], value[2]]);
        match action {
            b11111111::Jump => return Ok(Jump{to: data}),
            _ => action,
        };
        match action & 0b11_00_1111 {
            b11_00_1111::LoadExtendedImmediate => return Ok(LoadExtendedImmediate { to: Word::from(action), value: data }),
            _ => action,
        };
        match action & 0b11_000_111 {
            b11_000_111::JumpIf => return Ok(JumpIf(Test::from(action), data)),
            _ => action,
        };
        Err(value)
    }
}

impl Op {
    pub fn len(&self) -> u8 {
        match self {
            Call{..} => 3,
            _ => 1,
        }
    }

    pub fn extract(value: &[u8]) -> build::Result {
        let mut bytes = value.iter().copied();
        let code = match Op::try_from([bytes.next().ok_or(Error::NoData)?]) {
            Ok(op) => return Ok((op, 1)),
            Err(code) => code,
        };
        let code = match Op::try_from([code[0], bytes.next().ok_or(Error::Invalid(code))?]) {
            Ok(op) => return Ok((op, 2)),
            Err(code) => code,
        };
        match Op::try_from([code[0], code[1], bytes.next().ok_or(Error::InvalidPair(code))?]) {
            Ok(op) => Ok((op, 3)),
            Err(code) => Err(Error::InvalidTriple(code))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn no_op() {
        let op = Op::extract(&[0x00]).unwrap();
        assert_eq!(op.0, NOP(4));
    }

    #[test]
    fn reset_from_val() {
        let op = Op::extract(&[0xD7]).unwrap();
        assert_eq!(op.0, Reset{vector: 2});
    }

    #[test]
    fn return_if() {
        let op = Op::extract(&[0xD8]).unwrap();
        assert_eq!(op.0, ReturnIf(Is(Carry)));
        let op = Op::extract(&[0xF0]).unwrap();
        assert_eq!(op.0, ReturnIf(Not(Sign)))
    }

    #[test]
    fn load_xi() {
        let op = Op::extract(&[0x31, 0x25, 0x02]).unwrap();
        assert_eq!(op.0, LoadExtendedImmediate { to: Word::SP, value: 549 });
        let fail = Op::extract(&[0x11, 0x21]).unwrap_err();
        assert_eq!(fail, Error::InvalidPair([0x11, 0x21]));
    }
}