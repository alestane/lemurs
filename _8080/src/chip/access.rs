use core::ops::{Shl, ShlAssign, Index, IndexMut};

use super::State;

pub enum Byte {
	B,
	C,
	D,
	E,
	H, 
	L,
    M,
	A,
    RAM(u16),
	In(u8),
	Out(u8),
}

pub enum Word {
    BC,
    DE,
    HL,
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

impl Index<Byte> for State {
	type Output = u8;
	fn index(&self, i: Byte) -> &Self::Output {
        use Byte::*;
		match i {
            B => &self.register[1],
            C => &self.register[0],
            D => &self.register[3],
            E => &self.register[2],
            H => &self.register[5],
            L => &self.register[4],
            M => &self[Word::HL << self],
            A => &self.register[6],
            RAM(i) => &self[i],
			In(port) => &self.port_in[port as usize],
			Out(port) => &self.port_out[port as usize],
		}
	}
}

impl Shl<&State> for u16 {
    type Output = u16;
    fn shl(self, chip: &State) -> Self::Output {
        u16::from_le_bytes([chip.ram[self as usize], chip.ram[self as usize + 1]])
    }
}

impl ShlAssign<(u16, u16)> for State {
    fn shl_assign(&mut self, (idx, val): (u16, u16)) {
        [self.ram[idx as usize], self.ram[idx as usize + 1]] = val.to_le_bytes();
    }
}

impl ShlAssign<(Word, u16)> for State {
    fn shl_assign(&mut self, (target, val): (Word, u16)) {
        use Word::*;
        match target {
            BC => [self.register[0], self.register[1]] = val.to_le_bytes(),
            DE => [self.register[2], self.register[3]] = val.to_le_bytes(),
            HL => [self.register[4], self.register[5]] = val.to_le_bytes(),
            PSW => {
                let [a, flags] = val.to_le_bytes();
                self.register[6] = a;
                self.extract_flags(flags);
            }
            SP => self.sp = val,
            RAM(i) => *self <<= (i, val),
            Stack => *self <<= (self.sp, val),
            Indirect => *self <<= (HL << self, val),
        };
    }
}

impl Shl<&State> for Word {
    type Output = u16;
    fn shl(self, chip: &State) -> Self::Output {
        use Word::*;
        match self {
            BC => u16::from_le_bytes([chip.register[0], chip.register[1]]),
            DE => u16::from_le_bytes([chip.register[2], chip.register[3]]),
            HL => u16::from_le_bytes([chip.register[4], chip.register[5]]),
            PSW => u16::from_le_bytes([chip.register[6], chip.flags()]),
            SP => chip.sp,
            RAM(i) => i << chip,
            Stack => chip.sp << chip,
            Indirect => (HL << chip) << chip
        }
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
			Zone::RAM => &mut self.ram[..0],
		}		
	}
}