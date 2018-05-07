use bitrange::BitRange;

type GPR = u8;
type Address = u16;

type HalfWord = u8;
type Word = u16;

#[derive(Debug, PartialEq)]
pub enum Instruction {
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
    pub fn from_u16(value: &u16) -> Option<Instruction> {
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
