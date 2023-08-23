use crate::{array, num::NonZeroU8, String, Vec};
use super::{State, access::*};

mod opcode;
use opcode::{Op, Op::*};

pub trait Outcome where Self: Sized {
    type Output<I>;
    fn deliver(count: Option<Self>) -> Self::Output<Self>;
    fn affirm(self) -> Self::Output<Self>;
    fn floor() -> Self::Output<Self>;
    fn did_fail(op: Result<(Op, usize), opcode::Error>) -> Self::Output<(Op, usize)>;
    fn unless<F: FnOnce()>(outcome: Self::Output<Self>, f: F) -> Self::Output<Self>;
}

pub type Failure = Option<Result<String, String>>;

#[cfg(debug_assertions)]
impl Outcome for NonZeroU8 {
    type Output<I> = Result<I, Failure>;
    fn deliver(count: Option<Self>) -> Self::Output<Self> {
        count.ok_or(None)
    }
    fn affirm(self) -> Self::Output<Self> {
        Ok(self)
    }
    fn floor() -> Self::Output<Self> {
        Ok( unsafe { NonZeroU8::new_unchecked(1) } )
    }
    fn did_fail(op: Result<(Op, usize), opcode::Error>) -> Self::Output<(Op, usize)> {
        op.or(Err(None))
    }
    fn unless<F: FnOnce()>(outcome: Self::Output<Self>, f: F) -> Self::Output<Self> {
        outcome.or_else(|e| {f(); Err(e)})
    }
}

#[cfg(not(debug_assertions))]
impl Outcome for NonZeroU8 {
    type Output<I> = Option<I>;
    fn deliver(count: Option<Self>) -> Self::Output<Self> {
        count
    }
    fn affirm(self) -> Self::Output<Self> {
        Some(self)
    }
    fn floor() -> Self::Output<Self> {
        Some(unsafe { NonZeroU8::new_unchecked(1) })
    }
    fn did_fail(op: Result<(Op, usize), opcode::Error>) -> Self::Output<(Op, usize)> {
        op.ok()
    }
    fn unless<F: FnOnce()>(outcome: Self::Output<Self>, f: F) -> Self::Output<Self> {
        outcome.or_else(|| {f(); None})
    }
}

type OpOutcome = <NonZeroU8 as Outcome>::Output<NonZeroU8>;

impl State {
	pub fn execute(&mut self) -> OpOutcome {
		if !self.active { return NonZeroU8::floor() };
        let (op, len) = NonZeroU8::did_fail(Op::extract(&self.ram[self.pc as usize..]))?;
        self.pc += len as u16; 
        let elapsed = NonZeroU8::unless(op.execute_on(self), || self.active = false)?;
        if self.pc as usize >= self.ram.len() { self.active = false };
        NonZeroU8::affirm(elapsed)
	}

    pub fn interrupt(&mut self, op: Op) -> Result<bool, opcode::Error> {
        if op.len() == 1 {
            Ok(self.interrupts && { 
                self.active = true; 
                self.interrupts = false; 
                let _ = op.execute_on(self); 
                true 
            })
        } else {
            Err(opcode::Error::NotUsable(op))
        }
    }

    pub fn reset_to(&mut self, index: usize) -> Result<bool, opcode::OutOfRange> {
        match index {
            0..8 => Ok(self.interrupt(Op::Reset{vector: index as u8}).ok().unwrap()),
            _ => Err(opcode::OutOfRange)
        }
    }
}

#[cfg(debug_assertions)]
fn check_listeners(chip: &mut State, addr: u16) -> Failure {
    fn consolidate<R, F: Fn(R)->String>(sequence: Vec<R>, f: F) -> String {
        sequence.into_iter().map(f)
            .intersperse(String::from("\n")).map(|s| s.chars().collect::<Vec<_>>().into_iter())
            .flatten().collect::<String>()

    }
    unsafe {
        let ram = array::from_ref(&chip.ram[0]) as *const [u8;1];
        let offset = Double::DE << &*chip;
        let switch = chip[Register::C];
        let results: Vec<_> = chip.callbacks.iter().map(|op| op(&*ram, addr, offset, switch)).flatten().collect();
        if results.len() == 0 { return None };
        let (successes, failures): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);
        if failures.len() > 0 {
            Some(Err( consolidate(failures, Result::unwrap_err) ))
        } else {
            Some(Ok( consolidate(successes, Result::unwrap) ))
        }
    };
    None
}

impl Op {
    fn execute_on(self, chip: &mut State) -> OpOutcome {
        let cycles = match self {
            Call{sub} => {
                #[cfg(debug_assertions)]
                if let Some(conclusion) = check_listeners(chip, sub) { return Err(Some(conclusion)); }
                *chip <<= (Word::Stack, chip.pc);
                chip.pc = sub;
                17
            }
            Reset{vector} => {
                *chip <<= (Word::Stack, chip.pc);
                chip.pc = vector as u16 * 8;
                11
            }
            NOP(n) => n,
        };
        NonZeroU8::deliver(NonZeroU8::new(cycles))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reset_from_val() {
        let op = Op::try_from([0xD7]).unwrap();
        assert_eq!(op, Reset{vector: 2});
    }
}