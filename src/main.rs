extern crate byteorder;
extern crate num_traits;
#[macro_use]
extern crate enum_primitive_derive;
extern crate rand;
extern crate sdl2;

use std::env;
use std::fs::File;
use std::io::{Cursor, Error, Read, Write};
use std::time::Duration;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

mod bitrange;
use bitrange::BitRange;

const FONT_SET: [u8; 80] = [
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

type GPR = u8;
type Address = u16;

type HalfWord = u8;
type Word = u16;

struct CPU {
    regs: [u8; 16],
    address: u16,
    stack: Vec<usize>,
    memory: Cursor<Vec<u8>>,

    delay_timer: u8,
    sound_timer: u8,

    pc: usize,

    keys: [u8; 16],
    display: Option<Canvas<Window>>,
    grid: Vec<u8>,
    big_endian: bool,
}

#[derive(Debug, PartialEq)]
enum Instruction {
    ClearDisplay,
    Return,
    JumpToAddress(Address),
    CallSubroutine(Address),
    SkipIfEqual(GPR, HalfWord),
    SkipIfNotEqual(GPR, HalfWord),
    SkipIfEqualRegister(GPR, HalfWord),
    LoadConst(GPR, HalfWord),
    AddConst(GPR, HalfWord),
    AssignValue(GPR, GPR),
    SetOr(GPR, GPR),
    SetAnd(GPR, GPR),
    SetXor(GPR, GPR),
    Add(GPR, GPR),
    Subtract(GPR, GPR),
    ShiftRight(GPR, GPR),
    Reduce(GPR, GPR),
    ShiftLeft(GPR, GPR),
    SkipIfNotEqualRegister(GPR, GPR),
    SetMemoryAddress(Address),
    JumpToV0Address(Address),
    BitwiseRandom(GPR, HalfWord),
    DrawSprite(GPR, GPR, HalfWord),
    SkipIfPressed(GPR),
    SkipIfNotPressed(GPR),
    LoadDelay(GPR),
    WaitForPress(GPR),
    SetDelay(GPR),
    SetSound(GPR),
    AddOffset(GPR),
    SetMemoryForFont(GPR),
    SetBCD(GPR),
    DumpReg(GPR),
    LoadReg(GPR),
}

fn first(value: &u16) -> u8 {
    value.range_u8(12..15)
}

fn second(value: &u16) -> u8 {
    value.range_u8(8..11)
}

fn third(value: &u16) -> u8 {
    value.range_u8(4..7)
}

fn last(value: &u16) -> u8 {
    value.range_u8(0..3)
}

fn last_two(value: &u16) -> u8 {
    value.range_u8(0..7)
}

fn last_three(value: &u16) -> u16 {
    value.range_u16(0..11)
}

impl Instruction {
    fn from_u16(value: &u16) -> Option<Instruction> {
        match first(&value) {
            0x0 => match last_two(&value) {
                0xE0 => Some(Instruction::ClearDisplay),
                0xEE => Some(Instruction::Return),
                _ => None,
            },
            0x1 => Some(Instruction::JumpToAddress(last_three(&value))),
            0x2 => Some(Instruction::CallSubroutine(last_three(&value))),
            0x3 => Some(Instruction::SkipIfEqual(second(&value), last_two(&value))),
            0x4 => Some(Instruction::SkipIfNotEqual(
                second(&value),
                last_two(&value),
            )),
            0x5 => Some(Instruction::SkipIfEqualRegister(
                second(&value),
                third(&value),
            )),
            0x6 => Some(Instruction::LoadConst(second(&value), last_two(&value))),
            0x7 => Some(Instruction::AddConst(second(&value), last_two(&value))),
            0x8 => match last(&value) {
                0x0 => Some(Instruction::AssignValue(second(&value), third(&value))),
                0x1 => Some(Instruction::SetOr(second(&value), third(&value))),
                0x2 => Some(Instruction::SetAnd(second(&value), third(&value))),
                0x3 => Some(Instruction::SetXor(second(&value), third(&value))),
                0x4 => Some(Instruction::Add(second(&value), third(&value))),
                0x5 => Some(Instruction::Subtract(second(&value), third(&value))),
                0x6 => Some(Instruction::ShiftRight(second(&value), third(&value))),
                0x7 => Some(Instruction::Reduce(second(&value), third(&value))),
                0xE => Some(Instruction::ShiftLeft(second(&value), third(&value))),
                _ => None,
            },
            0x9 => Some(Instruction::SkipIfNotEqualRegister(
                second(&value),
                third(&value),
            )),
            0xA => Some(Instruction::SetMemoryAddress(last_three(&value))),
            0xB => Some(Instruction::JumpToV0Address(last_three(&value))),
            0xC => Some(Instruction::BitwiseRandom(second(&value), last_two(&value))),
            0xD => Some(Instruction::DrawSprite(
                second(&value),
                third(&value),
                last(&value),
            )),
            0xE => match last_two(&value) {
                0x9E => Some(Instruction::SkipIfPressed(second(&value))),
                0xA1 => Some(Instruction::SkipIfNotPressed(second(&value))),
                _ => None,
            },
            0xF => match last_two(&value) {
                0x07 => Some(Instruction::LoadDelay(second(&value))),
                0x0A => Some(Instruction::WaitForPress(second(&value))),
                0x15 => Some(Instruction::SetDelay(second(&value))),
                0x18 => Some(Instruction::SetSound(second(&value))),
                0x1E => Some(Instruction::AddOffset(second(&value))),
                0x29 => Some(Instruction::SetMemoryForFont(second(&value))),
                0x33 => Some(Instruction::SetBCD(second(&value))),
                0x55 => Some(Instruction::DumpReg(second(&value))),
                0x65 => Some(Instruction::LoadReg(second(&value))),
                _ => None,
            },

            _ => None,
        }
    }
}

fn read_bin(filename: &String) -> Result<Vec<u8>, Error> {
    let mut file = File::open(filename)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

impl CPU {
    fn new(data: &Vec<u8>, display: Option<Canvas<Window>>) -> CPU {
        let mut memory = vec![0; 4096];
        for i in 0..data.len() {
            memory[0x200 + i] = data[i];
        }

        for i in 0..FONT_SET.len() {
            memory[0x0 + i] = FONT_SET[i];
        }

        CPU {
            regs: [0; 16],
            address: 0,
            stack: vec![],
            memory: Cursor::new(memory),
            delay_timer: 0,
            sound_timer: 0,
            pc: 0x200,
            keys: [0; 16],
            display: display,
            grid: vec![0; 2048],
            big_endian: true,
        }
    }

    fn show(&mut self) {
        if let Some(ref mut canvas) = self.display {
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            canvas.set_draw_color(Color::RGB(255, 255, 255));

            for y in 0..32 {
                for x in 0..64 {
                    if self.grid[(y*64) + x] == 1 {
                        canvas
                            .fill_rect(Rect::new(
                                (x as u8) as i32 * 10,
                                (y as u8) as i32 * 10,
                                10,
                                10,
                            ))
                            .unwrap();
                    }
                }
            }

            canvas.present();
        }
    }

    fn clear(&mut self) {
        for idx in 0..2046 {
            self.grid[idx] = 0;
        }
    }

    fn inc_pc(&mut self) {
        self.pc += 2;
    }

    fn fetch_opcode(&mut self) -> Result<u16, std::io::Error> {
        self.memory.set_position(self.pc as u64);
        if self.big_endian {
            self.memory.read_u16::<BigEndian>()
        } else {
            self.memory.read_u16::<LittleEndian>()
        }
    }

    fn timer_tick(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn do_instruction(&mut self, instruction: &Instruction) {
        let mut should_increment = true;
        self.timer_tick();

        match instruction {
            Instruction::ClearDisplay => {
                self.clear();
            }

            Instruction::Return => {
                if let Some(ret) = self.stack.pop() {
                    self.pc = ret as usize;
                    should_increment = true;
                };
            }

            Instruction::JumpToAddress(address) => {
                self.pc = *address as usize;
                should_increment = false;
            }

            Instruction::CallSubroutine(address) => {
                self.stack.push(self.pc);
                self.pc = *address as usize;
                should_increment = false;
            }

            Instruction::SkipIfEqual(vx, value) => {
                if self.regs[*vx as usize] == *value {
                    self.inc_pc();
                }
            }

            Instruction::SkipIfNotEqual(vx, value) => {
                if self.regs[*vx as usize] != *value {
                    self.inc_pc();
                }
            }

            Instruction::SkipIfEqualRegister(vx, vy) => {
                if self.regs[*vx as usize] == self.regs[*vy as usize] {
                    self.inc_pc();
                }
            }

            Instruction::LoadConst(vx, value) => {
                self.regs[*vx as usize] = *value;
            }

            Instruction::AddConst(vx, value) => {
                let idx = *vx as usize;
                self.regs[idx] = self.regs[idx].wrapping_add(*value);
            }

            Instruction::AssignValue(vx, vy) => {
                self.regs[*vx as usize] = self.regs[*vy as usize];
            }

            Instruction::SetOr(vx, vy) => {
                self.regs[*vx as usize] |= self.regs[*vy as usize];
            }

            Instruction::SetAnd(vx, vy) => {
                self.regs[*vx as usize] &= self.regs[*vy as usize];
            }

            Instruction::SetXor(vx, vy) => {
                self.regs[*vx as usize] ^= self.regs[*vy as usize];
            }

            Instruction::Add(vx, vy) => {
                let x = self.regs[*vx as usize];
                let y = self.regs[*vy as usize];

                let ret = x as u16 + y as u16;
                self.regs[*vx as usize] = x.wrapping_add(y);
                self.regs[0xF] = (ret > 255) as u8;
            }

            Instruction::Subtract(vx, vy) => {
                let x = self.regs[*vx as usize];
                let y = self.regs[*vy as usize];

                self.regs[0xF] = (x > y) as u8;
                self.regs[*vx as usize] = x.wrapping_sub(y);
            }

            Instruction::ShiftRight(vx, vy) => {
                self.regs[0xF] = self.regs[*vy as usize] & 1;
                self.regs[*vx as usize] = self.regs[*vy as usize] >> 1;
            }

            Instruction::Reduce(vx, vy) => {
                let x = self.regs[*vx as usize];
                let y = self.regs[*vy as usize];

                self.regs[*vx as usize] = y.wrapping_sub(x);
                self.regs[0xF] = (y > x) as u8;
            }

            Instruction::ShiftLeft(vx, vy) => {
                self.regs[0xF] = self.regs[*vy as usize] >> 7;
                self.regs[*vx as usize] = self.regs[*vy as usize] << 1;
            }

            Instruction::SkipIfNotEqualRegister(vx, vy) => {
                if self.regs[*vx as usize] != self.regs[*vy as usize] {
                    self.inc_pc();
                }
            }

            Instruction::SetMemoryAddress(address) => {
                self.address = *address;
            }

            Instruction::JumpToV0Address(address) => {
                self.pc = (*address + self.regs[0] as u16) as usize;
                should_increment = false;
            }

            Instruction::BitwiseRandom(vx, value) => {
                self.regs[*vx as usize] = rand::random::<u8>() & *value;
            }

            Instruction::DrawSprite(vx, vy, height) => {
                if let Some(ref mut canvas) = self.display {
                    let start_x = self.regs[*vx as usize];
                    let start_y = self.regs[*vy as usize];
                    let height = *height;
                    self.regs[0xf] = 0;


                    for y in 0..height { 
                        self.memory.set_position((self.address + y as u16) as u64);
                        let row = self.memory.read_u8().unwrap();

                        for x in 0..8 {
                            let grid_pos = (((y as u16 + start_y as u16) * 64) + (x as u16 + start_x as u16)) as usize;
                            if (row >> 7-x) & 1 != 0 {
                                self.regs[0xf] = (self.grid[grid_pos] == 1) as u8;
                                self.grid[grid_pos] ^= 1;
                            }
                        }
                    }
                }
            }

            Instruction::SkipIfPressed(vx) => {
                let key = self.regs[*vx as usize] as usize;
                if self.keys[key] == 1 {
                    self.inc_pc();
                }
            }
            Instruction::SkipIfNotPressed(vx) => {
                let key = self.regs[*vx as usize] as usize;
                if self.keys[key] != 1 {
                    self.inc_pc();
                }
            }

            Instruction::LoadDelay(vx) => {
                self.regs[*vx as usize] = self.delay_timer;
            }

            Instruction::WaitForPress(vx) => {
                let key = self.regs[*vx as usize] as usize;
                if self.keys[key] != 1 {
                    should_increment = false;
                }
            }

            Instruction::SetDelay(vx) => {
                self.delay_timer = self.regs[*vx as usize];
            }

            Instruction::SetSound(vx) => {
                self.sound_timer = self.regs[*vx as usize];
            }

            Instruction::AddOffset(vx) => {
                self.address += self.regs[*vx as usize] as u16;
            }

            Instruction::SetMemoryForFont(vx) => {
                self.address = (self.regs[*vx as usize] * 5) as u16;
            }

            Instruction::SetBCD(vx) => {
                let val = self.regs[*vx as usize];

                let h = val / 100;
                let t = (val / 10) % 10;
                let d = (val % 100) % 10;
                self.memory.set_position(self.address as u64);
                self.memory.write(&[h, t, d]);
            }

            Instruction::DumpReg(vx) => {
                for idx in 0..*vx + 1 {
                    self.memory.set_position(self.address as u64);
                    self.memory.write(&[self.regs[idx as usize]]);
                    self.address += 1;
                }
            }

            Instruction::LoadReg(vx) => {
                for idx in 0..*vx + 1 {
                    self.memory.set_position((self.address as u16) as u64);
                    self.regs[idx as usize] = self.memory.read_u8().unwrap();
                    self.address += 1;
                }
            }

            _ => (),
        }

        if should_increment {
            self.inc_pc();
        }
    }
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

    // use byteorder::WriteBytesExt;
    // let mut data: Vec<u8> = vec![];
    // data.write_u16::<BigEndian>(0x2204);
    // data.write_u16::<BigEndian>(0x1200);

    // data.write_u16::<BigEndian>(0x600a);
    // data.write_u16::<BigEndian>(0x6138);
    // data.write_u16::<BigEndian>(0x6218);
    
    // data.write_u16::<BigEndian>(0xf029);
    // data.write_u16::<BigEndian>(0xd125);

    // // data.write_u16::<BigEndian>(0x6001);
    // // data.write_u16::<BigEndian>(0xf029);
    // // data.write_u16::<BigEndian>(0xda05);

    // data.write_u16::<BigEndian>(0x00ee);
    // println!("{:?}", data);

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut cpu = CPU::new(&data, Some(canvas));
    cpu.big_endian = big_endian;

    let mut event_pump = sdl_context.event_pump().unwrap();

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
        ::std::thread::sleep(Duration::from_millis(2));

        if cpu.keys[15] != 1 {
            let raw = cpu.fetch_opcode().unwrap();
            if let Some(instruction) = Instruction::from_u16(&raw) {
                // println!("{:#x} {:#x} {:#?}", cpu.pc, raw, &instruction);
                cpu.do_instruction(&instruction);
            } else {
                println!("{:#x} {:#x}", cpu.pc, raw);
            }
        }
        cpu.show();
        // canvas.set_draw_color(Color::RGB(0, 0, 0));
        // The rest of the game loop goes here...
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};

    #[test]
    fn test_opcode_to_instruction() {
        assert_eq!(
            Instruction::from_u16(&0x00e0),
            Some(Instruction::ClearDisplay)
        );
        assert_eq!(Instruction::from_u16(&0x00ee), Some(Instruction::Return));
        assert_eq!(
            Instruction::from_u16(&0x1123),
            Some(Instruction::JumpToAddress(0x0123))
        );
        assert_eq!(
            Instruction::from_u16(&0x2234),
            Some(Instruction::CallSubroutine(0x0234))
        );
        assert_eq!(
            Instruction::from_u16(&0x3345),
            Some(Instruction::SkipIfEqual(0x3, 0x45))
        );
        assert_eq!(
            Instruction::from_u16(&0x4456),
            Some(Instruction::SkipIfNotEqual(0x4, 0x56))
        );
        assert_eq!(
            Instruction::from_u16(&0x5560),
            Some(Instruction::SkipIfEqualRegister(0x5, 0x6))
        );
        assert_eq!(
            Instruction::from_u16(&0x6678),
            Some(Instruction::LoadConst(0x6, 0x78))
        );
        assert_eq!(
            Instruction::from_u16(&0x7789),
            Some(Instruction::AddConst(0x7, 0x89))
        );

        assert_eq!(
            Instruction::from_u16(&0x8890),
            Some(Instruction::AssignValue(0x8, 0x9))
        );
        assert_eq!(
            Instruction::from_u16(&0x8891),
            Some(Instruction::SetOr(0x8, 0x9))
        );
        assert_eq!(
            Instruction::from_u16(&0x8892),
            Some(Instruction::SetAnd(0x8, 0x9))
        );
        assert_eq!(
            Instruction::from_u16(&0x8893),
            Some(Instruction::SetXor(0x8, 0x9))
        );
        assert_eq!(
            Instruction::from_u16(&0x8894),
            Some(Instruction::Add(0x8, 0x9))
        );
        assert_eq!(
            Instruction::from_u16(&0x8895),
            Some(Instruction::Subtract(0x8, 0x9))
        );
        assert_eq!(
            Instruction::from_u16(&0x8896),
            Some(Instruction::ShiftRight(0x8, 0x9))
        );
        assert_eq!(
            Instruction::from_u16(&0x8897),
            Some(Instruction::Reduce(0x8, 0x9))
        );
        assert_eq!(
            Instruction::from_u16(&0x889e),
            Some(Instruction::ShiftLeft(0x8, 0x9))
        );

        assert_eq!(
            Instruction::from_u16(&0x9910),
            Some(Instruction::SkipIfNotEqualRegister(0x9, 0x1))
        );
        assert_eq!(
            Instruction::from_u16(&0xabcd),
            Some(Instruction::SetMemoryAddress(0xbcd))
        );
        assert_eq!(
            Instruction::from_u16(&0xbcde),
            Some(Instruction::JumpToV0Address(0xcde))
        );
        assert_eq!(
            Instruction::from_u16(&0xcdef),
            Some(Instruction::BitwiseRandom(0xd, 0xef))
        );
        assert_eq!(
            Instruction::from_u16(&0xd12f),
            Some(Instruction::DrawSprite(0x1, 0x2, 0xf))
        );
        assert_eq!(
            Instruction::from_u16(&0xef9e),
            Some(Instruction::SkipIfPressed(0xf))
        );
        assert_eq!(
            Instruction::from_u16(&0xeaa1),
            Some(Instruction::SkipIfNotPressed(0xa))
        );

        assert_eq!(
            Instruction::from_u16(&0xf107),
            Some(Instruction::LoadDelay(0x1))
        );
        assert_eq!(
            Instruction::from_u16(&0xf20a),
            Some(Instruction::WaitForPress(0x2))
        );
        assert_eq!(
            Instruction::from_u16(&0xf315),
            Some(Instruction::SetDelay(0x3))
        );
        assert_eq!(
            Instruction::from_u16(&0xf418),
            Some(Instruction::SetSound(0x4))
        );
        assert_eq!(
            Instruction::from_u16(&0xf51e),
            Some(Instruction::AddOffset(0x5))
        );
        assert_eq!(
            Instruction::from_u16(&0xf629),
            Some(Instruction::SetMemoryForFont(0x6))
        );
        assert_eq!(
            Instruction::from_u16(&0xf733),
            Some(Instruction::SetBCD(0x7))
        );
        assert_eq!(
            Instruction::from_u16(&0xf855),
            Some(Instruction::DumpReg(0x8))
        );
        assert_eq!(
            Instruction::from_u16(&0xf965),
            Some(Instruction::LoadReg(0x9))
        );
    }

    #[test]
    fn test_clear_and_basics() {
        let mut data = [0; 2];
        BigEndian::write_u16(&mut data, 0x00e0);
        let mut cpu = CPU::new(&data.to_vec(), None);
        assert_eq!(cpu.pc, 0x200);

        cpu.stack.push(0x300);

        let raw = cpu.fetch_opcode().unwrap();
        assert_eq!(raw, 0x00e0);

        let instruction = Instruction::from_u16(&raw);
        assert_eq!(instruction, Some(Instruction::ClearDisplay));
        cpu.do_instruction(&instruction.unwrap());
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_return() {
        let mut cpu = CPU::new(&vec![], None);
        cpu.stack.push(0x400);
        cpu.do_instruction(&Instruction::Return);
        assert_eq!(cpu.stack.len(), 0);
        assert_eq!(cpu.pc, 0x402);
    }

    #[test]
    fn test_jump_to_address() {
        let mut cpu = CPU::new(&vec![], None);
        cpu.do_instruction(&Instruction::JumpToAddress(0x412));
        assert_eq!(cpu.pc, 0x412);
    }

    #[test]
    fn test_call_subroutine() {
        let mut cpu = CPU::new(&vec![], None);
        cpu.pc = 0x655;
        cpu.do_instruction(&Instruction::CallSubroutine(0x595));
        assert_eq!(cpu.stack, vec![0x655]);
        assert_eq!(cpu.pc, 0x595);
    }

    #[test]
    fn test_skip_if_equal() {
        let mut cpu = CPU::new(&vec![], None);
        cpu.pc = 0x655;
        cpu.regs[0x5] = 0x23;
        cpu.do_instruction(&Instruction::SkipIfEqual(0x5, 0x23));
        assert_eq!(cpu.pc, 0x659);

        cpu.regs[0x5] = 0x24;
        cpu.do_instruction(&Instruction::SkipIfEqual(0x5, 0x23));
        assert_eq!(cpu.pc, 0x65B);
    }

    #[test]
    fn test_skip_if_not_equal() {
        let mut cpu = CPU::new(&vec![], None);
        cpu.pc = 0x655;
        cpu.regs[0x5] = 0x24;
        cpu.do_instruction(&Instruction::SkipIfEqual(0x5, 0x23));
        assert_eq!(cpu.pc, 0x657);

        cpu.regs[0x5] = 0x23;
        cpu.do_instruction(&Instruction::SkipIfEqual(0x5, 0x23));
        assert_eq!(cpu.pc, 0x65B);
    }

    #[test]
    fn test_skip_if_equal_register() {
        let mut cpu = CPU::new(&vec![], None);
        cpu.pc = 0x655;
        cpu.regs[0x5] = 0x14;
        cpu.regs[0x6] = 0x14;
        cpu.do_instruction(&Instruction::SkipIfEqualRegister(0x5, 0x6));
        assert_eq!(cpu.pc, 0x659);

        cpu.pc = 0x655;
        cpu.regs[0x5] = 0x04;
        cpu.regs[0x6] = 0x14;
        cpu.do_instruction(&Instruction::SkipIfEqualRegister(0x5, 0x6));
        assert_eq!(cpu.pc, 0x657);
    }

    #[test]
    fn test_load_const() {
        let mut cpu = CPU::new(&vec![], None);
        cpu.regs[0xe] = 0x0;
        cpu.do_instruction(&Instruction::LoadConst(0xe, 0x6A));
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.regs[0xe], 0x6a);
    }

    #[test]
    fn test_add_const() {
        let mut cpu = CPU::new(&vec![], None);
        cpu.regs[0x1] = 0x12;
        cpu.do_instruction(&Instruction::AddConst(0x1, 0x13));
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.regs[0x1], 0x25);
    }

    #[test]
    fn test_assign_value() {
        let mut cpu = CPU::new(&vec![], None);
        cpu.regs[0x2] = 0xff;
        cpu.regs[0x3] = 0xaa;
        cpu.do_instruction(&Instruction::AssignValue(0x2, 0x3));
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.regs[0x2], 0xaa);

        cpu.regs[0x2] = 0xff;
        cpu.do_instruction(&Instruction::AssignValue(0x3, 0x2));
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.regs[0x3], 0xff);
    }

    #[test]
    fn test_set_or() {
        let mut cpu = CPU::new(&vec![], None);
        cpu.regs[0x2] = 0x01;
        cpu.regs[0x3] = 0x03;
        cpu.do_instruction(&Instruction::SetOr(0x2, 0x3));
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.regs[0x2], 0x03);
        assert_eq!(cpu.regs[0x3], 0x03);
    }

    #[test]
    fn test_set_and() {
        let mut cpu = CPU::new(&vec![], None);
        cpu.regs[0x2] = 0b11;
        cpu.regs[0x3] = 0b10;
        cpu.do_instruction(&Instruction::SetAnd(0x2, 0x3));
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.regs[0x2], 0b10);
        assert_eq!(cpu.regs[0x3], 0b10);
    }

    #[test]
    fn test_add() {
        let mut cpu = CPU::new(&vec![], None);
        cpu.regs[0x2] = 253;
        cpu.regs[0x3] = 1;
        cpu.do_instruction(&Instruction::Add(0x2, 0x3));
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.regs[0x2], 254);
        assert_eq!(cpu.regs[0xf], 0);

        cpu.regs[0x2] = 254;
        cpu.regs[0x3] = 3;
        cpu.do_instruction(&Instruction::Add(0x2, 0x3));
        assert_eq!(cpu.regs[0x2], 1);
        assert_eq!(cpu.regs[0xf], 1);
    }

    #[test]
    fn test_subtract() {
        let mut cpu = CPU::new(&vec![], None);
        cpu.regs[0x2] = 2;
        cpu.regs[0x3] = 1;
        cpu.do_instruction(&Instruction::Subtract(0x2, 0x3));
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.regs[0x2], 1);
        assert_eq!(cpu.regs[0xf], 1);

        cpu.regs[0x2] = 1;
        cpu.regs[0x3] = 2;
        cpu.do_instruction(&Instruction::Subtract(0x2, 0x3));
        assert_eq!(cpu.regs[0x2], 255);
        assert_eq!(cpu.regs[0xf], 0);
    }

    #[test]
    fn test_shift_right() {
        let mut cpu = CPU::new(&vec![], None);
        cpu.regs[0x4] = 0b00000000; 
        cpu.regs[0x5] = 0b11101110;
        cpu.do_instruction(&Instruction::ShiftRight(0x4, 0x5));
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.regs[0x4], 0b01110111);
        assert_eq!(cpu.regs[0x5], 0b11101110);
        assert_eq!(cpu.regs[0xf], 0);

        cpu.regs[0x4] = 0b00000000; 
        cpu.regs[0x5] = 0b01110111;
        cpu.do_instruction(&Instruction::ShiftRight(0x4, 0x5));
        assert_eq!(cpu.regs[0x4], 0b00111011);
        assert_eq!(cpu.regs[0x5], 0b01110111);
        assert_eq!(cpu.regs[0xf], 1);
    }

    #[test]
    fn test_reduce() {
        let mut cpu = CPU::new(&vec![], None);
        cpu.regs[0x4] = 1; 
        cpu.regs[0x5] = 243;
        cpu.do_instruction(&Instruction::Reduce(0x4, 0x5));
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.regs[0x4], 242);
        assert_eq!(cpu.regs[0x5], 243);
        assert_eq!(cpu.regs[0xf], 1);

        cpu.regs[0x4] = 3; 
        cpu.regs[0x5] = 2;
        cpu.do_instruction(&Instruction::Reduce(0x4, 0x5));
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.regs[0x4], 255);
        assert_eq!(cpu.regs[0x5], 2);
        assert_eq!(cpu.regs[0xf], 0);
    }

    #[test]
    fn test_shift_left() {
        let mut cpu = CPU::new(&vec![], None);
        cpu.regs[0x4] = 0b00000000; 
        cpu.regs[0x5] = 0b11101110;
        cpu.do_instruction(&Instruction::ShiftLeft(0x4, 0x5));
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.regs[0x4], 0b11011100);
        assert_eq!(cpu.regs[0x5], 0b11101110);
        assert_eq!(cpu.regs[0xf], 1);

        cpu.regs[0x4] = 0b00000000; 
        cpu.regs[0x5] = 0b01110111;
        cpu.do_instruction(&Instruction::ShiftLeft(0x4, 0x5));
        assert_eq!(cpu.regs[0x4], 0b11101110);
        assert_eq!(cpu.regs[0x5], 0b01110111);
        assert_eq!(cpu.regs[0xf], 0);
    }

    #[test]
    fn test_set_memory_address() {
        let mut cpu = CPU::new(&vec![], None);
        assert_eq!(cpu.address, 0x0);
        assert_eq!(cpu.pc, 0x200);
        cpu.do_instruction(&Instruction::SetMemoryAddress(0x2b4));
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.address, 0x2b4);
    }

    #[test]
    fn test_set_bcd() {
        let mut cpu = CPU::new(&vec![], None);
        cpu.address = 0x0300;
        cpu.regs[0x0] = 129;
        cpu.do_instruction(&Instruction::SetBCD(0x0));
        let raw = cpu.memory.clone().into_inner();
        assert_eq!(raw[0x300], 1);
        assert_eq!(raw[0x301], 2);
        assert_eq!(raw[0x302], 9);
        assert_eq!(raw[0x303], 0);

        cpu.address = 0x0400;
        cpu.regs[0x1] = 19;
        cpu.do_instruction(&Instruction::SetBCD(0x1));
        let raw = cpu.memory.clone().into_inner();
        assert_eq!(raw[0x400], 0);
        assert_eq!(raw[0x401], 1);
        assert_eq!(raw[0x402], 9);
        assert_eq!(raw[0x403], 0);

        cpu.address = 0x0500;
        cpu.regs[0x2] = 8;
        cpu.do_instruction(&Instruction::SetBCD(0x2));
        let raw = cpu.memory.clone().into_inner();
        assert_eq!(raw[0x500], 0);
        assert_eq!(raw[0x501], 0);
        assert_eq!(raw[0x502], 8);
        assert_eq!(raw[0x503], 0);
    }

    #[test]
    fn test_dump_reg() {
        let mut cpu = CPU::new(&vec![], None);
        cpu.regs[0x0] = 0x01;
        cpu.regs[0x1] = 0x02;
        cpu.regs[0x2] = 0x03;
        cpu.regs[0x3] = 0x04;
        cpu.regs[0x4] = 0x05;
        cpu.regs[0x5] = 0x06;
        cpu.regs[0x6] = 0x07;
        cpu.regs[0x7] = 0x08;
        cpu.regs[0x8] = 0x09;
        cpu.regs[0x9] = 0x0a;
        cpu.regs[0xa] = 0x0b;
        cpu.regs[0xb] = 0x0c;
        cpu.regs[0xc] = 0x0d;
        cpu.regs[0xd] = 0x0e;
        cpu.regs[0xe] = 0x0f;
        cpu.address = 0x0300;
        cpu.memory.set_position(cpu.address as u64);
        cpu.do_instruction(&Instruction::DumpReg(0xe));
        assert_eq!(cpu.pc, 0x202);

        let raw = cpu.memory.into_inner();
        assert_eq!(cpu.address, 0x30f);
        assert_eq!(raw[0x0300], 0x1);
        assert_eq!(raw[0x0301], 0x2);
        assert_eq!(raw[0x0302], 0x3);
        assert_eq!(raw[0x0303], 0x4);
        assert_eq!(raw[0x0304], 0x5);
        assert_eq!(raw[0x0305], 0x6);
        assert_eq!(raw[0x0306], 0x7);
        assert_eq!(raw[0x0307], 0x8);
        assert_eq!(raw[0x0308], 0x9);
        assert_eq!(raw[0x0309], 0xa);
        assert_eq!(raw[0x030a], 0xb);
        assert_eq!(raw[0x030b], 0xc);
        assert_eq!(raw[0x030c], 0xd);
        assert_eq!(raw[0x030d], 0xe);
        assert_eq!(raw[0x030e], 0xf);
    }

    #[test]
    fn test_load_reg() {
        let mut cpu = CPU::new(&vec![], None);

        cpu.address = 0x0300;
        cpu.memory.set_position(cpu.address as u64);
        cpu.memory
            .write_all(&[
                0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf, 0x10,
            ])
            .unwrap();
        cpu.do_instruction(&Instruction::LoadReg(0xe));
        assert_eq!(cpu.pc, 0x202);

        let raw = cpu.memory.into_inner();
        assert_eq!(cpu.address, 0x30f);

        assert_eq!(cpu.regs[0x0], 0x01);
        assert_eq!(cpu.regs[0x1], 0x02);
        assert_eq!(cpu.regs[0x2], 0x03);
        assert_eq!(cpu.regs[0x3], 0x04);
        assert_eq!(cpu.regs[0x4], 0x05);
        assert_eq!(cpu.regs[0x5], 0x06);
        assert_eq!(cpu.regs[0x6], 0x07);
        assert_eq!(cpu.regs[0x7], 0x08);
        assert_eq!(cpu.regs[0x8], 0x09);
        assert_eq!(cpu.regs[0x9], 0x0a);
        assert_eq!(cpu.regs[0xa], 0x0b);
        assert_eq!(cpu.regs[0xb], 0x0c);
        assert_eq!(cpu.regs[0xc], 0x0d);
        assert_eq!(cpu.regs[0xd], 0x0e);
        assert_eq!(cpu.regs[0xe], 0x0f);
        assert_eq!(cpu.regs[0xf], 0x00);
    }

    #[test]
    fn test_set_memory_for_font() {
        let mut cpu = CPU::new(&vec![], None);
        cpu.regs[0x0] = 0;
        cpu.do_instruction(&Instruction::SetMemoryForFont(0x0));
        assert_eq!(cpu.address, 0x0);

        cpu.regs[0x0] = 9;
        cpu.do_instruction(&Instruction::SetMemoryForFont(0x0));
        assert_eq!(cpu.address, 45);

        cpu.regs[0x0] = 0xf;
        cpu.do_instruction(&Instruction::SetMemoryForFont(0x0));
        assert_eq!(cpu.address, 75);
    }

    #[test]
    fn test_it() {
        let mut data = vec![0; 4];
        data.write_u16::<BigEndian>(0xf029);
        data.write_u16::<BigEndian>(0xd00f);
    }
}
