use std::io::{Cursor, Error, Write};

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use byteorder::{BigEndian, ReadBytesExt};
use rand;

use instructions::Instruction;
use FONT4X5;

const GRID_WIDTH: usize = 64;
const GRID_HEIGHT: usize = 32;

pub struct CPU {
    pub regs: [u8; 16],
    pub address: u16,
    pub stack: Vec<usize>,
    pub memory: Cursor<Vec<u8>>,

    pub delay_timer: u8,
    pub sound_timer: u8,

    pub pc: usize,

    pub keys: [u8; 16],
    pub display: Option<Canvas<Window>>,
    pub grid: Vec<u8>,
}

impl CPU {
    pub fn new(data: &Vec<u8>, display: Option<Canvas<Window>>) -> CPU {
        let mut memory = vec![0; 4096];
        for i in 0..data.len() {
            memory[0x200 + i] = data[i];
        }

        for i in 0..FONT4X5.len() {
            memory[0x0 + i] = FONT4X5[i];
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
            grid: vec![0; GRID_WIDTH * GRID_HEIGHT],
        }
    }

    pub fn show(&mut self) {
        if let Some(ref mut canvas) = self.display {
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            canvas.set_draw_color(Color::RGB(255, 255, 255));

            for y in 0..GRID_HEIGHT {
                for x in 0..GRID_WIDTH {
                    if self.grid[(y * GRID_WIDTH) + x] == 1 {
                        canvas
                            .fill_rect(Rect::new((x as u8) as i32 * 10,
                                                 (y as u8) as i32 * 10,
                                                 10,
                                                 10)).unwrap();
                    }
                }
            }

            canvas.present();
        }
    }

    fn clear(&mut self) {
        for idx in 0..GRID_WIDTH * GRID_HEIGHT {
            self.grid[idx] = 0;
        }
    }

    fn inc_pc(&mut self) {
        self.pc += 2;
    }

    pub fn fetch_opcode(&mut self) -> Result<u16, Error> {
        self.memory.set_position(self.pc as u64);
        self.memory.read_u16::<BigEndian>()
    }

    fn timer_tick(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn do_instruction(&mut self, instruction: &Instruction) -> Result<(), Error> {
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
                let mut start_x = self.regs[*vx as usize];
                let mut start_y = self.regs[*vy as usize];
                let height = *height;
                self.regs[0xf] = 0;

                if start_x > (GRID_WIDTH - 1) as u8 {
                    start_x = 0;
                }

                for y in 0..height {
                    self.memory.set_position((self.address + y as u16) as u64);
                    let row = self.memory.read_u8()?;

                    let final_y = y as u16 + start_y as u16;
                    for x in 0..8 {
                        let final_x = x as u16 + start_x as u16;
                        if final_x > (GRID_WIDTH - 1) as u16 {
                            continue;
                        }

                        let grid_pos = ((final_y * GRID_WIDTH as u16) + final_x) as usize;
                        if (row >> 7 - x) & 1 != 0 {
                            self.regs[0xf] = (self.grid[grid_pos] == 1) as u8;
                            self.grid[grid_pos] ^= 1;
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
                self.memory.write(&[h, t, d])?;
            }

            Instruction::DumpReg(vx) => {
                for idx in 0..*vx + 1 {
                    self.memory.set_position(self.address as u64);
                    self.memory.write(&[self.regs[idx as usize]])?;
                    self.address += 1;
                }
            }

            Instruction::LoadReg(vx) => {
                for idx in 0..*vx + 1 {
                    self.memory.set_position((self.address as u16) as u64);
                    self.regs[idx as usize] = self.memory.read_u8()?;
                    self.address += 1;
                }
            }
        }

        if should_increment {
            self.inc_pc();
        }

        Ok(())
    }
}
