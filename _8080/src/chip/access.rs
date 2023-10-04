use crate::prelude::*;
use core::{mem, ops::{Shl, ShlAssign}};

/// This enumerates the internal byte registers of the 8080. You can access these
/// by indexing the State struct with them; `let val = st[Register::D];`
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

/// A register pair in the CPU, interpreted as a little-endian 16-bit value,
/// where the `B`, `D` or `H` register contains the more-significant byte and
/// the `C`, `E` or `L` register contains the less-significant byte.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Double {
    BC = 0,
    DE = 1,
    HL = 2,
}

use self::{Register as R, Double as D, Internal as I, Word as W};

/// `Byte` enumerates any one-byte region that can be specified for a read or write:
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Byte {
    /// - A byte register on the CPU chip (B, C, D, E, H, L, A)
    Single(Register),
    /// - A byte in memory at the address contained in the `HL` register pair
    Indirect,
    /// - A byte in memory at a specified address (usually for ops like `LXI`)
    RAM(u16),
}

/// Any of the 16-bit registers in the CPU, including the register pairs,
/// but also the program counter and the stack pointer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Internal {
    Wide(Double),
    ProgramCounter,
    StackPointer,
}
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Word {
    OnBoard(Internal),
    ProgramStatus,
    RAM(u16),
    Stack,
}

#[disclose(super)]
impl State {
    /// This method exposes the flag bits of the 8080 in the same format as the PSW
    /// pseudo-register, `mz0a0p1c`, where
    ///
    /// - `m` is the sign flag;
    /// - `z` is the zero flag;
    /// - `a` is the auxilliary carry flag;
    /// - `p` is the even-parity flag;
    /// - `c` is the carry flag.
    pub fn flags(&self) -> raw::u8 {
        self.c as raw::u8 |
        0b10u8 |
        (self.p as raw::u8) << 2 |
        (self.a as raw::u8) << 4 |
        (self.z as raw::u8) << 6 |
        (self.m as raw::u8) << 7
    }
    /// Whether or not the processor is in a stopped state (not executing operations from the PC).
    /// The processor will return to an active state if it receives an interrupt.
    ///
    /// Note that if interrupts are disabled when the processor is halted, the processor will
    /// remain stopped until it is reset from outside.
    pub fn is_stopped(&self) -> bool { !self.active }
    /// Whether the processor is accepting interrupts; this is disabled automatically when an
    /// interrupt is received, to allow the interrupt to finish processing without being further
    /// disrupted. It is also set by the `EI` operation and reset by the `DI` operation.
    pub fn is_interrupt_ready(&self) -> bool { self.interrupts }
    fn extract_flags(&mut self, bits: raw::u8) {
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
    fn update_flags_for(&mut self, value: u8) -> &mut bool {
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
    fn status(&self) -> u16 {
        Wrapping(raw::u16::from_le_bytes([self[Register::A].0, self.flags()]))
    }

    fn push(&mut self) -> u16 {
        self.sp -= 2;
        self.sp
    }

    fn pop(&mut self) -> u16 {
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
    type Output = u8;
    fn index(&self, index: Register) -> &Self::Output { &self.register[index as usize] }
}

impl IndexMut<Register> for State {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output { &mut self.register[index as usize] }
}

impl Index<Double> for State {
    type Output = u16;
    fn index(&self, index: Double) -> &Self::Output {
        let index = 2 * index as raw::u8;
        unsafe{ mem::transmute::<&u8, &Self::Output>(&self.register[index as usize]) }
    }
}

impl IndexMut<Double> for State {
    fn index_mut(&mut self, index: Double) -> &mut Self::Output {
        let index = 2 * index as raw::u8;
        unsafe{ mem::transmute::<&mut u8, &mut Self::Output>(&mut self.register[index as usize]) }
    }
}

impl Index<Internal> for State {
    type Output = u16;
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

impl<H: Harness + ?Sized, C: BorrowMut<H>> Shl<&Machine<H, C>> for u16 {
    type Output = u16;
    fn shl(self, host: &Machine<H, C>) -> Self::Output {
        host.read_word(self)
    }
}

impl<H: Harness + ?Sized, C: BorrowMut<H>> Shl<&Machine<H, C>> for Word {
    type Output = u16;
    fn shl(self, host: &Machine<H, C>) -> Self::Output {
        match self {
            Self::OnBoard(internal) => host.chip[internal],
            Self::ProgramStatus => {
            	Wrapping(raw::u16::from_le_bytes([host.chip.register[6].0, host.chip.flags()]))
            }
            Self::RAM(i) => host.read_word(i),
            Self::Stack => panic!("Can't pop from stack without mutate access"),
        }
    }
}

impl<H: Harness + ?Sized, C: BorrowMut<H>> Shl<&mut Machine<H, C>> for Word {
    type Output = u16;
    fn shl(self, host: &mut Machine<H, C>) -> Self::Output {
        match self {
            Word::Stack => {
                let addr = host.chip.sp;
                host.chip.sp += 2;
                host.read_word(addr)
            }
            _ => self << &*host,
        }
    }
}

impl<H: Harness + ?Sized, C: BorrowMut<H>> ShlAssign<(u16, u16)> for Machine<H, C> {
    fn shl_assign(&mut self, (index, value): (u16, u16)) {
        self.write_word(index, value);
    }
}

impl<H: Harness + ?Sized, C: BorrowMut<H>> ShlAssign<(Word, u16)> for Machine<H, C> {
    fn shl_assign(&mut self, (index, value): (Word, u16)) {
        match index {
            W::OnBoard(internal) => self.chip[internal] = value,
            W::RAM(address) => self.write_word(address, value),
            W::ProgramStatus => {
                let [a, f] = value.0.to_le_bytes();
                self.chip[R::A] = Wrapping(a);
                self.chip.extract_flags(f);
            }
            W::Stack => {
                self.chip.sp -= 2;
                let sp = self.chip.sp;
                self.write_word(sp, value);
            }
        }
    }
}
