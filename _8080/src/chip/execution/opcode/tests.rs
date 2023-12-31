use super::{*, Op::*};    
    
fn decode(value: &[raw::u8]) -> Result<(Op, usize), self::Error> {
    Op::extract(value.iter().copied().map(|v| Wrapping(v)))
}

#[test]
fn no_op() {
    let op = decode(&[0x00]).unwrap();
    assert_eq!(op.0, NOP(4));
}

#[test]
fn inc() {
    let op = decode(&[0x1C]).unwrap();
    assert_eq!(op.0, IncrementByte { register: Single(E) });
    let op = decode(&[0x23]).unwrap();
    assert_eq!(op.0, IncrementWord { register: Wide(HL) });
}

#[test]
fn dec() {
    let op = decode(&[0x35, 0x78]).unwrap();
    assert_eq!(op.0, DecrementByte{ register: Byte::Indirect });
    let op = decode(&[0x1B]).unwrap();
    assert_eq!(op.0, DecrementWord{register: Wide(DE)});
}

#[test]
fn and() {
    let op = decode(&[0xE6, 0x79]).unwrap();
    assert_eq!(op.0, AndWith{value: Wrapping(0x79)});
    let op = decode(&[0xA5]).unwrap();
    assert_eq!(op.0, And{from: Single(L)});
}

#[test]
fn xor() {
    let op = decode(&[0xEE, 0x4D]).unwrap();
    assert_eq!(op.0, ExclusiveOrWith { value: Wrapping(0x4D) });
    let op = decode(&[0xA9]).unwrap();
    assert_eq!(op.0, ExclusiveOr { from: Byte::Single(C) });
}

#[test]
fn or() {
    let op = decode(&[0xF6, 0x23]).unwrap();
    assert_eq!(op.0, OrWith{value: Wrapping(0x23)});
    let op = decode(&[0xB2]).unwrap();
    assert_eq!(op.0, Or{from: Single(D)});
}

#[test]
fn not() {
    let op = decode(&[0x2F]).unwrap();
    assert_eq!(op.0, ComplementAccumulator);
}

#[test]
fn xthl() {
    let op = decode(&[0xE3, 0x1D]).unwrap();
    assert_eq!(op.0, ExchangeTopWithHilo);
}

#[test]
fn move_() {
    let op = decode(&[0x56]).unwrap();
    assert_eq!(op.0, Move{to: Single(D), from: Byte::Indirect});
    let op = decode(&[76]).unwrap();
    assert_ne!(op.0, Move{to: Byte::Indirect, from: Byte::Indirect});
}

#[test]
fn reset_from_val() {
    let op = decode(&[0xD7]).unwrap();
    assert_eq!(op.0, Reset{vector: 2});
}

#[test]
fn return_() {
    let op = decode(&[0xC9, 0x31, 0x00]).unwrap();
    assert_eq!(op.0, Return);
}

#[test]
fn return_if() {
    let op = decode(&[0xD8]).unwrap();
    assert_eq!(op.0, ReturnIf(Is(Carry)));
    let op = decode(&[0xF0]).unwrap();
    assert_eq!(op.0, ReturnIf(Not(Negative)));
}

#[test]
fn transfer() {
    let op = decode(&[0x31, 0x25, 0x02]).unwrap();
    assert_eq!(op.0, LoadExtendedWith { to: StackPointer, value: Wrapping(549) });
    let fail = decode(&[0x11, 0x21]).unwrap_err();
    assert_eq!(fail, Error::InvalidPair([Wrapping(0x11), Wrapping(0x21)]));
    let op = decode(&[0x32, 0x67, 0x9A]).unwrap();
    assert_eq!(op.0, StoreAccumulator { address: Wrapping(0x9A67) });
    let op = decode(&[0x3A, 0x31, 0xE7]).unwrap();
    assert_eq!(op.0, LoadAccumulator { address: Wrapping(0xE731) });
    let op = decode(&[0x2A, 0x5B, 0x73]).unwrap();
    assert_eq!(op.0, LoadHilo{address: Wrapping(0x735B)});
    let op = decode(&[0x2A]).unwrap_err();
    assert_eq!(op, Error::Invalid([Wrapping(0x2A)]));
    let op = decode(&[0x22, 0x03, 0x01]).unwrap();
    assert_eq!(op.0, StoreHilo{address: Wrapping(0x0103)});
    let op = decode(&[0x1A]).unwrap();
    assert_eq!(op.0, LoadAccumulatorIndirect { register: DE });
    let op = decode(&[0x02]).unwrap();
    assert_eq!(op.0, StoreAccumulatorIndirect { register: BC });
    let op = decode(&[0x39]).unwrap();
    assert_eq!(op.0, DoubleAdd { register: StackPointer });
    let op = decode(&[0xF9]).unwrap();
    assert_eq!(op.0, StackPointerFromHilo);
    let op = decode(&[0xE9, 0x13]).unwrap();
    assert_eq!(op.0, ProgramCounterFromHilo);
}

#[test]
fn halt() {
    let op = decode(&[0x76]).unwrap();
    assert_eq!(op.0, Halt);
}

#[test]
fn jump() {
    let op = decode(&[0xC3, 0x74, 0x31]).unwrap();
    assert_eq!(op.0, Jump { to: Wrapping(0x3174) });
    let op = decode(&[0xF2, 0x31, 0x4A]).unwrap();
    assert_eq!(op.0, JumpIf(Not(Negative), Wrapping(0x4A31)));
    let fail = decode(&[0xFA]).unwrap_err();
    assert_eq!(fail, Error::Invalid([Wrapping(0xFA)]));
}

#[test]
fn call() {
    let op = decode(&[0xCD, 0xD3, 0x08]).unwrap();
    assert_eq!(op.0, Call { sub: Wrapping(0x08D3) });
    let op = decode(&[0xE4, 0x4B, 0x03]).unwrap();
    assert_eq!(op.0, CallIf(Not(EvenParity), Wrapping(0x034B)));
    let fail = decode(&[0xD2, 0x07]).unwrap_err();
    assert_eq!(fail, Error::InvalidPair([Wrapping(0xD2), Wrapping(0x07)]));
}

#[test]
fn add() {
    let op = decode(&[0xC6, 0x39, 0x02]).unwrap();
    assert_eq!(op.0, AddTo { value: Wrapping(0x39), carry: false });
    let fail = decode(&[0xC6]).unwrap_err();
    assert_eq!(fail, Error::Invalid([Wrapping(0xC6)]));
    let op = decode(&[0xCE, 0x72]).unwrap();
    assert_eq!(op.0, AddTo{value: Wrapping(0x72), carry: true});
    let op = decode(&[0x84, 0x87]).unwrap();
    assert_eq!(op.0, Add{from: Single(H), carry: false});
    let op = decode(&[0x89]).unwrap();
    assert_eq!(op.0, Add{from: Single(C), carry: true });
    let op = decode(&[0x27]).unwrap();
    assert_eq!(op.0, DecimalAddAdjust);
}

#[test]
fn pop() {
    let op = decode(&[0xE1]).unwrap();
    assert_eq!(op.0, Pop(OnBoard(Wide(HL))));
}

#[test]
fn push() {
    let op = decode(&[0xD5, 0xEB, 0x0E]).unwrap();
    assert_eq!(op.1, 1);
    assert_eq!(op.0, Push(OnBoard(Wide(DE))));
    let op = decode(&[0xF5, 0xB0]).unwrap();
    assert_eq!(op.0, Push(ProgramStatus));
}

#[test]
fn rotate() {
    let op = decode(&[0x0F, 0x0F]).unwrap();
    assert_eq!(op.0, RotateRightCarrying);
    let op = decode(&[0x07]).unwrap();
    assert_eq!(op.0, RotateLeftCarrying);
    let op = decode(&[0x17]).unwrap();
    assert_eq!(op.0, RotateAccumulatorLeft);
    let op = decode(&[0x1F, 0x84]).unwrap();
    assert_eq!(op.0, RotateAccumulatorRight)
}

#[test]
fn move_i() {
    let op = decode(&[0x0E, 0x09, 0xCD]).unwrap();
    assert_eq!(op.1, 2);
    assert_eq!(op.0, MoveData { value: Wrapping(0x09), to: Single(C) });
    let fail = decode(&[0x26]).unwrap_err();
    assert_eq!(fail, Error::Invalid([Wrapping(0x26)]));
}

#[test]
fn subtract() {
    let op = decode(&[0xD6, 0x79, 0x01]).unwrap();
    assert_eq!(op.0, SubtractBy{value: Wrapping(0x79), carry: false});
    let op = decode(&[0xDE, 0x9E]).unwrap();
    assert_eq!(op.0, SubtractBy{value: Wrapping(0x9E), carry: true});
    let op = decode(&[0x95]).unwrap();
    assert_eq!(op.0, Subtract{from: Single(L), carry: false});
    let op = decode(&[0x9E]).unwrap();
    assert_eq!(op.0, Subtract{from: Byte::Indirect, carry: true});
}

#[test]
fn compare() {
    let op = decode(&[0xFE, 0x2B]).unwrap();
    assert_eq!(op.1, 2);
    assert_eq!(op.0, CompareWith{value: Wrapping(0x2B)});
    let fail = decode(&[0xFE]).unwrap_err();
    assert_eq!(fail, Error::Invalid([Wrapping(0xFE)]));
    let op = decode(&[0xBF]).unwrap();
    assert_eq!(op.0, Compare{from: Single(A)});
}

#[test]
fn internals() {
    let op = decode(&[0x37]).unwrap();
    assert_eq!(op.0, CarryFlag(true));
    let op = decode(&[0x3F, 0x00]).unwrap();
    assert_eq!(op.0, CarryFlag(false));
    let op = decode(&[0xF3]).unwrap();
    assert_eq!(op.0, Interrupts(false));
    let op = decode(&[0xFB]).unwrap();
    assert_eq!(op.0, Interrupts(true));
}

#[test]
fn io() {
    let op = decode(&[0xD3, 0x85]).unwrap();
    assert_eq!(op.0, Out(0x85));
    let op = decode(&[0xDB, 0x45]).unwrap();
    assert_eq!(op.0, In(0x45));
}

#[test]
fn all() {
    for op in 0u8..=255u8 {
        match decode(&[op, 0x45, 0x3B]) {
            Ok(..) | Err(Error::Unknown(..)) => (),
            Err(err) => panic!("{err:X}"),
        };
    }
}