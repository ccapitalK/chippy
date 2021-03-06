extern crate argparse;
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

use argparse::{ArgumentParser, StoreTrue, Store};

//use std::fmt;
//use std::path;

fn main_loop(mut contexts: Contexts, cpu: &mut Cpu, file_name: &str) {
    let audio_context = contexts.sdl.audio().unwrap();
    let mut events_source = contexts.sdl.event_pump().unwrap();
    let num_frames = 0i64;
    let frames_per_second = 60i64;

    io::load_rom(cpu, file_name);

    // start timer subsystem
    let mut timer_subsys  = contexts.sdl.timer().unwrap();
    'main: loop {
        let instructions_per_second = cpu.get_ips() as i64;
        //timing stuff
        //Note: This will probably break pretty badly if the program starts lagging heavily
        let start_frame_time = timer_subsys.ticks() as i64;
        let frame_expected_time = 
            ((num_frames+1)*1000)/frames_per_second-(num_frames*1000)/frames_per_second;
        let end_frame_time = frame_expected_time + start_frame_time;
        let instructions_by_frame_end = (frame_expected_time*instructions_per_second)/1000;

        {
            for event in events_source.poll_iter() {
                match event {
                    Event::Quit {..} => break 'main,
                    Event::KeyDown {keycode: Some(keycode), ..} => match keycode {
                        // Exit emulator
                        Keycode::Escape    => break 'main,
                        // Increase emulator speed
                        Keycode::Q           => {
                            cpu.increase_ips();
                        }
                        // Decrease emulator speed
                        Keycode::A           => {
                            cpu.decrease_ips();
                        }
                        // Reset emulator
                        Keycode::Backspace => {
                            cpu.reset();
                            io::load_rom(cpu, file_name);
                        }
                        // Pass input into Chip8 io routine
                        keycode => io::parse_input(cpu, keycode, io::KeyState::KeyDown),
                    }
                    Event::KeyUp {keycode: Some(keycode), ..} => io::parse_input(cpu, keycode, io::KeyState::KeyUp),
                    _ => (),
                };
            }
            for _ in 0..instructions_by_frame_end {
                match cpu.exec_instruction() {
                    Err(v) => println!("Error in cpu.exec_instruction(): {}", v),
                    Ok(()) => (),
                }
            }
            io::draw_screen(&mut contexts, &cpu);

        }

        //timing stuff
        let current_time = timer_subsys.ticks();
        //println!("{}", frame_time_elapsed);
        timer_subsys.delay((end_frame_time as u32) - current_time);
        cpu.decr_dt();
        cpu.decr_st();
    }
}

fn main() {
    let mut instructions_per_second = cpu::DEFAULT_INS_PER_SECOND;
    let mut file_name = String::new();

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Chip8 emulator");
        ap.refer(&mut instructions_per_second)
            .add_option(&["-f", "--freq"], Store, "Instructions per second");
        ap.refer(&mut file_name)
            .add_argument("<Rom File>", Store, "Name of rom file")
            .required();
        ap.parse_args_or_exit();
    }
    let mut cpu = Cpu::new();
    cpu.set_ips(instructions_per_second);

    sdl::with_contexts(move |contexts| main_loop(contexts, &mut cpu, &file_name));
}
