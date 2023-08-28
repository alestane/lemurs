use core::ops::{Shl, ShlAssign, Index, IndexMut};

use super::State;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Register {
    A = 7,
    B = 1,
    C = 0,
    D = 3,
    E = 2, 
    H = 5,
    L = 4,
    M = 6,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Double {
    BC = 0,
    DE = 1,
    HL = 2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Byte {
    Single(Register),
    RAM(u16),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Word {
    Wide(Double),
    PSW,
    SP,
    RAM(u16),
    Stack,
    Indirect,
}

impl State<'_> {
    pub fn flags(&self) -> u8 {
        self.c as u8 | 
        0b10u8 |
        (self.p as u8) << 2 |
        (self.a as u8) << 4 |
        (self.z as u8) << 6 |
        (self.m as u8) << 7
    }
    pub fn extract_flags(&mut self, bits: u8) {
        (self.c, self.p, self.a, self.z, self.m) = (
            bits & 0b00000001 != 0, 
            bits & 0b00000100 != 0,
            bits & 0b00010000 != 0,
            bits & 0b01000000 != 0,
            bits & 0b10000000 != 0,
        );
    }
    pub fn update_flags(&mut self) -> &mut bool {
        let accumulator = self.register[6];
        let mut parity = accumulator;
        for offset in [4, 2, 1] {
            parity ^= parity >> offset;
        }
        self.p = parity & 0b01 == 0;
        self.z = accumulator == 0;
        self.m = accumulator & 0b1000_0000 != 0;
        self.a = false;
        &mut self.c
    }
}

impl Index<u16> for State<'_> {
    type Output = u8;
    fn index(&self, i: u16) -> &Self::Output {
        &self.board[i]
    }
}

impl IndexMut<u16> for State<'_> {
    fn index_mut(&mut self, i: u16) -> &mut Self::Output {
        &mut self.board[i]
    }
}

impl Index<Register> for State<'_> {
    type Output = u8;
    fn index(&self, index: Register) -> &Self::Output {
        match index {
            Register::M => &self[Word::Wide(Double::HL) << self],
            Register::A => &self.register[6],
            index => &self.register[index as usize],
        }
    }
}

impl IndexMut<Register> for State<'_> {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        match index {
            Register::M => {
                let addr = Word::Wide(Double::HL) << &*self;
                &mut self[addr]
            }
            Register::A => &mut self.register[6],
            index => &mut self.register[index as usize],
        }
    }
}

impl Index<Byte> for State<'_> {
	type Output = u8;
	fn index(&self, i: Byte) -> &Self::Output {
        use Byte::*;
		match i {
            Single(index) => &self[index],
            RAM(i) => &self[i],
		}
	}
}

impl IndexMut<Byte> for State<'_> {
	fn index_mut(&mut self, i: Byte) -> &mut Self::Output {
        use Byte::*;
		match i {
            Single(index) => &mut self[index],
            RAM(i) => &mut self[i],
		}
	}
}

impl Shl<&State<'_>> for u16 {
    type Output = u16;
    fn shl(self, chip: &State) -> Self::Output {
        u16::from_le_bytes([chip[self], chip[self + 1]])
    }
}

impl Shl<&State<'_>> for Double {
    type Output = u16;
    fn shl(self, chip: &State) -> Self::Output {
        let index = 2 * self as usize;
        u16::from_le_bytes([chip.register[index], chip.register[index + 1]])
    }
}

impl Shl<&State<'_>> for Word {
    type Output = u16;
    fn shl(self, chip: &State) -> Self::Output {
        use Word::*;
        match self {
            Wide(pair) => pair << chip,
            PSW => u16::from_le_bytes([chip.register[6], chip.flags()]),
            SP => chip.sp,
            RAM(i) => i << chip,
            Stack => panic!("Can't pop from stack without mutate access"),
            Indirect => (Double::HL << chip) << chip
        }
    }
}

impl Shl<&mut State<'_>> for Word {
    type Output = u16;
    fn shl(self, chip: &mut State) -> Self::Output {
        match self {
            Word::Stack => {
                let addr = chip.sp;
                chip.sp += 2;
                addr << &*chip
            }
            _ => self << &*chip,
        }
    }
}

impl ShlAssign<(u16, u16)> for State<'_> {
    fn shl_assign(&mut self, (idx, val): (u16, u16)) {
        [self[idx], self[idx+ 1]] = val.to_le_bytes();
    }
}

impl ShlAssign<(Double, u16)> for State<'_> {
    fn shl_assign(&mut self, (pair, value): (Double, u16)) {
        let index = 2 * pair as usize;
        [self.register[index], self.register[index + 1]] = value.to_le_bytes();
    }
}

impl ShlAssign<(Word, u16)> for State<'_> {
    fn shl_assign(&mut self, (target, val): (Word, u16)) {
        use Word::*; use Double::*;
        match target {
            Wide(pair) => *self <<= (pair, val),
            PSW => {
                let [a, flags] = val.to_le_bytes();
                self.register[6] = a;
                self.extract_flags(flags);
            }
            SP => self.sp = val,
            RAM(i) => *self <<= (i, val),
            Stack => { self.sp -= 2; *self <<= (self.sp, val) },
            Indirect => *self <<= (HL << &*self, val),
        };
    }
}
