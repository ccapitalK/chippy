extern crate byteorder;
extern crate rand;
extern crate sdl2;
mod cpu;
use cpu::Cpu;
mod mem;
mod io;
mod sdl;

use sdl2::keyboard::Keycode;
use sdl2::event::Event;
use sdl::Contexts;

//use std::fmt;
//use std::path;

fn main_loop(mut contexts: Contexts, cpu: &mut Cpu) {
    let mut events_source = contexts.sdl.event_pump().unwrap();
    let mut total_instructions_executed = 0u64;
        

    io::load_rom(cpu);

    // start timer subsystem
    let mut timer_subsys  = contexts.sdl.timer().unwrap();
    'main: loop {
        let instructions_per_second = cpu.get_ips();
        //timing stuff
        //Note: This will probably break pretty badly if the program starts lagging heavily
        let start_frame_time = timer_subsys.ticks() as u64;
        let end_frame_time   = start_frame_time+16;
        let instructions_by_frame_end = (end_frame_time*instructions_per_second)/1000;

        {
            for event in events_source.poll_iter() {
                match event {
                    Event::Quit {..} => break 'main,
                    Event::KeyDown {keycode: Some(keycode), ..} => match keycode {
                        // Exit emulator
                        Keycode::Escape    => break 'main,
                        // Increase emulator speed
                        Keycode::KpPlus      => {
                            cpu.increase_ips();
                        }
                        // Decrease emulator speed
                        Keycode::KpMinus     => {
                            cpu.decrease_ips();
                        }
                        Keycode::Backspace => {
                            cpu.reset();
                            io::load_rom(cpu);
                        }
                        // Pass input into Chip8 io routine
                        keycode => io::parse_input(cpu, keycode, io::KeyState::KeyDown),
                    }
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
            io::draw_screen(&mut contexts, &cpu);

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
    let mut cpu = Cpu::new();

    sdl::with_contexts(move |contexts| main_loop(contexts, &mut cpu));
}
