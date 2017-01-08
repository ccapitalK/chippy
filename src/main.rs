extern crate byteorder;
extern crate rand;
extern crate sdl2;
mod cpu;
use cpu::Cpu;
mod mem;
mod io;

use sdl2::keyboard::Keycode;
use sdl2::event::Event;

//use std::fmt;
//use std::path;

fn main_loop(sdl_context: &sdl2::Sdl, renderer: &mut sdl2::render::Renderer, cpu: &mut Cpu) {
    let mut events_source = sdl_context.event_pump().unwrap();
    let instructions_per_second = 400u64;
    let mut total_instructions_executed = 0u64;

    io::load_rom(cpu);

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
                        Keycode::Backspace => {
                            cpu.reset();
                            io::load_rom(cpu);
                        },
                        keycode => io::parse_input(cpu, keycode, io::KeyState::KeyDown),
                    },
                    Event::KeyUp {keycode: Some(keycode), ..} => io::parse_input(cpu, keycode, io::KeyState::KeyUp),
                    _ => (),
                };
            }
            for _ in total_instructions_executed..instructions_by_frame_end {
                match cpu.exec_instruction() {
                    Err(v) => println!("Error in cpu.exec_instruction(): {}", v),
                    Ok(()) => (),
                }
            }
            io::draw_screen(renderer, &cpu);

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
    if std::env::args().count() != 2 {
        println!("Usage: chippy [rom file]");
        std::process::exit(1);
    }
    let sdl_context = sdl2::init().unwrap();
    let video_sybsys = sdl_context.video().unwrap();
    let window = video_sybsys.window("Chippy", io::WINDOW_WIDTH, io::WINDOW_HEIGHT)
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
