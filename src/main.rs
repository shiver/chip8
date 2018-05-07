extern crate chip8;

extern crate byteorder;
extern crate num_traits;
#[macro_use]
extern crate enum_primitive_derive;
extern crate sdl2;

use std::env;
use std::fs::File;
use std::io::{Cursor, Error, Read, Write};
use std::time::{Duration, Instant};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

mod bitrange;
use bitrange::BitRange;

use chip8::cpu::CPU;
use chip8::instructions::Instruction;


const FRAME_TICK: Duration = Duration::from_millis(16);
const CPU_TICK: Duration = Duration::from_millis(1);

fn read_bin(filename: &String) -> Result<Vec<u8>, Error> {
    let mut file = File::open(filename)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("CHIP-8", 800, 400)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let filename = env::args().nth(1).expect("filename?");
    let mut big_endian = true;
    if let Some(nd) = env::args().nth(2) {
        big_endian = false;
    }

    let data = read_bin(&filename).unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut cpu = CPU::new(&data, Some(canvas));
    cpu.big_endian = big_endian;

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut cpu_last = Instant::now();
    let mut frame_last = Instant::now();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                Event::KeyDown {
                    keycode: Some(Keycode::Space), .. 
                } => cpu.keys[15] ^= 1,

                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => cpu.keys[4] = 1,
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => cpu.keys[6] = 1,
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => cpu.keys[8] = 1,
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => cpu.keys[2] = 1,
                Event::KeyDown {
                    keycode: Some(Keycode::Return),
                    ..
                } => cpu.keys[5] = 1,

                Event::KeyUp {
                    keycode: Some(Keycode::Left),
                    ..
                } => cpu.keys[4] = 0,
                Event::KeyUp {
                    keycode: Some(Keycode::Right),
                    ..
                } => cpu.keys[6] = 0,
                Event::KeyUp {
                    keycode: Some(Keycode::Up),
                    ..
                } => cpu.keys[8] = 0,
                Event::KeyUp {
                    keycode: Some(Keycode::Down),
                    ..
                } => cpu.keys[2] = 0,
                Event::KeyUp {
                    keycode: Some(Keycode::Return),
                    ..
                } => cpu.keys[5] = 0,

                _ => {}
            }
        }

        if cpu.keys[15] != 1 {
            if cpu_last.elapsed() >= CPU_TICK { 
                let raw = cpu.fetch_opcode().unwrap();
                if let Some(instruction) = Instruction::from_u16(&raw) {
                    cpu.do_instruction(&instruction);
                }

                cpu_last = Instant::now();
            }
        }

        if frame_last.elapsed() >= FRAME_TICK {
            cpu.show();
            frame_last = Instant::now();
        }
    }
}
