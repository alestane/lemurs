use super::*;    
    
fn decode(value: &[u8]) -> crate::Result<(Op, usize), self::Error> {
    if value.len() < 1 { return Err(self::Error::NoData); }
    let code = match Op::try_from([value[0]]) {
        Ok(op) => return Ok( (op, 1) ),
        Err(code) => code,
    };
    if value.len() < 2 { return Err(self::Error::Invalid(code)); }
    let code = match Op::try_from([code[0], value[1]]) {
        Ok(op) => return Ok( (op, 2) ),
        Err(code) => code,
    };
    if value.len() < 3 { return Err(Error::InvalidPair(code)); }
    match Op::try_from([code[0], code[1], value[2]]) {
        Ok(op) => Ok( (op, 3) ),
        Err(code) => Err(self::Error::InvalidTriple(code))
    } 
}

#[test]
fn no_op() {
    let op = decode(&[0x00]).unwrap();
    assert_eq!(op.0, NOP(4));
}

#[test]
fn inc() {
    let op = decode(&[0x1C]).unwrap();
    assert_eq!(op.0, IncrementByte { register: Byte::Single(Register::E) });
}

#[test]
fn dec() {
    let op = decode(&[0x35, 0x78]).unwrap();
    assert_eq!(op.0, DecrementByte{ register: Byte::Indirect });
}

#[test]
fn and() {
    let op = decode(&[0xE6, 0x79]).unwrap();
    assert_eq!(op.0, AndWith{value: 0x79});
    let op = decode(&[0xA5]).unwrap();
    assert_eq!(op.0, And{from: Byte::Single(Register::L)});
}

#[test]
fn xor() {
    let op = decode(&[0xEE, 0x4D]).unwrap();
    assert_eq!(op.0, ExclusiveOrWith { value: 0x4D });
    let op = decode(&[0xA9]).unwrap();
    assert_eq!(op.0, ExclusiveOr { from: Byte::Single(Register::C) });
}

#[test]
fn or() {
    let op = decode(&[0xF6, 0x23]).unwrap();
    assert_eq!(op.0, OrWith{value: 0x23});
    let op = decode(&[0xB2]).unwrap();
    assert_eq!(op.0, Or{from: Byte::Single(Register::D)});
}

#[test]
fn xthl() {
    let op = decode(&[0xE3, 0x1D]).unwrap();
    assert_eq!(op.0, ExchangeTopWithHilo);
}

#[test]
fn move_() {
    let op = decode(&[0x56]).unwrap();
    assert_eq!(op.0, Move{to: Byte::Single(Register::D), from: Byte::Indirect});
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
fn load_xi() {
    let op = decode(&[0x31, 0x25, 0x02]).unwrap();
    assert_eq!(op.0, LoadExtendedWith { to: Internal::StackPointer, value: 549 });
    let fail = decode(&[0x11, 0x21]).unwrap_err();
    assert_eq!(fail, Error::InvalidPair([0x11, 0x21]));
}

#[test]
fn halt() {
    let op = decode(&[0x76]).unwrap();
    assert_eq!(op.0, Halt);
}

#[test]
fn jump() {
    let op = decode(&[0xC3, 0x74, 0x31]).unwrap();
    assert_eq!(op.0, Jump { to: 0x3174 });
    let op = decode(&[0xF2, 0x31, 0x4A]).unwrap();
    assert_eq!(op.0, JumpIf(Not(Negative), 0x4A31));
    let fail = decode(&[0xFA]).unwrap_err();
    assert_eq!(fail, Error::Invalid([0xFA]));
}

#[test]
fn call() {
    let op = decode(&[0xCD, 0xD3, 0x08]).unwrap();
    assert_eq!(op.0, Call { sub: 0x08D3 });
    let op = decode(&[0xE4, 0x4B, 0x03]).unwrap();
    assert_eq!(op.0, CallIf(Not(EvenParity), 0x034B));
    let fail = decode(&[0xD2, 0x07]).unwrap_err();
    assert_eq!(fail, Error::InvalidPair([0xD2, 0x07]));
}

#[test]
fn add() {
    let op = decode(&[0xC6, 0x39, 0x02]).unwrap();
    assert_eq!(op.0, AddTo { value: 0x39, carry: false });
    let fail = decode(&[0xC6]).unwrap_err();
    assert_eq!(fail, Error::Invalid([0xC6]));
    let op = decode(&[0xCE, 0x72]).unwrap();
    assert_eq!(op.0, AddTo{value: 0x72, carry: true});
    let op = decode(&[0x84, 0x87]).unwrap();
    assert_eq!(op.0, Add{from: Byte::Single(Register::H), carry: false});
    let op = decode(&[0x89]).unwrap();
    assert_eq!(op.0, Add{from: Byte::Single(Register::C), carry: true });
}

#[test]
fn pop() {
    let op = decode(&[0xE1]).unwrap();
    assert_eq!(op.0, Pop(Word::OnBoard(Internal::Wide(Double::HL))));
}

#[test]
fn push() {
    let op = decode(&[0xD5, 0xEB, 0x0E]).unwrap();
    assert_eq!(op.1, 1);
    assert_eq!(op.0, Push(Word::OnBoard(Internal::Wide(Double::DE))));
    let op = decode(&[0xF5, 0xB0]).unwrap();
    assert_eq!(op.0, Push(Word::ProgramStatus));
}

#[test]
fn rotate() {
    let op = decode(&[0x0F, 0x0F]).unwrap();
    assert_eq!(op.0, RotateRightCarrying);
}

#[test]
fn move_i() {
    let op = decode(&[0x0E, 0x09, 0xCD]).unwrap();
    assert_eq!(op.1, 2);
    assert_eq!(op.0, MoveData { value: 0x09, to: Byte::Single(Register::C) });
    let fail = decode(&[0x26]).unwrap_err();
    assert_eq!(fail, Error::Invalid([0x26]));
}

#[test]
fn subtract() {
    let op = decode(&[0xD6, 0x79, 0x01]).unwrap();
    assert_eq!(op.0, SubtractBy{value: 0x79, carry: false});
    let op = decode(&[0xDE, 0x9E]).unwrap();
    assert_eq!(op.0, SubtractBy{value: 0x9E, carry: true});
    let op = decode(&[0x95]).unwrap();
    assert_eq!(op.0, Subtract{from: Byte::Single(Register::L), carry: false});
    let op = decode(&[0x9E]).unwrap();
    assert_eq!(op.0, Subtract{from: Byte::Indirect, carry: true});
}

#[test]
fn compare() {
    let op = decode(&[0xFE, 0x2B]).unwrap();
    assert_eq!(op.1, 2);
    assert_eq!(op.0, CompareWith{value: 0x2B});
    let fail = decode(&[0xFE]).unwrap_err();
    assert_eq!(fail, Error::Invalid([0xFE]));
}
