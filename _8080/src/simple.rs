use super::{SimpleBoard, bits};
use core::{any, ops::{Deref, Index, IndexMut, Range, RangeFull, RangeFrom, RangeTo, RangeToInclusive}, num::Wrapping};

impl Default for SimpleBoard {
    fn default() -> Self {
        Self {
            ram: [0; _],
            port_out: [0; _],
            port_in: [0; _],
        }
    }
}

impl Deref for SimpleBoard {
    type Target = [u8];
    fn deref(&self) -> &Self::Target { &self.ram[..] }
}

impl crate::Harness for SimpleBoard {
    fn read(&self, from: bits::u16) -> bits::u8 { Wrapping(self[from.0]) }
    fn read_word(&self, from: bits::u16) -> bits::u16 {
        Wrapping(u16::from_le_bytes([self.ram[from.0 as usize], self.ram[from.0 as usize + 1]]))
    }
    fn write(&mut self, to: bits::u16, value: bits::u8) { self.ram[to.0 as usize] = value.0; }
    fn write_word(&mut self, to: bits::u16, value: bits::u16) {
        [self.ram[to.0 as usize], self.ram[to.0.wrapping_add(1) as usize]] = value.0.to_le_bytes();
    }
	fn input(&mut self, port: u8) -> bits::u8 {
		Wrapping(self.port_in[port as usize])
	}
	fn output(&mut self, port: u8, value: bits::u8) {
		self.port_out[port as usize] = value.0
	}
    fn as_any(&self) -> Option<&dyn any::Any> {
        Some(self)
    }
}

impl Index<u16> for SimpleBoard {
    type Output = u8;
    fn index(&self, index: u16) -> &Self::Output { &self.ram[index as usize] }
}

impl Index<Range<u16>> for SimpleBoard {
    type Output = [u8];
    fn index(&self, index: Range<u16>) -> &Self::Output { &self.ram[index.start as usize..index.end as usize] }
}

impl Index<RangeFrom<u16>> for SimpleBoard {
    type Output = [u8];
    fn index(&self, index: RangeFrom<u16>) -> &Self::Output { &self.ram[index.start as usize..] }
}

impl Index<RangeTo<u16>> for SimpleBoard {
    type Output = [u8];
    fn index(&self, index: RangeTo<u16>) -> &Self::Output { &self.ram[..index.end as usize] }
}

impl Index<RangeToInclusive<u16>> for SimpleBoard {
    type Output = [u8];
    fn index(&self, index: RangeToInclusive<u16>) -> &Self::Output { &self.ram[..=index.end as usize] }
}

impl Index<RangeFull> for SimpleBoard {
    type Output = [u8];
    fn index(&self, _index: RangeFull) -> &Self::Output { &self.ram[..] }
}

impl IndexMut<u16> for SimpleBoard {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output { &mut self.ram[index as usize] }
}

impl IndexMut<Range<u16>> for SimpleBoard {
    fn index_mut(&mut self, index: Range<u16>) -> &mut Self::Output { &mut self.ram[index.start as usize..index.end as usize] }
}

impl IndexMut<RangeFrom<u16>> for SimpleBoard {
    fn index_mut(&mut self, index: RangeFrom<u16>) -> &mut Self::Output { &mut self.ram[index.start as usize..] }
}

impl IndexMut<RangeTo<u16>> for SimpleBoard {
    fn index_mut(&mut self, index: RangeTo<u16>) -> &mut Self::Output { &mut self.ram[..index.end as usize] }
}

impl IndexMut<RangeToInclusive<u16>> for SimpleBoard {
    fn index_mut(&mut self, index: RangeToInclusive<u16>) -> &mut Self::Output { &mut self.ram[..=index.end as usize] }
}

impl IndexMut<RangeFull> for SimpleBoard {
    fn index_mut(&mut self, _index: RangeFull) -> &mut Self::Output { &mut self.ram[..] }
}
