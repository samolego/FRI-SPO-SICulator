use std::fs::File;
use std::io::{Read, Write};
use crate::device::device_trait::Device;

pub struct FileDevice {
    file: File,
}

impl Device for FileDevice {
    fn test(&self) -> bool {
        // Return whether the file exists
        self.file.metadata().is_ok()
    }

    fn read(&mut self) -> u8 {
        // Read the file and return the first byte
        let mut buffer = [0; 1];
        self.file.read(&mut buffer).expect(format!("Failed to read file ...").as_str());
        buffer[0]
    }

    fn write(&mut self, value: u8) {
        // Write the byte to the file
        self.file.write_all(&[value]).expect(format!("Failed to write file ...").as_str());
    }
}

impl FileDevice {
    pub fn new(device_addr: u8) -> Self {
        // Create file if it doesn't exist
        let file_name = format!("{:02X}.dev", device_addr);
        let exists = std::path::Path::new(&file_name).exists();
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(!exists)
            .open(&file_name)
            .expect(format!("Failed to create device file {device_addr}.").as_str());

        Self {
            file,
        }
    }
}
