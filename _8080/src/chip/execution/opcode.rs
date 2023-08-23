use crate::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    NOP(u8),
    Call{sub: u16},
    Reset{vector: u8},
}
use Op::*;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum Flag {
    Zero,
    Carry,
    Parity,
    Sign,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum Test {
    Not(Flag),
    Is(Flag),
}

use Flag::*;
use Test::*;

#[allow(non_upper_case_globals)]
mod b11111111 {
    pub const RotateLeftCarrying: u8        = 0b00000111;
    pub const RotateRightCarrying: u8       = 0b00001111;
    pub const RotateAccumulatorLeft: u8     = 0b00010111;
    pub const RotateAccumulatorRight: u8    = 0b00011111;

    pub const DecimalAddAdjust: u8      = 0b00100111;
    pub const ComplementAccumulator: u8 = 0b00101111;

    pub const SetCarry: u8          = 0b00110111;
    pub const ComplementCarry: u8   = 0b00111111;

    pub const Halt: u8      = 0b01110110;
    pub const Return: u8    = 0b11001001;

    pub const Output: u8    = 0b11010011;
    pub const Input: u8     = 0b11011011;

    pub const ExchangeTopWithHilo: u8       = 0b11100011;
    pub const ProgramCounterFromHilo: u8    = 0b11101001;
    pub const ExchangeDoubleWithHilo: u8    = 0b11101011;
    pub const StackPointerFromHilo: u8      = 0b11111001;


}

#[allow(non_upper_case_globals)]
mod b11_000_111 {
    pub const Reset: u8 = 0b11_000_111;
    pub const ReturnIf: u8 = 0b11_000_000;
    pub const CallIf: u8 = 0b11_000_100;
}

pub struct OutOfRange;
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
            let _value = match value & 0b11_000_111 {
                b11_000_111::Reset => return Ok(Reset{vector: value >> 3 & 0x07}),
                _ => value,
            };
        }
        Err(value)
    }
}

impl TryFrom<[u8;2]> for Op {
    type Error = [u8;2];
    fn try_from(value: [u8;2]) -> Result<Self, Self::Error> {
        Err(value)
    }
}

impl TryFrom<[u8;3]> for Op {
    type Error = [u8;3];
    fn try_from(value: [u8;3]) -> Result<Self, Self::Error> {
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