extern crate byteorder;
extern crate rand;
extern crate sdl2;
mod cpu;
use cpu::Cpu;
use sdl2::messagebox::{show_simple_message_box, MESSAGEBOX_WARNING};
mod mem;

//use std::fmt;
use std::path;

fn main() {
    let mut cpu = Cpu::new();
    //show_simple_message_box(MESSAGEBOX_WARNING, "Hello", "Nope", None)
    //    .expect("Failed to show sdl2 messagebox!");
    println!("{:?}", cpu);
}
