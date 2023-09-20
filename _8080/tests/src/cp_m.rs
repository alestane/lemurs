use lemurs_8080::{Harness, support::*, op};
use core::{ops::{Deref, Index, IndexMut}, num::Wrapping};
use std::collections::HashSet;

#[allow(non_camel_case_types)]
pub struct CP_M {
    dead: u8,
    ram: [u8;0x10000],
    port: [u8;256],
    history: std::collections::HashSet<Wrapping<u16>>,
    order: Vec<Wrapping<u16>>,
}

impl CP_M {
    pub fn with_program(mut code: &[u8]) -> Self {
        let mut new = Self {
            dead: 0,
            ram: [0;_],
            port: [0;_],
            history: HashSet::new(),
            order: vec!()
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
            0..=0x00FF => { self.dead = 0; &mut self.dead },
            i@0x0100.. => &mut self.ram[i as usize],
        }
    }
}

impl Deref for CP_M {
    type Target = [u8];
    fn deref(&self) -> &Self::Target { &self.ram[..] }
}

impl Harness for CP_M {
    fn read(&self, from: Wrapping<u16>) -> Wrapping<u8> { Wrapping(self.ram[from.0 as usize]) }
    fn write(&mut self, value: Wrapping<u8>, to: Wrapping<u16>) { if (0x100..).contains(&to.0) { self.ram[to.0 as usize] = value.0; } }
    fn input(&mut self, port: u8) -> Wrapping<u8> { Wrapping(self.port[port as usize]) }
    fn output(&mut self, port: u8, value: Wrapping<u8>) { self.port[port as usize] = value.0; }
    fn did_execute(&mut self, client: &_8080::State, _did: op::Op) -> Result<Option<op::Op>, String> {
        use _8080::support::{Double, Register};
        self.order.push(client.pc);
        if client.pc.0 >= 0x01AB && self.history.contains(&client.pc) {
            eprintln!("\n{:?}", self.order);
            return Err(format!("Repeated instruction at {:#06X}", client.pc));
        } else {
            self.history.insert(client.pc);
        }
        match client.pc.0 {
            0 => {
                print!("\n");
                return (self.dead == 0).then_some(Some(op::Halt)).ok_or(String::from("Failed tests"));
            }
            5 => { 
                let offset = client[Double::DE].0;
                match client[Register::C].0 {
                    2 => print!("{}", client[Register::E].0 as char),
                    9 => {
                        let text = &self.ram[offset as usize..];
                        if let Some(text) = text.splitn(2, |c| *c == '$' as u8).next() {
                            if let Ok(text) = std::str::from_utf8(text) {
                                print!("{text}");
                            };
                        };
                    }
                    _ => (),
                };
                return Ok(Some(op::Return));
            }
            0x0689 => {
                let from = u16::from_le_bytes([self[client.sp.0], self[client.sp.0 + 1]]) - 3;
                self.dead = true as u8;
                eprintln!("Entered CPU Error routine from {from:#06X}");
                let (a, cy, _ac, pe, m, z) = (client.register[6], client.c as u8, client.a as u8, client.p as u8, client.m as u8, client.z as u8);
                eprintln!("a={a:02X}H,C={cy},P={pe},S={m},Z={z}");
            }
            _ => (),
        };
        Ok ( None )
    }
}