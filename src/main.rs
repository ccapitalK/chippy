extern crate byteorder;
extern crate rand;
extern crate sdl2;
mod cpu;
use cpu::Cpu;
use sdl2::messagebox;
use sdl2::pixels;
mod mem;

//use std::fmt;
use std::path;

const WINDOW_WIDTH:  u32 = 640;
const WINDOW_HEIGHT: u32 = 320;

//fn draw_screen

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_sybsys = sdl_context.video().unwrap();
    let window = video_sybsys.window("Chippy", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut renderer = window.renderer().build().unwrap();
    renderer.set_draw_color(pixels::Color::RGB(0, 0, 0));
    renderer.clear();
    renderer.present();

    //messagebox::show_simple_message_box(messagebox::MESSAGEBOX_WARNING, "Hello", "Nope", None)
    //    .expect("Failed to show sdl2 messagebox!");
    let mut cpu = Cpu::new();
    cpu.memory.draw_sprite(0u16, 5u8, 5u8, 5u8);
    println!("{:#?}", cpu);
}
