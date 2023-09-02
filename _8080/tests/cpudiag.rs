#![feature(generic_arg_infer)]

extern crate _8080;
use _8080::State;

mod src {
    pub mod cp_m;
}
use src::*;

#[cfg(debug_assertions)]
#[test]
fn exercise() {
    println!("currently at {}", std::env::current_dir().unwrap().display());
    let body = std::fs::read("tests/cpudiag.bin").expect("Couldn't load test file.");
    let mut machine = cp_m::CP_M::with_program(&body);
    let sample = State::with(&mut machine);
    let mut cycles = 0usize;
    for outcome in sample {
        match outcome {
            Ok(duration) => cycles += usize::from(duration),
            Err(txt) => {
                panic!("Stopped without completing after {cycles} cycles.\n{txt}\n");
            }
        }
    };
//    sample.map(usize::from).sum();
    println!("Completed successfully.");
    println!("Total of {cycles} cycles executed.")
}