use super::*;
use crate::{chip::*, SimpleBoard};
use opcode::{Test::*, Flag::*};

macro_rules! flag_name {
    ( m ) => ( "Sign" );
    ( c ) => ( "Carry" );
    ( z ) => ( "Zero" );
    ( p ) => ( "Parity" );
    ( a ) => ( "Aux" );
}

macro_rules! assert_flags {
    { $host:expr $(, !$flag:ident )+ } => {
        $(assert!(!$host.$flag, "{} flag set\n", flag_name!($flag)));+
    };
    { $host:expr $(, $flag:ident )+ } => {
        $(assert!($host.$flag, "{} flag reset\n", flag_name!($flag)));+
    }
}

#[test]
fn add() {
    let mut env = Socket::default();
    let mut chip = State::new();
    chip[A] = Wrapping(0x75);
    AddTo { value: Wrapping(0x49), carry: false }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip.register[6].0, 0xBE);
    assert_flags!(chip, !a, !c, !z);
    assert_flags!{chip, m, p};

    AddTo { value: Wrapping(0x43), carry: false }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip.register[6].0, 0x01, "Sum was {}", chip.register[6]);
    assert_flags!(chip, a, c);
    assert_flags!(chip, !z, !m, !p);

    AddTo { value: Wrapping(0x7E), carry: true }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip.register[6].0, 0x80, "Sum was {}", chip.register[6]);
    assert_flags!(chip, a, m);
    assert_flags!(chip, !c, !z, !p);

    chip[L] = Wrapping(0x5A);
    Add{from:Single(L), carry: false}.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0xDA);
    assert_flags!(chip, !a, !c, !z, !p);
    assert_flags!(chip, m);

    chip.c = true;
    chip[E] = Wrapping(0b0001_1100);
    Add{from: Single(E), carry: true}.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0b1111_0111);
    assert_flags!(chip, a, m);
    assert_flags!(chip, !c, !z, !p);
    Add{from: Single(L), carry: true}.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0b0101_0001);
    assert_flags!(chip, a, c);
    assert_flags!(chip, !z, !p, !m);

    chip[D] = Wrapping(0x45);
    chip[H] = Wrapping(0xE4);
    DoubleAdd{register: Wide(DE)}.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[HL].0, 0x2976);
    assert_flags!(chip, c);
}

#[test]
fn decimal() {
    let mut env = Socket::default();
    let mut chip = State::new();
    chip[A] = Wrapping(0x9B);
    DecimalAddAdjust.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0x01);
    assert_flags!(chip, a, c);
    assert_flags!(chip, !z, !m, !p);
}

#[test]
fn and() {
    let mut env = Socket::default();
    let mut chip = State::new();
    chip[A] = Wrapping(0b01011101);
    AndWith { value: Wrapping(0b11011011) }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip.register[6].0, 0b01011001);
    assert_flags!(chip, !a, !c, !z, !m);
    assert_flags!(chip, p);

    AndWith { value: Wrapping(0b10100100) }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip.register[6].0, 0b00000000);
    assert_flags!(chip, !a, !c, !m);
    assert_flags!(chip, z, p);

    chip[A] = Wrapping(0b0110_1110);
    chip[D] = Wrapping(0b1011_1001);
    And{ from: Single(D) }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0b0010_1000);
    assert_flags!(chip, p);
    assert_flags!(chip, !c, !m, !z);
}

    #[test]
fn call() {
    let mut env = SimpleBoard::default();
    let mut chip = State::new();
    chip.pc = Wrapping(0x000C);
    chip.sp = Wrapping(0x0100);
    let stack = chip.sp;
    env[stack.0] = 0x55;
    Call{sub: Wrapping(0x00A2) }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip.pc.0, 0x00A2);
    assert_eq!(chip.sp.0, 0x00FE);
    assert_eq!(env[0x00FE], 0x0C);
    assert_eq!(env[0x00FF], 0x00);
    assert_eq!(env[0x0100], 0x55);

    chip.register[6] = Wrapping(0xC4);
    AddTo { value: Wrapping(0x3C), carry: false }.execute_on(&mut chip, &mut env).unwrap();
    CallIf(Not(Zero), Wrapping(0x2000)).execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip.pc.0, 0x00A2);
    assert_eq!(chip.sp.0, 0x00FE);
    assert_eq!(env[0x00FE], 0x0C);
    assert_eq!(env[0x00FF], 0x00);
    
    CallIf(Is(EvenParity), Wrapping(0x1300)).execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip.pc.0, 0x1300);
    assert_eq!(chip.sp.0, 0x00FC);
    assert_eq!(env[0x00FC], 0xA2);
    assert_eq!(env[0x00FD], 0x00);
}

#[test]
fn dec() {
    let mut env = Socket::default();
    let mut chip = State::new();
    chip[L] = Wrapping(0x50);
    DecrementByte { register: Single(L) }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[L].0, 0x4F);
    assert_flags!(chip, a);
    assert_flags!(chip, !p);
    chip[H] = Wrapping(0x4C);
    DecrementWord { register: Wide(HL) }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[HL].0, 0x4C4E);
}

#[test]
fn xthl() {
    let mut env = SimpleBoard::default();
    let mut chip = State::new();
    chip.sp = Wrapping(0x7BE3);
    chip[HL] = Wrapping(0x3472);
    [env[0x7BE3], env[0x7BE4]] = [0x43, 0x29];
    ExchangeTopWithHilo.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip.sp.0, 0x7BE3);
    assert_eq!(chip[L].0, 0x43);
    assert_eq!(chip[H].0, 0x29);
    assert_eq!(env.read_word(chip.sp).0, 0x3472);
}

#[test]
fn xor() {
    let mut env = Socket::default();
    let mut chip = State::new();
    chip[A] = Wrapping(0b1001_1100);
    *chip.update_flags() = true;
    ExclusiveOrWith { value: Wrapping(0b0011_1110) }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0b1010_0010);
    assert_flags!(chip, !c, !z, !p);
    assert_flags!(chip, m);

    chip[D] = Wrapping(0b1010_0010);
    ExclusiveOr { from: Single(D) }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0);
    assert_flags!(chip, z, p);
    assert_flags!(chip, !c, !m);
}

#[test]
fn compare() {
    let mut env = Socket::default();
    let mut chip = State::new();
    chip[A] = Wrapping(0b0101_1011);
    CompareWith { value: Wrapping(0b1010_0011) }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0x5B);
    assert_flags!(chip, a, c, m, p);
    assert_flags!(chip, !z);
    chip[L] = Wrapping(0b0010_0110);
    Compare{from: Single(L)}.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0x5B);
    assert_flags!(chip, a, p);
    assert_flags!(chip, !c, !m, !z);
    chip[A].0 = 0xF5;
    CompareWith{value: Wrapping(0)}.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0xF5);
    assert_flags!(chip, p, m);
    assert_flags!(chip, !a, !z, !c);
}

#[test]
fn not() {
    let mut env = Socket::default();
    let mut chip = State::new();
    chip[A] = Wrapping(0b0101_1101);
    ComplementAccumulator.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0b1010_0010);
}

#[test]
fn halt() {
    let mut env = Socket::default();
    let mut chip = State::new();
    chip.pc = Wrapping(0x2534);
    chip.pc += Halt.len() as raw::u16;
    Halt.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip.pc.0, 0x2535);
    assert!(!chip.active, "Processor not stopped");
}

#[test]
fn inc() {
    let mut env = Socket::default();
    let mut chip = State::new();
    chip[D] = Wrapping(0x17);
    IncrementByte { register: Single(D) }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[D].0, 0x18);
    assert_flags!(chip, !a, !m, !z);
    assert_flags!(chip, p);
    chip[E] = Wrapping(0xFF);
    IncrementWord{register: Wide(DE)}.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[DE].0, 0x1900);
}

#[test]
fn jump() {
    let mut env = Socket::default();
    let mut chip = State::new();
    Jump{ to: Wrapping(0x0340) }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip.pc.0, 0x0340);
    chip.register[6] = Wrapping(0x90);
    AddTo { value: Wrapping(0x73), carry: false }.execute_on(&mut chip, &mut env).unwrap();
    JumpIf(Not(Carry), Wrapping(0x1203)).execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip.pc.0, 0x0340);
    assert!(!chip.m, "MINUS flag was {} after result {}", chip.m, chip.register[6]);
    JumpIf(Not(Negative), Wrapping(0x5432)).execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip.pc.0, 0x5432);
}

#[test]
fn transfer() {
    let mut env = Socket::default();
    let mut chip = State::new();
    LoadExtendedWith { to: Wide(HL), value: Wrapping(0x6472) }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip.register[4].0, 0x72);
    assert_eq!(chip.register[5].0, 0x64);
    let mut env = SimpleBoard::default();
    chip.register[6] = Wrapping(0x5D);
    StoreAccumulator { address: Wrapping(0x59D3) }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(env[0x59D3], 0x5D);
    env[0x6275] = 0x6A;
    LoadAccumulator { address: Wrapping(0x6275) }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A], Wrapping(0x6A));
    env[0x8362] = 0x56;
    env[0x8363] = 0xA8;
    LoadHilo{address: Wrapping(0x8362)}.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[H].0, 0xA8);
    assert_eq!(chip[L].0, 0x56);
    StoreHilo{address: Wrapping(0x7632)}.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(env[0x7632], 0x56);
    assert_eq!(env[0x7633], 0xA8);
    chip[BC] = Wrapping(0x7633);
    LoadAccumulatorIndirect { register: BC }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0xA8);
    chip[DE] = Wrapping(0x8349);
    StoreAccumulatorIndirect { register: DE }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(env[0x8349], 0xA8);
}

#[test]
fn move_() {
    let mut env = SimpleBoard::default();
    let mut chip = State::new();
    chip[A] = Wrapping(0x05);
    chip[H] = Wrapping(0x02);
    chip[L] = Wrapping(0xA4);
    chip[B] = Wrapping(0x32);
    env[0x02A4] = 0xD4;
    env[0x0205] = 0xB2;
    Move{to: Single(L), from: Single(A)}.execute_on(&mut chip, &mut env).unwrap();
    Move{to: Single(B), from: Byte::Indirect}.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[B].0, 0xB2);
}

#[test]
fn or() {
    let mut env = Socket::default();
    let mut chip = State::new();
    chip[A] = Wrapping(0b0101_0110);
    *chip.update_flags() = true;
    OrWith{value: Wrapping(0b0001_0101)}.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A], Wrapping(0b0101_0111));
    assert_flags!(chip, !c, !z, !m, !p);
    chip[H] = Wrapping(0b1100_1001);
    Or{from: Single(H)}.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0b1101_1111);
    assert_flags!(chip, m);
    assert_flags!(chip, !c, !z, !p);
}

#[test]
fn pop() {
    let mut env = SimpleBoard::default();
    let mut chip = State::new();
    chip[BC] = Wrapping(0x8372);
    chip[DE] = Wrapping(0x4928);
    chip[HL] = Wrapping(0x5B6E);
    chip.sp =  Wrapping(0x0238);
    [env[0x0238], env[0x0239]] = [0xB6, 0x4E];
    Pop(OnBoard(Wide(BC))).execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[BC].0, 0x4EB6);
    assert_eq!(chip.sp.0, 0x023A);
}

#[test]
fn push() {
    let mut env = SimpleBoard::default();
    let mut chip = State::new();
    chip.sp =  Wrapping(0x4000);
    chip[BC] = Wrapping(0x3256);
    chip[DE] = Wrapping(0x2345);
    chip[HL] = Wrapping(0x7654);
    Push(OnBoard(Wide(HL))).execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(env[0x3FFE], 0x54);
    assert_eq!(env[0x3FFF], 0x76);
    assert_eq!(chip.sp.0, 0x3FFE);

    chip.register[6] = Wrapping(0x90);
    AddTo { value: Wrapping(0x73), carry: false }.execute_on(&mut chip, &mut env).unwrap();
    Push(ProgramStatus).execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip.sp.0, 0x3FFC);
    assert_eq!(env[0x3FFC], 0x03);
    assert_eq!(env[0x3FFD], 0b00000111);
}

#[test]
fn reset() {
    let mut env = SimpleBoard::default();
    let mut chip = State::new();
    chip.pc = Wrapping(0x0391);
    chip.sp = Wrapping(0x0200);
    Reset{vector: 0x05}.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip.pc.0, 0x0028);
    assert_eq!(chip.sp.0, 0x01FE);
    assert_eq!(env[0x01FE], 0x91);
    assert_eq!(env[0x01FF], 0x03);
}

#[test]
fn return_from() {
    let mut env = SimpleBoard::default();
    let mut chip = State::new();
    chip.pc = Wrapping(0x02B6);
    chip.sp = Wrapping(0x8EA5);
    [env[0x8EA5], env[0x8EA6]] = [0xFE,0x01];
    Return.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip.sp.0, 0x8EA7);
    assert_eq!(chip.pc.0, 0x01FE);
}

#[test]
fn rotate() {
    let mut env = Socket::default();
    let mut chip = State::new();
    chip[A] = Wrapping(0b0111_0101);
    RotateRightCarrying.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0b1011_1010);
    assert_flags!(chip, c);
    RotateRightCarrying.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0b0101_1101);
    assert_flags!(chip, !c);
}

#[test]
fn subtract() {
    let mut env = Socket::default();
    let mut chip = State::new();
    chip[A] = Wrapping(0b1001_0011);
    SubtractBy{value: Wrapping(0b1011_0110), carry: false}.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0b1101_1101);
    assert_flags!(chip, c, m, p);
    assert_flags!(chip, !a, !z);
    SubtractBy { value: Wrapping(0b1101_1101), carry: false }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0b0000_0000);
    assert_flags!(chip, a, z, p);
    assert_flags!(chip, !c, !m);
    chip.c = true;
    SubtractBy { value: Wrapping(0b0011_1100), carry: true }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0b1100_0011);
    assert_flags!(chip, c, m, p);
    assert_flags!(chip, !z);

    chip[C] = Wrapping(0b0001_0101);
    Subtract{from: Single(C), carry: false}.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0b1010_1110);
    assert_flags!(chip, m);
    assert_flags!(chip, !c, !z, !a, !p);

    chip[L] = Wrapping(0b0100_1001);
    Subtract{ from: Single(L), carry: true}.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0b0110_0101);
    assert_flags!(chip, a, p);
    assert_flags!(chip, !c, !z, !m);
    chip.c = true;
    Subtract{from: Single(C), carry: true}.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[A].0, 0b0100_1111);
    assert_flags!(chip, !a, !c, !z, !m, !p);
}

#[test]
fn exchange() {
    let mut env = Socket::default();
    let mut chip = State::new();
    chip[D] = Wrapping(0x2B);
    chip[E] = Wrapping(0x43);
    chip[H] = Wrapping(0xD1);
    chip[L] = Wrapping(0x6C);
    ExchangeDoubleWithHilo.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(chip[DE].0, 0xD16C);
    assert_eq!(chip[HL].0, 0x2B43);
}

#[test]
fn move_i() {
    let mut env = SimpleBoard::default();
    let mut chip = State::new();
    chip[HL] = Wrapping(0x0421);
    MoveData { value: Wrapping(0x02), to: Single(H) }.execute_on(&mut chip, &mut env).unwrap();
    MoveData { value: Wrapping(0x72), to: Byte::Indirect }.execute_on(&mut chip, &mut env).unwrap();
    assert_eq!(env[0x0221], 0x72);
}

#[test]
fn internals() {
    let mut env = Socket::default();
    let mut chip = State::new();
    CarryFlag(true).execute_on(&mut chip, &mut env).unwrap();
    assert!(chip.c);
    CarryFlag(false).execute_on(&mut chip, &mut env).unwrap();
    assert!(!chip.c);
}