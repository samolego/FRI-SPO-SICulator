use std::io;
use std::io::{Read, Write};
use crate::device::device_trait::Device;

pub struct StdInDevice;

impl Device for StdInDevice {
    fn test(&self) -> bool {
        true
    }

    fn read(&mut self) -> u8 {
        let mut buffer = [0; 1];
        io::stdin().read(&mut buffer).unwrap();
        buffer[0]
    }

    fn write(&mut self, _value: u8) {
        panic!("Cannot write to stdin")
    }
}

pub struct StdOutDevice;

impl Device for StdOutDevice {
    fn test(&self) -> bool {
        true
    }

    fn read(&mut self) -> u8 {
        panic!("Cannot read from stdout")
    }

    fn write(&mut self, value: u8) {
        io::stdout().write(&[value]).unwrap();
    }
}

pub struct StdErrDevice;
impl Device for StdErrDevice {
    fn test(&self) -> bool {
        true
    }

    fn read(&mut self) -> u8 {
        panic!("Cannot read from stderr")
    }

    fn write(&mut self, value: u8) {
        io::stderr().write(&[value]).unwrap();
    }
}
