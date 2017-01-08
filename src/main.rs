extern crate byteorder;
extern crate rand;
extern crate sdl2;
mod cpu;
use cpu::Cpu;
use sdl2::messagebox;
use sdl2::pixels;
use sdl2::keyboard::Keycode;
use sdl2::event::Event;
mod mem;

//use std::fmt;
use std::path;

const VIEW_WIDTH:  u32 = 640;
const VIEW_HEIGHT: u32 = 320;
const WINDOW_WIDTH : u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

fn draw_screen(renderer: &mut sdl2::render::Renderer, cpu: &Cpu){
    renderer.set_draw_color(pixels::Color::RGB(0, 0, 0));
    renderer.clear();
    renderer.set_draw_color(pixels::Color::RGB(255, 255, 255));
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

            let square_rect = sdl2::rect::Rect::new(x0, y0, (x1-x0) as u32, (y1-y0) as u32);
            if cpu.memory.get_cell(x as u8, y as u8) {
                //println!("{:?}", square_rect);
                if let Err(v) = renderer.fill_rect(square_rect) {
                    panic!("Call to fill_rect({:?}) failed: {}", square_rect, v);
                }
            }

        }
    }
    renderer.present();
}

fn main_loop(sdl_context: &sdl2::Sdl, renderer: &mut sdl2::render::Renderer, cpu: &mut Cpu) {
    let mut events_source = sdl_context.event_pump().unwrap();
    let instructions_per_second = 100u64;
    let mut total_instructions_executed = 0u64;
    cpu.memory.draw_sprite(0u16, 5u8, 5u8, 5u8);
    cpu.memory.draw_sprite(0u16, 0u8, 20u8, 5u8);

    //write infinite loop to program memory
    cpu.memory.memset(0x200, &vec![0x12, 0x00]);

    // start timer subsystem
    let mut timer_subsys  = sdl_context.timer().unwrap();
    'main: loop {
        //timing stuff
        let start_frame_time = timer_subsys.ticks() as u64;
        let end_frame_time   = start_frame_time+16;
        let instructions_by_frame_end = (end_frame_time*instructions_per_second)/1000;

        {
            for event in events_source.poll_iter() {
                match event {
                    Event::Quit {..} => break 'main,
                    Event::KeyDown {keycode: Some(keycode), ..} => match keycode {
                        Keycode::Escape    => break 'main,
                        Keycode::Backspace => cpu.reset(),
                        _ => (),
                    },
                    _ => (),
                };
            }
            for _ in total_instructions_executed..instructions_by_frame_end {
                match cpu.exec_instruction() {
                    Err(v) => println!("Error in cpu.exec_instruction(): {}", v),
                    Ok(()) => (),
                }
            }
            draw_screen(renderer, &cpu);

        }

        //timing stuff
        total_instructions_executed = instructions_by_frame_end;
        let frame_time_elapsed = (timer_subsys.ticks() as u64)-start_frame_time;
        //println!("{}", frame_time_elapsed);
        timer_subsys.delay(16u32.saturating_sub(frame_time_elapsed as u32));
        cpu.decr_dt();
        cpu.decr_st();
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_sybsys = sdl_context.video().unwrap();
    let window = video_sybsys.window("Chippy", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut renderer = window.renderer().build().unwrap();
    let mut cpu = Cpu::new();

    //TODO: Remove this after figuring out what is causing first rect to not render
    let square_rect = sdl2::rect::Rect::new(10, 10, 10, 10);
    renderer.fill_rect(square_rect);
    renderer.present();

    main_loop(&sdl_context, &mut renderer, &mut cpu);
}
