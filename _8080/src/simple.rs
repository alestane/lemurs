use crate::prelude::*;
use super::SimpleBoard;
use core::{any::Any, ops::{Range, RangeFull, RangeFrom, RangeTo, RangeToInclusive}};

impl Default for SimpleBoard {
    fn default() -> Self {
        Self {
            ram: [Wrapping(0); _],
            port_out: [Wrapping(0); _],
            port_in: [Wrapping(0); _],
        }
    }
}

impl Deref for SimpleBoard {
    type Target = [u8];
    fn deref(&self) -> &Self::Target { &self.ram[..] }
}

impl crate::Harness for SimpleBoard {
    fn read(&self, from: u16) -> u8 { self.ram[from.0 as usize] }
    fn read_word(&self, from: u16) -> u16 {
        Wrapping(raw::u16::from_le_bytes([self.ram[from.0 as usize].0, self.ram[from.0 as usize + 1].0]))
    }
    fn write(&mut self, to: u16, value: u8) { self.ram[to.0 as usize] = value; }
    fn write_word(&mut self, to: u16, value: u16) {
        let [lo, hi] = value.0.to_le_bytes();
        [self.ram[to.0 as usize], self.ram[to.0.wrapping_add(1) as usize]] = [Wrapping(lo), Wrapping(hi)];
    }
	fn input(&mut self, port: raw::u8) -> u8 {
		self.port_in[port as usize]
	}
	fn output(&mut self, port: raw::u8, value: u8) {
		self.port_out[port as usize] = value;
	}
    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

impl Index<raw::u16> for SimpleBoard {
    type Output = u8;
    fn index(&self, index: raw::u16) -> &Self::Output { &self.ram[index as usize] }
}

impl Index<Range<raw::u16>> for SimpleBoard {
    type Output = [u8];
    fn index(&self, index: Range<raw::u16>) -> &Self::Output { &self.ram[index.start as usize..index.end as usize] }
}

impl Index<RangeFrom<raw::u16>> for SimpleBoard {
    type Output = [u8];
    fn index(&self, index: RangeFrom<raw::u16>) -> &Self::Output { &self.ram[index.start as usize..] }
}

impl Index<RangeTo<raw::u16>> for SimpleBoard {
    type Output = [u8];
    fn index(&self, index: RangeTo<raw::u16>) -> &Self::Output { &self.ram[..index.end as usize] }
}

impl Index<RangeToInclusive<raw::u16>> for SimpleBoard {
    type Output = [u8];
    fn index(&self, index: RangeToInclusive<raw::u16>) -> &Self::Output { &self.ram[..=index.end as usize] }
}

impl Index<RangeFull> for SimpleBoard {
    type Output = [u8];
    fn index(&self, _index: RangeFull) -> &Self::Output { &self.ram[..] }
}

impl IndexMut<raw::u16> for SimpleBoard {
    fn index_mut(&mut self, index: raw::u16) -> &mut Self::Output { &mut self.ram[index as usize] }
}

impl IndexMut<Range<raw::u16>> for SimpleBoard {
    fn index_mut(&mut self, index: Range<raw::u16>) -> &mut Self::Output { &mut self.ram[index.start as usize..index.end as usize] }
}

impl IndexMut<RangeFrom<raw::u16>> for SimpleBoard {
    fn index_mut(&mut self, index: RangeFrom<raw::u16>) -> &mut Self::Output { &mut self.ram[index.start as usize..] }
}

impl IndexMut<RangeTo<raw::u16>> for SimpleBoard {
    fn index_mut(&mut self, index: RangeTo<raw::u16>) -> &mut Self::Output { &mut self.ram[..index.end as usize] }
}

impl IndexMut<RangeToInclusive<raw::u16>> for SimpleBoard {
    fn index_mut(&mut self, index: RangeToInclusive<raw::u16>) -> &mut Self::Output { &mut self.ram[..=index.end as usize] }
}

impl IndexMut<RangeFull> for SimpleBoard {
    fn index_mut(&mut self, _index: RangeFull) -> &mut Self::Output { &mut self.ram[..] }
}
