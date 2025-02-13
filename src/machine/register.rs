use crate::errors::RegisterError;

///
/// Represents a register in the SIC/XE machine.
/// Does not include the F register, which is a floating point register.
///
#[derive(Debug)]
pub struct Register(pub &'static str, pub u8);

impl Register {
    pub const A: Register = Register("A", 0);
    pub const X: Register = Register("X", 1);
    pub const L: Register = Register("L", 2);
    pub const B: Register = Register("B", 3);
    pub const S: Register = Register("S", 4);
    pub const T: Register = Register("T", 5);

    pub const F: Register = Register("F", 6);
    pub const PC: Register = Register("PC", 8);
    pub const SW: Register = Register("SW", 9);


    pub fn from_index(index: u8) -> Result<Self, RegisterError> {
        match index {
            0 => Ok(Self::A),
            1 => Ok(Self::X),
            2 => Ok(Self::L),
            3 => Ok(Self::B),
            4 => Ok(Self::S),
            5 => Ok(Self::T),
            6 => Ok(Self::F),
            8 => Ok(Self::PC),
            9 => Ok(Self::SW),
            _ => Err(RegisterError { index }),
        }
    }

    pub fn to_index(&self) -> usize {
        self.1 as usize
    }
}
