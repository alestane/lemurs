extern crate _8080;

use _8080::{Harness, support::*};
use core::ops::{Deref, Index, IndexMut};

#[allow(non_camel_case_types)]
pub struct CP_M {
    dead: u8,
    ram: [u8;0x10000],
    port: [u8;256],
}

impl CP_M {
    pub fn with_program(mut code: &[u8]) -> Self {
        let mut new = Self {
            dead: 0,
            ram: [0;_],
            port: [0;_],
        };
        [new.ram[0], new.ram[1], new.ram[2]] = [0xC3, 0x00, 0x01];
        let mut ram = &mut new.ram[0x100..];
        if ram.len() > code.len() { ram = &mut ram[..code.len()]; }
        if code.len() > ram.len() { code = &code[..ram.len()]; }
        ram.copy_from_slice(code);
        new
    }
}

impl Index<u16> for CP_M {
    type Output = u8;
    fn index(&self, index: u16) -> &Self::Output { &self.ram[index as usize] }
}

impl IndexMut<u16> for CP_M {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        match index {
            0..=255 => { self.dead = 0; &mut self.dead },
            i@256.. => &mut self.ram[i as usize],
        }
    }
}

impl Deref for CP_M {
    type Target = [u8];
    fn deref(&self) -> &Self::Target { &self.ram[..] }
}

impl Harness for CP_M {
    fn input(&mut self, port: u8) -> u8 { self.port[port as usize] }
    fn output(&mut self, port: u8, value: u8) { self.port[port as usize] = value; }
    fn did_execute(&self, client: &_8080::State) -> Result<(), Result<String, String>> {
        match client.pc {
            0 => Err(Err(String::from("aborted"))),
            5 => { 
                let offset = Word::Wide(Double::DE) << client;
                match client[Byte::Single(Register::C)] {
                    2 => println!("print char routine called"),
                    9 => {
                        let text = &self.ram[offset as usize + 3..];
                        if let Some(text) = text.splitn(2, |c| *c == '$' as u8).next() {
                            if let Ok(text) = std::str::from_utf8(text) {
                                println!("{text}");
                            }
                        };
                    }
                    _ => ()
                };
                Err(Ok(format!("Called end display routine at {offset}")))
            }
            _ => Ok( () ),
        }
    }
}