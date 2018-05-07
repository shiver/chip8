extern crate chip8;
extern crate byteorder;

use std::io::{Cursor, Error, Read, Write};

use byteorder::{ByteOrder, LittleEndian, WriteBytesExt, BigEndian};
use chip8::cpu::CPU;
use chip8::instructions::Instruction;

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
