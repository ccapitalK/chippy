extern crate sdl2;
use sdl2::pixels;
use sdl2::keyboard::Keycode;
use sdl2::render::Renderer;
use cpu::Cpu;
use mem;

pub const WINDOW_WIDTH : u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;

const VIEW_WIDTH:  u32 = 640;
const VIEW_HEIGHT: u32 = 320;

const KEYS_WIDTH:  u32 = 280;
const KEYS_HEIGHT: u32 = 280;

pub fn draw_view(renderer: &mut Renderer, cpu: &Cpu, x: i32, y: i32){
    renderer.set_draw_color(pixels::Color::RGB(255, 255, 255));
    let x_off = x;
    let y_off = y;
    {
        let border_rect = sdl2::rect::Rect::new(0, 0, VIEW_WIDTH, VIEW_HEIGHT);
        if let Err(v) = renderer.draw_rect(border_rect) {
            panic!("Call to fill_rect({:?}) failed: {}", border_rect, v);
        }
    }
    for x in 0..mem::SCREEN_WIDTH {
        for y in 0..mem::SCREEN_HEIGHT {

            let x = x as u32;
            let y = y as u32;

            let x0: i32 = 
                ((x*VIEW_WIDTH)/(mem::SCREEN_WIDTH as u32)) as i32;
            let y0: i32 = 
                ((y*VIEW_HEIGHT)/(mem::SCREEN_HEIGHT as u32)) as i32;
            let x1: i32 = 
                (((x+1)*VIEW_WIDTH)/(mem::SCREEN_WIDTH as u32)) as i32;
            let y1: i32 = 
                (((y+1)*VIEW_HEIGHT)/(mem::SCREEN_HEIGHT as u32)) as i32;

            let square_rect = sdl2::rect::Rect::new(x_off+x0, y_off+y0, (x1-x0) as u32, (y1-y0) as u32);
            if cpu.memory.get_cell(x as u8, y as u8) {
                if let Err(v) = renderer.fill_rect(square_rect) {
                    panic!("Call to fill_rect({:?}) failed: {}", square_rect, v);
                }
            }

        }
    }
}

pub fn draw_keys(renderer: &mut Renderer, cpu: &Cpu, x: i32, y: i32){
    let x_off = x;
    let y_off = y;
    for x in 0..4 {
        for y in 0..4 {
            let x0: i32 = 
                ((x*KEYS_WIDTH)/(4 as u32)) as i32;
            let y0: i32 = 
                ((y*KEYS_HEIGHT)/(4 as u32)) as i32;
            let x1: i32 = 
                (((x+1)*KEYS_WIDTH)/(4 as u32)) as i32;
            let y1: i32 = 
                (((y+1)*KEYS_HEIGHT)/(4 as u32)) as i32;

            let key_num = (x as u8)+(y as u8)*4u8;
            renderer.set_draw_color(pixels::Color::RGB(255, 255, 255));
            let square_rect = sdl2::rect::Rect::new(x_off+x0, y_off+y0, (x1-x0) as u32, (y1-y0) as u32);
            if cpu.get_key(key_num) {
                if let Err(v) = renderer.fill_rect(square_rect) {
                    panic!("Call to fill_rect({:?}) failed: {}", square_rect, v);
                }
            }else{
                let key_fade = cpu.get_key_trail(key_num);
                renderer.set_draw_color(pixels::Color::RGB(0, 0, key_fade));
                if let Err(v) = renderer.fill_rect(square_rect) {
                    panic!("Call to fill_rect({:?}) failed: {}", square_rect, v);
                }
                renderer.set_draw_color(pixels::Color::RGB(255, 255, 255));
                if let Err(v) = renderer.draw_rect(square_rect) {
                    panic!("Call to draw_rect({:?}) failed: {}", square_rect, v);
                }
            }
        }
    }
}

pub fn draw_screen(renderer: &mut Renderer, cpu: &Cpu){
    renderer.set_draw_color(pixels::Color::RGB(0, 0, 0));
    renderer.clear();
    draw_view(renderer, cpu, 0, 0);
    draw_keys(renderer, cpu, 0, 320);
    renderer.present();
}

pub enum KeyState {
    KeyDown,
    KeyUp,
}

#[inline(always)]
pub fn parse_input(cpu: &mut Cpu, keycode: Keycode, state: KeyState){
    let key_index = match keycode {
        Keycode::Num4 => Some(0x0u8),
        Keycode::Num5 => Some(0x1u8),
        Keycode::Num6 => Some(0x2u8),
        Keycode::Num7 => Some(0x3u8),

        Keycode::R    => Some(0x4u8),
        Keycode::T    => Some(0x5u8),
        Keycode::Y    => Some(0x6u8),
        Keycode::U    => Some(0x7u8),

        Keycode::F    => Some(0x8u8),
        Keycode::G    => Some(0x9u8),
        Keycode::H    => Some(0xau8),
        Keycode::J    => Some(0xbu8),

        Keycode::V    => Some(0xcu8),
        Keycode::B    => Some(0xdu8),
        Keycode::N    => Some(0xeu8),
        Keycode::M    => Some(0xfu8),
        _ => None,
    };
    if let Some(key_index) = key_index {
        match state {
            KeyState::KeyDown => cpu.keydown(key_index),
            KeyState::KeyUp   => cpu.keyup(key_index),
        };
    }
}

pub fn load_rom(cpu: &mut Cpu){
    for argument in ::std::env::args().skip(1) {
        println!("Reading rom \"{}\" ...", argument);
        if let Err(s) = cpu.memory.load_rom(&argument) {
            panic!("Error reading {}: {}", &argument, s);
        }
    }
}
