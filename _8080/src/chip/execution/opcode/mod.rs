use core::fmt::UpperHex;

use crate::{bits, num::Wrapping, convert::TryFrom, chip::access::{*, Byte::*, Register::*, Word::*, Double::*, Internal::*}};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    NOP(u8),
    Add{from: Byte, carry: bool},
    AddTo{value: bits::u8, carry: bool},
    And{from: Byte},
    AndWith{value: bits::u8},
    Call{sub: bits::u16},
    CallIf(Test, bits::u16),
    CarryFlag(bool),
    Compare{from: Byte},
    CompareWith{value: bits::u8},
    ComplementAccumulator,
    DecimalAddAdjust,
    DecrementByte{register: Byte},
    DecrementWord{register: Internal},
    Interrupts(bool),
    DoubleAdd{register: Internal},
    ExchangeDoubleWithHilo, 
    ExchangeTopWithHilo,
    ExclusiveOr{ from: Byte },
    ExclusiveOrWith{value: bits::u8},
    Halt,
    In(u8),
    IncrementByte{register: Byte},
    IncrementWord{register: Internal},
    Jump{to: bits::u16},
    JumpIf(Test, bits::u16),
    LoadAccumulator{address: bits::u16},
    LoadAccumulatorIndirect{register: Double},
    LoadExtendedWith{to: Internal, value: bits::u16 },
    LoadHilo{address: bits::u16},
    Move{to: Byte, from: Byte},
    MoveData{value: bits::u8, to: Byte},
    Or{from: Byte},
    OrWith{value: bits::u8},
    Out(u8),
    Pop(Word),
    ProgramCounterFromHilo,
    Push(Word),
    Reset{vector: u8},
    Return,
    ReturnIf(Test),
    RotateLeftCarrying,
    RotateRightCarrying,
    RotateAccumulatorLeft,
    RotateAccumulatorRight,
    StackPointerFromHilo,
    StoreAccumulator{address: bits::u16},
    StoreAccumulatorIndirect{register: Double},
    StoreHilo{address: bits::u16},
    Subtract{from: Byte, carry: bool},
    SubtractBy{value: bits::u8, carry: bool},
}
use Op::*;

impl From<u8> for Internal {
    fn from(value: u8) -> Self {
        match value & 0b00_11_0000 {
            0b00_00_0000 => Wide(BC),
            0b00_01_0000 => Wide(DE),
            0b00_10_0000 => Wide(HL),
            0b00_11_0000 => StackPointer,
            _ => unreachable!(),
        }
    }
}

impl From<u8> for Byte {
    fn from(value: u8) -> Self {
        match value & 0b00_111_000 {
            0b00_000_000 => Single(B),
            0b00_001_000 => Single(C),
            0b00_010_000 => Single(D),
            0b00_011_000 => Single(E),
            0b00_100_000 => Single(H),
            0b00_101_000 => Single(L),
            0b00_110_000 => Byte::Indirect,
            0b00_111_000 => Single(A),
            _ => unreachable!(),
        }
    }
}

impl Byte {
    fn split(value: u8) -> (Self, Self) {
        (Self::from(value), Self::from(value << 3))
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Flag {
    Zero,
    Carry,
    EvenParity,
    Negative,
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
            Not(EvenParity) => !env.p,
            Is(EvenParity) => env.p,
            Not(Negative) => !env.m,
            Is(Negative) => env.m,
        }
    }
}

impl From<u8> for Test {
    fn from(value: u8) -> Self {
        let test = match (value & 0b00_11_0_000) >> 4 {
            0b00 => Zero,
            0b01 => Carry,
            0b10 => EvenParity,
            0b11 => Negative,
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

    const StoreAccumulatorDirect: u8    = 0b00110010;
    const LoadAccumulatorDirect: u8 = 0b00111010;

    const StoreHiloDirect: u8   = 0b00100010;
    const LoadHiloDirect: u8    = 0b00101010;

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

    const DisableInterrupts: u8 = 0b11110011;
    const EnableInterrupts: u8  = 0b11111011;

    const AndImmediate: u8  = 0b11100110;
    const AddImmediate: u8  = 0b11000110;
    const AddImmediateCarrying: u8  = 0b11001110;
    const SubtractImmediate: u8     = 0b11010110;
    const SubtractImmediateBorrowing: u8    = 0b11011110;
    const ExclusiveOr: u8   = 0b11101110;
    const OrImmediate: u8   = 0b11110110;
    const CompareImmediate: u8   = 0b11111110;

    const StoreHiLoDirect: u8   = 0b00100010;
    const Jump: u8  = 0b11000011;
    const Call: u8  = 0b11001101;
}

#[disclose]
#[allow(non_upper_case_globals)]
mod b11_00_1111 {
    const LoadExtendedImmediate: u8 = 0b00_00_0001;
    const IncrementExtended: u8 = 0b00_00_0011;
    const DecrementExtended: u8 = 0b00_00_1011;
    const DoubleAdd: u8 = 0b00_00_1001;
    const Push: u8  = 0b11_00_0101;
    const Pop: u8 = 0b11_00_0001;
}

#[disclose]
#[allow(non_upper_case_globals)]
mod b11_000_111 {
    const IncrementRegister: u8 = 0b00_000_100;
    const DecrementRegister: u8 = 0b00_000_101;
    const JumpIf: u8 = 0b11_000_010;
    const Reset: u8 = 0b11_000_111;
    const ReturnIf: u8 = 0b11_000_000;
    const CallIf: u8 = 0b11_000_100;
    const MoveImmediate: u8 = 0b00_000_110;
}

#[disclose]
#[allow(non_upper_case_globals)]
mod b11_111_000 {
    const AddToAccumulator: u8  = 0b10_000_000;
    const AddCarryingToAccumulator : u8 = 0b10_001_000;
    const SubtractFromAccumulator: u8   = 0b10_010_000;
    const SubtractBorrowingFromAccumulator: u8  = 0b10_011_000;
    const AndWithAccumulator: u8    = 0b10_100_000;
    const ExclusiveOrWithAccumulator: u8    = 0b10_101_000;
    const OrWithAccumulator: u8 = 0b10_110_000;
    const CompareWithAccumulator: u8    = 0b10_111_000;
}

#[disclose]
#[allow(non_upper_case_globals)]
mod b11_000000 {
    const Move: u8  = 0b01_000000;
}

#[disclose]
#[allow(non_upper_case_globals)]
mod b111_0_1111 {
    const LoadAccumulatorIndirect: u8   = 0b000_0_1010;
    const StoreAccumulatorIndirect: u8  = 0b000_0_0010;
}

pub struct OutOfRange;
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
    Unknown(u8),
    NotUsable(Op),
    Mismatch(Op, u8),
    Invalid([u8;1]),
    InvalidPair([u8;2]),
    InvalidTriple([u8;3]),
    NoData,
}

impl UpperHex for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Mismatch(op, code) => write!(f, "Mismatch({op:?}, {code:X})"),
            Self::Invalid([a]) => write!(f, "Invalid([{a:#04X}])"),
            Self::InvalidPair([a, b]) => write!(f, "InvalidPair([{a:#04X}, {b:#04X}])"),
            Self::InvalidTriple([a, b, c]) => write!(f, "InvalidTriple([{a:#04X}, {b:#04X}, {c:#04X}])"),
            _ => write!(f, "{self:?}"),
        }
    }
}

impl TryFrom<[u8;1]> for Op {
    type Error = [u8;1];
    fn try_from(value: [u8;1]) -> Result<Self, Self::Error> {
        {
            let value = value[0];
            let value = match value & 0b11111111 {
                b11111111::NoOp => return Ok(NOP(4)),
                b11111111::ExchangeDoubleWithHilo => return Ok(ExchangeDoubleWithHilo),
                b11111111::Halt => return Ok(Halt),
                b11111111::Return => return Ok(Return),
                b11111111::ExchangeTopWithHilo => return Ok(ExchangeTopWithHilo),
                b11111111::RotateLeftCarrying => return Ok(RotateLeftCarrying),
                b11111111::RotateRightCarrying => return Ok(RotateRightCarrying),
                b11111111::RotateAccumulatorLeft => return Ok(RotateAccumulatorLeft),
                b11111111::RotateAccumulatorRight => return Ok(RotateAccumulatorRight),
                b11111111::SetCarry => return Ok(CarryFlag(true)),
                b11111111::ComplementCarry => return Ok(CarryFlag(false)),
                b11111111::DecimalAddAdjust => return  Ok(DecimalAddAdjust),
                b11111111::ComplementAccumulator => return Ok(ComplementAccumulator),
                b11111111::ProgramCounterFromHilo => return Ok(ProgramCounterFromHilo),
                b11111111::StackPointerFromHilo => return Ok(StackPointerFromHilo),
                b11111111::DisableInterrupts => return Ok(Interrupts(false)),
                b11111111::EnableInterrupts => return Ok(Interrupts(true)),
                _ => value
            };
            let _value = match value & 0b11_000_111 {
                b11_000_111::Reset => return Ok(Reset{vector: value >> 3 & 0x07}),
                b11_000_111::ReturnIf => return Ok(ReturnIf(Test::from(value))),
                b11_000_111::IncrementRegister => return Ok(IncrementByte { register: Byte::from(value) }),
                b11_000_111::DecrementRegister => return Ok(DecrementByte { register: Byte::from(value) }),
                _ => value,
            };
            let _value = match value & 0b11_00_1111 {
                b11_00_1111::DecrementExtended => return Ok(DecrementWord{register: Internal::from(value)}),
                b11_00_1111::IncrementExtended => return Ok(IncrementWord { register: Internal::from(value) }),
                b11_00_1111::DoubleAdd => return Ok(DoubleAdd{register: Internal::from(value)}),
                b11_00_1111::Push => return Ok(Push(match Internal::from(value) { StackPointer => ProgramStatus, wide => OnBoard(wide)})),
                b11_00_1111::Pop => return Ok(Pop(match Internal::from(value) { StackPointer => ProgramStatus, wide => OnBoard(wide)})),
                _ => value,
            };
            let _value = match (value & 0b11_111_000, value << 3) {
                (b11_111_000::AddToAccumulator, value) => return Ok(Add{from: Byte::from(value), carry: false}),
                (b11_111_000::AddCarryingToAccumulator, value) => return Ok(Add{ from: Byte::from(value), carry: true}),
                (b11_111_000::SubtractFromAccumulator, value) => return Ok(Subtract{ from: Byte::from(value), carry: false}),
                (b11_111_000::SubtractBorrowingFromAccumulator, value) => return Ok(Subtract{ from: Byte::from(value), carry: true}),
                (b11_111_000::AndWithAccumulator, value) => return Ok(And{from: Byte::from(value)}),
                (b11_111_000::ExclusiveOrWithAccumulator, value) => return Ok(ExclusiveOr { from: Byte::from(value) }),
                (b11_111_000::OrWithAccumulator, value) => return Ok(Or { from: Byte::from(value) }),
                (b11_111_000::CompareWithAccumulator, value) => return Ok(Compare{from: Byte::from(value)}),
                _ => value,
            };
            let _value = match value & 0b11_000000 {
                b11_000000::Move => {
                    let (to, from) = Byte::split(value);
                    return Ok(Move{to, from});
                }
                _ => value,
            };
            let _value = match value & 0b111_0_1111 {
                b111_0_1111::LoadAccumulatorIndirect => return Ok(LoadAccumulatorIndirect { 
                    register: if value & 0b000_1_0000 != 0 { DE } else { BC }
                }),
                b111_0_1111::StoreAccumulatorIndirect => return Ok(StoreAccumulatorIndirect { 
                    register: if value & 0b000_1_0000 != 0 { DE } else { BC }
                }),
                _ => value,
            };
        }
        Err(value)
    }
}

impl TryFrom<[u8;2]> for Op {
    type Error = [u8;2];
    fn try_from(code: [u8;2]) -> Result<Self, Self::Error> {
        let [action, value] = code;
        let value = Wrapping(value);
        let action = match action {
            b11111111::AddImmediate => return Ok(AddTo { value, carry: false }),
            b11111111::AddImmediateCarrying => return Ok(AddTo{ value, carry: true }),
            b11111111::SubtractImmediate => return Ok(SubtractBy{ value, carry: false }),
            b11111111::SubtractImmediateBorrowing => return Ok(SubtractBy { value, carry: true }),
            b11111111::AndImmediate => return Ok(AndWith { value }),
            b11111111::ExclusiveOr => return Ok(ExclusiveOrWith{value}),
            b11111111::OrImmediate => return Ok(OrWith{value}),
            b11111111::CompareImmediate => return Ok(CompareWith{ value }),
            b11111111::Output => return Ok(Out(code[1])),
            b11111111::Input => return Ok(In(code[1])),
            _next => action,
        };
        let _action = match action & 0b11_000_111 {
            b11_000_111::MoveImmediate => return Ok(MoveData{ value, to: Byte::from(action) }),
            _next => action,
        };
        Err(code)
    }
}

impl TryFrom<[u8;3]> for Op {
    type Error = [u8;3];
    fn try_from(value: [u8;3]) -> Result<Self, Self::Error> {
        let action = value[0];
        let data = Wrapping(u16::from_le_bytes([value[1], value[2]]));
        match action {
            b11111111::LoadHiloDirect => return Ok(LoadHilo{address: data}),
            b11111111::StoreHiloDirect => return Ok(StoreHilo{address: data}),
            b11111111::LoadAccumulatorDirect => return Ok(LoadAccumulator { address: data }),
            b11111111::StoreAccumulatorDirect => return Ok(StoreAccumulator { address: data }),
            b11111111::Jump => return Ok(Jump{to: data}),
            b11111111::Call => return Ok(Call{sub: data}),
            _ => action,
        };
        match action & 0b11_00_1111 {
            b11_00_1111::LoadExtendedImmediate => return Ok(LoadExtendedWith { to: Internal::from(action), value: data }),
            _ => action,
        };
        match action & 0b11_000_111 {
            b11_000_111::JumpIf => return Ok(JumpIf(Test::from(action), data)),
            b11_000_111::CallIf => return Ok(CallIf(Test::from(action), data)),
            _ => action,
        };
        Err(value)
    }
}

impl Op {
    pub fn len(&self) -> u8 {
        match self {
            Call{..} | CallIf(..) | Jump{..} | JumpIf(..) | LoadExtendedWith{..} | 
            ReturnIf(..) | StoreAccumulator{..} | LoadAccumulator {..} | LoadHilo{..} | StoreHilo {..}
                => 3,
            AddTo{..} | AndWith{..} | ExclusiveOrWith{..} | OrWith{..} | SubtractBy{..} | CompareWith{..} | MoveData{..} |
            Out(..) | In(..)
                => 2,
            NOP(..) | Push(..) | Reset{..} | ExchangeDoubleWithHilo | Return | Halt | Pop(..) | ExchangeTopWithHilo | 
            Move{..} | RotateLeftCarrying | RotateRightCarrying | RotateAccumulatorLeft | RotateAccumulatorRight | 
            IncrementByte {..} | DecrementByte {..} | Add{..}  | Subtract{..} | And{..} | ExclusiveOr{..} | Or{..} | 
            Compare{..} | IncrementWord{..} | DecrementWord {..} | Interrupts(..) | 
            LoadAccumulatorIndirect {..} | StoreAccumulatorIndirect{..} | 
            DoubleAdd{..} | CarryFlag(..) | DecimalAddAdjust | ComplementAccumulator | ProgramCounterFromHilo | StackPointerFromHilo
                => 1,
        }
    }

    pub fn extract<H: crate::Harness>(bus: &H, start: bits::u16) -> Result<(Op, usize), self::Error> {
        let code = match Op::try_from([bus.read(start).0]) {
            Ok(op) => return Ok((op, 1)),
            Err(code) => code,
        };
        match code[0] {
            0xCB | 0xD9 => return Err(Error::Unknown(code[0])),
            0xDD | 0xED | 0xFD => return Err(Error::Unknown(code[0])),
            nop if nop & 0b00_111_0000 == 0 => return Err(Error::Unknown(nop)),
            _ => ()
        };
        let code = match Op::try_from([code[0], bus.read(start + Wrapping(1)).0]) {
            Ok(op) => return Ok((op, 2)),
            Err(code) => code,
        };
        match Op::try_from([code[0], code[1], bus.read(start + Wrapping(2)).0]) {
            Ok(op) => Ok((op, 3)),
            Err(code) => Err(Error::InvalidTriple(code))
        }
    }
}

#[cfg(test)]
mod tests;