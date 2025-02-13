use std::collections::BTreeMap;
use crate::errors::RegisterError;
use crate::machine::register::Register;
use crate::device::device_trait::Device;
use crate::device::file_device::FileDevice;
use crate::device::std_device::{StdErrDevice, StdInDevice, StdOutDevice};


pub struct Machine {
    registers: [u32; 10],  // One off, but it's easier to index this way
    memory: BTreeMap<u32, u8>,
    pub(crate) devices: BTreeMap<u8, Box<dyn Device>>,
}

impl Machine {
    pub const MAX_ADDRESS: u32 = 0x00FF_FFFF;
    pub fn new() -> Self {
        let mut devices: BTreeMap<u8, Box<dyn Device>> = BTreeMap::new();
        // Set devices 0, 1 and 2 (stdin, stdout and stderr)
        devices.insert(0, Box::new(StdInDevice));
        devices.insert(1, Box::new(StdOutDevice));
        devices.insert(2, Box::new(StdErrDevice));

        let machine = Self {
            registers: [0; 10],
            memory: BTreeMap::new(),
            devices,
        };

        machine
    }


    pub fn reset(&mut self) {
        self.reset_registers();
        self.reset_memory();
        self.reset_devices();
    }

    fn reset_registers(&mut self) {
        self.registers = [0; 10];
    }

    fn reset_memory(&mut self) {
        self.memory.clear();
    }

    fn reset_devices(&mut self) {
        self.devices.clear();
    }

    /// Gets register value.
    pub fn get_reg(&self, register: &Register) -> u32 {
        self.registers[register.to_index()]
    }

    /// Gets the value in the F register as a floating point value.
    pub fn get_f_reg(&self) -> f32 {
        f32::from_bits(self.registers[6])
    }


    /// Sets register value.
    pub fn set_reg(&mut self, register: &Register, value: u32) {
        self.registers[register.to_index()] = value;
    }

    /// Sets the F register.
    /// Note: Floating point value is written as bits.
    pub fn set_f_reg(&mut self, value: f32) {
        self.registers[6] = value.to_bits();
    }

    /// Gets the register from its index.
    pub fn get_reg_from_index(&self, index: u8) -> Result<u32, RegisterError> {
        let reg = Register::from_index(index)?;

        let index = reg.to_index();
        Ok(self.registers[index])
    }


    /*
    pub fn set_reg_by_index(&mut self, index: u8, value: u32) -> Result<(), RegisterError> {
        let reg = Register::from_index(index)?;

        let index = reg.to_index();
        self.registers[index] = value;

        Ok(())
    }*/


    pub fn inc_x(&mut self) {
        let x = self.get_reg(&Register::X);
        self.set_reg(&Register::X, x + 1);
    }

    pub fn cmp_reg(&mut self, register: &Register, value: u32) {
        let x = self.get_reg(register);
        self.cmp_vals(x, value);
    }

    pub fn cmp_vals(&mut self, val1: u32, val2: u32) {
        self.set_reg(&Register::SW, match val1.cmp(&val2) {
                    std::cmp::Ordering::Less => 0x10,
                    std::cmp::Ordering::Equal => 0x00,
                    std::cmp::Ordering::Greater => 0x01,
        });
    }

    pub fn read_byte(&self, address: u32) -> u8 {
        if address > Self::MAX_ADDRESS {
            panic!("Address out of bounds: {:#X}", address);
        }

        match self.memory.get(&address) {
            Some(value) => *value,
            None => 0,
        }
    }

    pub fn write_byte(&mut self, address: u32, value: u8) {
        if address > Self::MAX_ADDRESS {
            panic!("Address out of bounds: {:#X}", address);
        }

        self.memory.insert(address, value);
    }

    pub fn read_word(&self, address: u32) -> u32 {
        let mut word = 0;

        for i in 0..3 {
            let byte = self.read_byte(address + i);
            word <<= 8;
            word |= byte as u32;
        }

        word
    }

    pub fn write_word(&mut self, address: u32, value: u32) {
        let address = address + 2;
        for i in 0..3 {
            let byte = (value >> (8 * i)) as u8;
            self.write_byte(address - i, byte);
        }
    }

    pub fn read_float(&self, address: u32) -> f32 {
        let word = self.read_word(address);
        f32::from_bits(word)
    }

    pub fn write_float(&mut self, address: u32, value: f32) {
        let word = value.to_bits();
        self.write_word(address, word);
    }

    ///
    pub fn get_device(&mut self, address: u8) -> &mut Box<dyn Device> {
        // If device exists, return it, otherwise create a new one
        self.devices.entry(address).or_insert(Box::new(FileDevice::new(address)))
    }


    /// Gets current instruction from memory and increments PC.
    /// Returns the instruction at the memory address pointed to by PC.
    pub(crate) fn fetch(&mut self) -> u8 {
        let pc = self.get_reg(&Register::PC);
        let value = self.read_byte(pc);
        self.set_reg(&Register::PC, pc + 1);

        value
    }
}
