#![feature(lazy_cell)]

extern crate _8080;
use _8080::State;

fn cp_m(ram:  &[u8], addr: u16, offset: u16, switch: u8) -> Option<Result<String, String>> {
    match addr {
        0 => Some(Err(String::from("aborted"))),
        5 => { 
            match switch {
                2 => println!("print char routine called"),
                9 => {
                    let text = &ram[offset as usize + 3..];
                    if let Some(text) = text.splitn(2, |c| *c == '$' as u8).next() {
                        if let Ok(text) = std::str::from_utf8(text) {
                            println!("{text}");
                        }
                    };
                }
                _ => ()
            };
            Some(Err(format!("Called end display routine at {offset}")))
        }
        _ => None,
    }
    
}

#[cfg(debug_assertions)]
#[test]
fn exercise() {
    println!("currently at {}", std::env::current_dir().unwrap().display());
    let mut ram = vec![0;256];
    (ram[0], ram[1], ram[2]) = (0xC3, 0x00, 0x01);
    let mut body = std::fs::read("tests/cpudiag.bin").expect("Couldn't load test file.");
    ram.append(&mut body);
    let mut sample = State::from(ram.as_slice());
    sample.add_callback(&cp_m);
    let cycles: usize = sample.map(usize::from).sum();
    println!("Total of {cycles} cycles executed.")
}