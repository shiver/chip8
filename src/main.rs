extern crate chip8;
extern crate sdl2;
#[macro_use] extern crate failure;

use std::env;
use std::time::{Duration, Instant};
use std::process::exit;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

pub use failure::{Error, Fail, err_msg};
use chip8::{read_binary, Context};
use chip8::cpu::CPU;
use chip8::instructions::Instruction;

const FRAME_TICK: Duration = Duration::from_millis(16);
const CPU_TICK: Duration = Duration::from_millis(1);

fn init_canvas(context: &mut Context) -> Result<&Context, Error> {
    let sdl_context = match sdl2::init() {
        Ok(v) => v,
        Err(s) => return Err(err_msg(s))
    };

    let video_subsystem = match sdl_context.video() {
        Ok(v) => v,
        Err(s) => return Err(err_msg(s))
    };

    let window = video_subsystem
        .window("CHIP-8", 800, 400)
        .position_centered()
        .build()?;

    context.sdl_context = Some(sdl_context);
    context.canvas = Some(window.into_canvas().build()?);

    Ok(context)
}

fn init_event_subsystem(context: &mut Context) -> Result<&Context, Error> {
    if let Some(ref sdl_context) = context.sdl_context {
        context.events = match sdl_context.event_pump() {
            Ok(v) => Some(v),
            Err(s) => return Err(err_msg(s))
        };

        Ok(context)
    } else {
        Err(format_err!("SDL context should have been available, but it wasn't!"))
    }
}

fn main() {
    let mut context = Context{canvas: None, grid: vec![0; 2046], key_map: [0; 16], events: None, sdl_context: None};
    init_canvas(&mut context);
    init_event_subsystem(&mut context);

    let filename = env::args().nth(1).expect("filename?");

    let data = match read_binary(&filename) {
        Ok(data) => data,
        Err(e) => {
            println!("Error reading binary \"{}\": {}", filename, e);
            exit(1);
        }
    };

    let mut cpu = CPU::new(&data, context.canvas);

    let mut event_pump = context.events.expect("Event subsystem should have been available, but it wasn't!");

    let mut cpu_last = Instant::now();
    let mut frame_last = Instant::now();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,

                Event::KeyDown { keycode: Some(Keycode::Space), .. } => cpu.keys[15] ^= 1,

                Event::KeyDown { keycode: Some(Keycode::Left), .. } => cpu.keys[4] = 1,
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => cpu.keys[6] = 1,
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => cpu.keys[8] = 1,
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => cpu.keys[2] = 1,
                Event::KeyDown { keycode: Some(Keycode::Return), .. } => cpu.keys[5] = 1,

                Event::KeyUp { keycode: Some(Keycode::Left), .. } => cpu.keys[4] = 0,
                Event::KeyUp { keycode: Some(Keycode::Right), .. } => cpu.keys[6] = 0,
                Event::KeyUp { keycode: Some(Keycode::Up), .. } => cpu.keys[8] = 0,
                Event::KeyUp { keycode: Some(Keycode::Down), .. } => cpu.keys[2] = 0,
                Event::KeyUp { keycode: Some(Keycode::Return), .. } => cpu.keys[5] = 0,

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
