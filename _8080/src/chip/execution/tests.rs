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
fn dec() {
    let mut env = Socket::default();
    let mut chip = State::new();
    chip[Register::L] = 0x50;
    DecrementByte { register: Byte::Single(Register::L) }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[Register::L], 0x4F);
    assert!(chip.a, "aux flag reset");
    assert!(!chip.p, "parity flag even");
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
    chip[A] = 0b10011100;
    *chip.update_flags() = true;
    ExclusiveOrWith { value: 0b00111110 }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A], 0b10100010);
    assert!(!chip.c, "Carry flag set");
    assert!(!chip.z, "Zero flag set");
    assert!(chip.m, "minus flag reset");
    assert!(!chip.p, "parity flag even");

    chip[D] = 0b10100010;
    ExclusiveOrWithAccumulator { from: Byte::Single(D) }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A], 0);
    assert!(chip.z, "Zero flag reset");
    assert!(chip.p, "parity flag odd");
    assert!(!chip.c, "carry flag set");
    assert!(!chip.m, "sign flag set");
}

#[test]
fn compare_i() {
    let mut env = Socket::default();
    let mut chip = State::new();
    chip[A]  = 0b01011011;
    CompareWith { value: 0b10100011 }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A], 0x5B);
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
fn inc() {
    let mut env = Socket::default();
    let mut chip = State::new();
    chip[D] = 0x17;
    IncrementByte { register: Byte::Single(D) }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[D], 0x18);
    assert!(!chip.a, "Aux flag set\n");
    assert!(chip.p, "parity flag odd\n");
    assert!(!chip.m, "sign flag set\n");
    assert!(!chip.z, "zero flag set\n");
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
    chip[A] = 0x05;
    chip[H] = 0x02;
    chip[L] = 0xA4;
    chip[B] = 0x32;
    env[0x02A4] = 0xD4;
    env[0x0205] = 0xB2;
    Move{to: Byte::Single(L), from: Byte::Single(A)}.execute_on(&mut chip, &mut env).unwrap();
    Move{to: Byte::Single(B), from: Byte::Indirect}.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[B], 0xB2);
}

#[test]
fn or() {
    let mut env = Socket::default();
    let mut chip = State::new();
    chip[A] = 0b01010110;
    *chip.update_flags() = true;
    OrWith{value: 0b00010101}.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A], 0b01010111);
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
    chip[A] = 0b0111_0101;
    RotateRightCarrying.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A], 0b1011_1010);
    assert!(chip.c, "Carry bit cleared");
    RotateRightCarrying.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A], 0b0101_1101);
    assert!(!chip.c, "Carry bit set");
}

#[test]
fn subtract() {
    let mut env = Socket::default();
    let mut chip = State::new();
    chip[A] = 0b1001_0011;
    SubtractBy{value: 0b1011_0110, carry: false}.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A], 0b1101_1101);
    assert!(!chip.a, "Auxilliary carry flag set");
    assert!(chip.c, "Carry flag cleared");
    assert!(!chip.z, "Zero flag set");
    assert!(chip.m, "Sign flag cleared");
    assert!(chip.p, "Parity flag odd");
    SubtractBy { value: 0b1101_1101, carry: false }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A], 0b0000_0000);
    assert!(chip.a, "Auxilliary carry flag clear");
    assert!(!chip.c, "Carry flag set");
    assert!(chip.z, "Zero flag cleared");
    assert!(!chip.m, "Sign flag set");
    assert!(chip.p, "Parity flag odd");
    chip.c = true;
    SubtractBy { value: 0b0011_1100, carry: true }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A], 0b1100_0011);
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
