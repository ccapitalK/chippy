extern crate byteorder;
mod cpu;
use cpu::Cpu;
mod mem;

//use std::fmt;
use std::path;

fn main() {
    let mut cpu = Cpu::new();
    println!("{:?}", cpu);
}