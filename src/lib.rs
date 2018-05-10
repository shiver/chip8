extern crate byteorder;
extern crate rand;
extern crate sdl2;
extern crate failure;

mod bitrange;
pub mod instructions;
pub mod cpu;

use std::fs::File;
use std::io::Read;
use std::time::Duration;

pub use failure::{Error, Fail};

use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::EventPump;
use sdl2::Sdl;

pub const FRAME_TICK: Duration = Duration::from_millis(16);
pub const CPU_TICK: Duration = Duration::from_millis(2);

pub const FONT4X5: [u8; 80] = [
    0xf0, 0x90, 0x90, 0x90, 0xf0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xf0, 0x10, 0xf0, 0x80, 0xf0, // 2
    0xf0, 0x10, 0xf0, 0x10, 0xf0, // 3
    0x90, 0x90, 0xf0, 0x10, 0x10, // 4
    0xf0, 0x80, 0xf0, 0x10, 0xf0, // 5
    0xf0, 0x80, 0xf0, 0x90, 0xf0, // 6
    0xf0, 0x10, 0x20, 0x40, 0x40, // 7
    0xf0, 0x90, 0xf0, 0x90, 0xf0, // 8
    0xf0, 0x90, 0xf0, 0x10, 0xf0, // 9
    0xf0, 0x90, 0xf0, 0x90, 0x90, // A
    0xe0, 0x90, 0xe0, 0x90, 0xe0, // B
    0xf0, 0x80, 0x80, 0x80, 0x80, // C
    0xe0, 0x90, 0x90, 0x90, 0xe0, // D
    0xf0, 0x80, 0xf0, 0x80, 0xf0, // E
    0xf0, 0x80, 0xf0, 0x80, 0x80, // F
];

pub struct Context {
    pub sdl_context: Option<Sdl>,
    pub canvas: Option<Canvas<Window>>,
    pub events: Option<EventPump>,
    pub grid: Vec<u8>,
    pub key_map: [u8; 16],
}


pub fn read_binary(filename: &String) -> Result<Vec<u8>, Error> {
    let mut file = File::open(filename)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}
