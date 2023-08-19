use core::ops::{Deref, Shl, ShlAssign, Index, IndexMut};

use super::State;

impl Deref for State {
	type Target = [u8];
	fn deref(&self) -> &Self::Target {
		&self.ram[..]
	}
}

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

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Double {
    BC = 0,
    DE = 1,
    HL = 2,
}

#[derive(Debug, Clone, Copy)]
pub enum Byte {
    Single(Register),
    RAM(u16),
	In(u8),
	Out(u8),
}

#[derive(Debug, Clone, Copy)]
pub enum Word {
    Wide(Double),
    PSW,
    SP,
    RAM(u16),
    Stack,
    Indirect,
}

pub enum Zone {
	In, 
	Out,
	RAM,
}

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
}

impl Index<u16> for State {
    type Output = u8;
    fn index(&self, i: u16) -> &Self::Output {
        &self.ram[i as usize]
    }
}

impl IndexMut<u16> for State {
    fn index_mut(&mut self, i: u16) -> &mut Self::Output {
        &mut self.ram[i as usize]
    }
}

impl Index<Register> for State {
    type Output = u8;
    fn index(&self, index: Register) -> &Self::Output {
        match index {
            Register::M => &self[Word::Wide(Double::HL) << self],
            Register::A => &self.register[6],
            index => &self.register[index as usize],
        }
    }
}

impl IndexMut<Register> for State {
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

impl Index<Byte> for State {
	type Output = u8;
	fn index(&self, i: Byte) -> &Self::Output {
        use Byte::*;
		match i {
            Single(index) => &self[index],
            RAM(i) => &self[i],
			In(port) => &self.port_in[port as usize],
			Out(port) => &self.port_out[port as usize],
		}
	}
}

impl IndexMut<Byte> for State {
	fn index_mut(&mut self, i: Byte) -> &mut Self::Output {
        use Byte::*;
		match i {
            Single(index) => &mut self[index],
            RAM(i) => &mut self[i],
			In(_port) => panic!("Can't write to input ports."),
			Out(port) => &mut self.port_out[port as usize],
		}
	}
}

impl Shl<&State> for u16 {
    type Output = u16;
    fn shl(self, chip: &State) -> Self::Output {
        u16::from_le_bytes([chip.ram[self as usize], chip.ram[self as usize + 1]])
    }
}

impl Shl<&State> for Double {
    type Output = u16;
    fn shl(self, chip: &State) -> Self::Output {
        let index = 2 * self as usize;
        u16::from_le_bytes([chip.register[index], chip.register[index + 1]])
    }
}

impl Shl<&State> for Word {
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

impl Shl<&mut State> for Word {
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

impl ShlAssign<(u16, u16)> for State {
    fn shl_assign(&mut self, (idx, val): (u16, u16)) {
        [self.ram[idx as usize], self.ram[idx as usize + 1]] = val.to_le_bytes();
    }
}

impl ShlAssign<(Double, u16)> for State {
    fn shl_assign(&mut self, (pair, value): (Double, u16)) {
        let index = 2 * pair as usize;
        [self.register[index], self.register[index + 1]] = value.to_le_bytes();
    }
}

impl ShlAssign<(Word, u16)> for State {
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

impl Index<Zone> for State {
	type Output = [u8];
	fn index(&self, z: Zone) -> &Self::Output {
		match z {
			Zone::In => &self.port_in[..],
			Zone::Out => &self.port_out[..],
			Zone::RAM => &self.ram[..],
		}
	}
}

impl IndexMut<Zone> for State {
	fn index_mut(&mut self, z: Zone) -> &mut Self::Output {
		match z {
			Zone::In => &mut self.port_in[..],
			Zone::Out => &mut self.port_out[..0],
            #[cfg(not(debug_assertions))]
			Zone::RAM => &mut self.ram[..0],
            #[cfg(debug_assertions)]
            Zone::RAM => &mut self.ram[..],
		}		
	}
}