use crate::machine::register::Register;

pub struct Format1 {
    pub opcode: u8,
}

pub struct Format2 {
    pub opcode: u8,
    pub r1: Register,
    pub r2: Register,
}

pub struct Format34 {
    pub opcode: u8,
    pub address: u32,
    pub flags: u8,
}
