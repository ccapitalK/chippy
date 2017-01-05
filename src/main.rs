extern crate byteorder;
mod cpu;
use cpu::Cpu;
mod mem;
use mem::Mem;

use std::fmt;
use std::path;

fn main() {
    let mut cpu = Cpu::new();
    println!("{:?}", cpu);
}
