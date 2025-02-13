use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::ops::Sub;
use std::time::{Duration, Instant};
use crate::machine::machine::Machine;
use crate::machine::register::Register;
use crate::opcode::format::{Format1, Format2, Format34};
use crate::opcode::instruction::Instruction;
use bitflags::parser::ParseHex;

pub struct Runner {
    machine: Machine,
    frequency: u64,
    last_ex: Option<Instant>,
}

impl Runner {
    pub fn new() -> Self {
        Self {
            machine: Machine::new(),
            frequency: 1_000_000,
            last_ex: None,
        }
    }

    pub fn machine(&self) -> &Machine {
        &self.machine
    }

    pub fn running(&self) -> bool {
        self.last_ex.is_some()
    }

    pub fn load_file(&mut self, mut file: &File) -> Result<(), String> {
        // Check running
        if self.running() {
            // Todo - auto stop machine & clear memory
            //return Err("Please stop machine first".into());
        }
        self.machine.reset();

        // Load file into memory
        let mut buffer = [0; 1];
        let mut char_buffer6 = [0; 6];

        if file.read(&mut buffer).is_ok() {
            // We should have read 'H'
            if buffer[0] != 'H' as u8 {
                return Err(format!("Unexpected char, expected 'H', found {}.", buffer[0] as char));
            }
        }

        // Read 6 chars into prg_name
        if let Err(e) = file.read_exact(&mut char_buffer6) {
            return Err(e.to_string())
        }

        if let Err(e) = file.read_exact(&mut char_buffer6) {
            return Err(e.to_string())
        }

        let start_addr = u32::from_str_radix(String::from_utf8(Vec::from(char_buffer6)).unwrap().as_str(), 16);
        if let Err(e) = start_addr {
            return Err(e.to_string());
        }

        let start_addr = start_addr.unwrap();

        // Read prg len
        if let Err(e) = file.read_exact(&mut char_buffer6) {
            return Err(e.to_string())
        }

        let prg_len = u32::from_str_radix(String::from_utf8(Vec::from(char_buffer6)).unwrap().as_str(), 16);

        if let Err(e) = prg_len {
            return Err(e.to_string());
        }

        let _prg_len = prg_len.unwrap();

        let reader = BufReader::new(file);

        let mut end_record = None;
        for line in reader.lines() {
            if let Ok(line) = line {
                // First char is either T or E
                if line.starts_with('T') {
                    let addr_start = u32::from_str_radix(&line[1..7], 16).unwrap();
                    let mut i = 0;
                    
                    // Leave 2 bits out
                    let mut instructions = line[9..line.len()].chars().peekable();

                    while instructions.peek().is_some() {
                        let instruction: String = instructions.by_ref().take(2).collect();
                        let instruction = u8::parse_hex(instruction.as_str()).unwrap();

                        // Write instruction to machine
                        self.machine.write_byte(start_addr + addr_start + i, instruction);
                        i += 1;
                    }

                } else if line.starts_with('E') {
                    if end_record.is_some() {
                        return Err("Expected only 1 E record.".into());
                    }
                    end_record = Some(u32::parse_hex(&line[1..line.len()]).unwrap())
                }
            } else {
                return Err(line.err().unwrap().to_string())
            }
        }
        
        if end_record.is_none() {
            return Err("Expected E record at th end of file.".into());
        }
        
        let end_record = end_record.unwrap();
        // Set PC value to the first instruction address
        self.machine.set_reg(&Register::PC, end_record);


        Ok(())
    }

    pub fn stop(&mut self) {
        self.last_ex = None;
    }

    pub fn start(&mut self) {
        self.last_ex = Some(Instant::now().sub(Duration::from_secs(10)));
    }


    pub fn try_step(&mut self) -> Result<(), String> {
        let now = Instant::now();

        if let Some(prev) = self.last_ex {
            if now.duration_since(prev) >= Duration::from_micros(1_000_000 / self.frequency) {
                // Execute
                return self.step();
            }
        }
        Ok(())
    }


    /// Execute instruction at PC
    fn step(&mut self) -> Result<(), String> {
        let first_byte = self.machine.fetch();
        let opcode = first_byte & 0xFC;

        let instruction: Box<dyn Instruction> = match first_byte {
            // Format 1
            0xC0..=0xC8 | 0xF0..=0xF8 => Box::new(Format1 { opcode }),
            // Format 2
            0x90..=0xB8 => {
                let registers = self.machine.fetch();

                let r1 = (registers >> 4) & 0xF;
                let r1 = Register::from_index(r1)?;
                let r2 = registers & 0xF;
                let r2 = Register::from_index(r2)?;

                let instruct = Format2 { opcode, r1, r2 };
                Box::new(instruct)
            }
            // Format 3 / 4
            0x00..=0x88 | 0xD0..=0xE0 => {
                let mut flags = self.machine.fetch();
                let adplus = flags & 0xF;
                flags >>= 4;

                // Merge 2 bits from opcode
                flags |= (first_byte & 0b11) << 4;
                let extended = flags & 1 != 0;

                let mut addr = (((adplus as u16) << 8) | self.machine.fetch() as u16) as u32;

                if extended {
                    addr = addr << 8 | self.machine.fetch() as u32;
                }

                let instruct = Format34 { opcode, address: addr, flags };
                Box::new(instruct)
            }
            _ => return Err(format!("Invalid opcode {opcode:02X} at address {:06X}.", self.machine.get_reg(&Register::PC) - 3)),
        };

        self.last_ex = Some(Instant::now());
        instruction.exec(&mut self.machine)
    }
}