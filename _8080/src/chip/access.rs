use core::{ops::{Shl, ShlAssign, Index, IndexMut, DerefMut}, mem};
use crate::{chip::State, Machine, Harness, Wrapping, bits};

#[cfg(target_endian="little")]
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Register {
    A = 6,
    B = 1,
    C = 0,
    D = 3,
    E = 2, 
    H = 5,
    L = 4,
}

#[cfg(target_endian="big")]
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Register {
    A = 6,
    B = 0,
    C = 1,
    D = 2,
    E = 3, 
    H = 4,
    L = 5,
}


impl Byte {
    pub fn use_bus(&self) -> bool {
        match self { Byte::Indirect | Byte::RAM(_) => true, _ => false }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Double {
    BC = 0,
    DE = 1,
    HL = 2,
}

use self::{Register as R, Double as D, Internal as I, Word as W};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Byte {
    Single(Register),
    Indirect,
    RAM(bits::u16),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Internal {
    Wide(Double),
    ProgramCounter,
    StackPointer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Word {
    OnBoard(Internal),
    ProgramStatus,
    RAM(bits::u16),
    Stack,
}

#[disclose(super)]
impl State {
    fn flags(&self) -> u8 {
        self.c as u8 | 
        0b10u8 |
        (self.p as u8) << 2 |
        (self.a as u8) << 4 |
        (self.z as u8) << 6 |
        (self.m as u8) << 7
    }
    fn extract_flags(&mut self, bits: u8) {
        (self.c, self.p, self.a, self.z, self.m) = (
            bits & 0b00000001 != 0, 
            bits & 0b00000100 != 0,
            bits & 0b00010000 != 0,
            bits & 0b01000000 != 0,
            bits & 0b10000000 != 0,
        );
    }
    #[must_use]
    fn update_flags(&mut self) -> &mut bool {
        self.update_flags_for(self[Register::A])
    }
    #[must_use]
    fn update_flags_for(&mut self, value: bits::u8) -> &mut bool {
        let value = value.0;
        let mut parity = value;
        for offset in [4, 2, 1] {
            parity ^= parity >> offset;
        }
        self.p = (parity & 0x01) == 0;
        self.z = value == 0;
        self.m = value & 0b1000_0000 != 0;
        self.a = false;
        &mut self.c
    }
    fn status(&self) -> bits::u16 {
        Wrapping(u16::from_le_bytes([self[Register::A].0, self.flags()]))
    }

    fn push(&mut self) -> bits::u16 {
        self.sp -= 2;
        self.sp
    }

    fn pop(&mut self) -> bits::u16 {
        let address = self.sp;
        self.sp += 2;
        address
    }

    fn resolve(&self, target: Byte) -> Byte {
        match target {
            Byte::Indirect => Byte::RAM(self[D::HL]),
            _ => target
        }
    }
}

impl Index<Register> for State {
    type Output = bits::u8;
    fn index(&self, index: Register) -> &Self::Output { &self.register[index as usize] }
}

impl IndexMut<Register> for State {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output { &mut self.register[index as usize] }
}

impl Index<Double> for State {
    type Output = bits::u16;
    fn index(&self, index: Double) -> &Self::Output {
        let index = 2 * index as u8;
        unsafe{ mem::transmute::<&bits::u8, &Self::Output>(&self.register[index as usize]) }
    }
}

impl IndexMut<Double> for State {
    fn index_mut(&mut self, index: Double) -> &mut Self::Output {
        let index = 2 * index as u8;
        unsafe{ mem::transmute::<&mut bits::u8, &mut Self::Output>(&mut self.register[index as usize]) }
    }
}

impl Index<Internal> for State {
    type Output = bits::u16;
    fn index(&self, index: Internal) -> &Self::Output {
        match index {
            I::Wide(pair) => &self[pair],
            I::ProgramCounter => &self.pc,
            I::StackPointer => &self.sp,
        }
    }
}

impl IndexMut<Internal> for State {
    fn index_mut(&mut self, index: Internal) -> &mut Self::Output {
        match index {
            I::Wide(pair) => &mut self[pair],
            I::ProgramCounter => &mut self.pc,
            I::StackPointer => &mut self.sp,
        }
    }
}

impl<H: Harness, C: DerefMut<Target = H>> Shl<&Machine<H, C>> for bits::u16 {
    type Output = bits::u16;
    fn shl(self, host: &Machine<H, C>) -> Self::Output {
        host.board.read_word(self)
    }
}

impl<H: Harness, C: DerefMut<Target = H>> Shl<&Machine<H, C>> for Word {
    type Output = bits::u16;
    fn shl(self, host: &Machine<H, C>) -> Self::Output {
        match self {
            Self::OnBoard(internal) => host.chip[internal],
            Self::ProgramStatus => Wrapping(u16::from_le_bytes([host.chip.register[6].0, host.chip.flags()])),
            Self::RAM(i) => host.board.read_word(i),
            Self::Stack => panic!("Can't pop from stack without mutate access"),
        }
    }
}

impl<H: Harness, C: DerefMut<Target = H>> Shl<&mut Machine<H, C>> for Word {
    type Output = bits::u16;
    fn shl(self, host: &mut Machine<H, C>) -> Self::Output {
        match self {
            Word::Stack => {
                let addr = host.chip.sp;
                host.chip.sp += 2;
                host.board.read_word(addr)
            }
            _ => self << &*host,
        }
    }
}

impl<H: Harness, C: DerefMut<Target = H>> ShlAssign<(bits::u16, bits::u16)> for Machine<H, C> {
    fn shl_assign(&mut self, (index, value): (bits::u16, bits::u16)) {
        self.board.write_word(index, value);
    }
}

impl<H: Harness, C: DerefMut<Target = H>> ShlAssign<(Word, bits::u16)> for Machine<H, C> {
    fn shl_assign(&mut self, (index, value): (Word, bits::u16)) {
        match index {
            W::OnBoard(internal) => self.chip[internal] = value,
            W::RAM(address) => self.board.write_word(address, value),
            W::ProgramStatus => {
                let [a, f] = value.0.to_le_bytes();
                self.chip[R::A] = Wrapping(a);
                self.chip.extract_flags(f);
            }
            W::Stack => {
                self.chip.sp -= 2;
                self.board.write_word(self.chip.sp, value);
            }
        }
    }
}