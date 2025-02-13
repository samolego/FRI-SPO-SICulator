use crate::machine::machine::Machine;
use crate::machine::register::Register;
use crate::opcode::format::{Format1, Format2, Format34};
use crate::opcode::opcode::Opcode;

pub trait Instruction {
    fn exec(&self, machine: &mut Machine) -> Result<(), String>;
}

impl Instruction for Format1 {
    fn exec(&self, machine: &mut Machine) -> Result<(), String> {
        match self.opcode {
            Opcode::FLOAT => {
                let a = machine.get_reg(&Register::A);
                machine.set_f_reg(a as f32);
                Ok(())
            }
            Opcode::FIX => {
                let f = machine.get_f_reg();
                machine.set_reg(&Register::A, f as u32);
                Ok(())
            }
            Opcode::NORM => {
                todo!("What does NORM do?")
            }
            Opcode::SIO => {
                todo!("SIO not implemented yet.")
            }
            Opcode::HIO => {
                todo!("HIO not implemented yet.")
            }
            Opcode::TIO => {
                todo!("TIO not implemented yet.")
            }
            _ => Err(format!("Invalid F1 opcode {}.", self.opcode)),
        }
    }
}


impl Instruction for Format2 {
    fn exec(&self, machine: &mut Machine) -> Result<(), String> {
        let r1 = machine.get_reg(&self.r1);
        let r2 = machine.get_reg(&self.r2);

        match self.opcode {
            Opcode::ADDR => {
                machine.set_reg(&self.r2, r1 + r2);
                Ok(())
            }
            Opcode::SUBR => {
                machine.set_reg(&self.r2, r2 - r1);
                Ok(())
            }
            Opcode::MULR => {
                machine.set_reg(&self.r2, r1 * r2);
                Ok(())
            }
            Opcode::DIVR => {
                machine.set_reg(&self.r2, r2 / r1);
                Ok(())
            }
            Opcode::COMPR => {
                machine.cmp_vals(r1, r2);
                Ok(())
            }
            Opcode::SHIFTL => {
                machine.set_reg(&self.r1, r1 << r2);
                Ok(())
            }
            Opcode::SHIFTR => {
                machine.set_reg(&self.r1, r1 >> r2);
                Ok(())
            }
            Opcode::RMO => {
                // RMO
                machine.set_reg(&self.r2, r1);
                Ok(())
            }
            Opcode::SVC => {
                todo!("SVC not implemented yet.")
            }
            Opcode::CLEAR => {
                machine.set_reg(&self.r1, 0);
                Ok(())
            }
            Opcode::TIXR => {
                // Increment X
                machine.inc_x();
                // Compare X to r1
                machine.cmp_reg(&Register::X, r1);
                Ok(())
            }
            _ => Err(format!("Invalid F2 opcode {}.", self.opcode)),
        }
    }
}

impl Instruction for Format34 {
    fn exec(&self, machine: &mut Machine) -> Result<(), String> {
        let extended = self.flags & 1 == 1;
        let flags = self.flags >> 1;
        let mut provided_addr = self.address;
        let mut use_addr = if flags & 0b10 != 0 {
            // Base-relative
            let base = machine.get_reg(&Register::B);
            // Check negative
            let provided_addr = provided_addr as i8;
            (base as i64 + i64::from(provided_addr)) as u32
        } else if flags & 1 == 1 {
            // PC-relative
            let pc = machine.get_reg(&Register::PC);

            if extended && provided_addr & 1 << 23 != 0 {
                provided_addr = provided_addr | 0xFFF00000;
            } else if provided_addr & 1 << 11 != 0 {
                provided_addr = provided_addr | 0xFFFFF000;
            }

            // Check negative
            // When LDA #stackptr -> we must allow 260, but with
            let provided_addr = provided_addr as i64;

            (pc as i64 + provided_addr) as u32
        } else {
            // Simple
            provided_addr
        };


        let flags = flags >> 3;
        let opvalue = if flags & 1 == 1 && flags & 0b10 == 0 {
            // n = 0, i = 1
            // Immediate
            use_addr
        } else if flags & 0b10 != 0 && flags & 1 == 0 {
            // Posredno
            use_addr = machine.read_word(use_addr);
            machine.read_word(use_addr)
        } else {
            // Simple
            machine.read_word(use_addr)
        };


        match self.opcode {
            Opcode::LDA => {
                machine.set_reg(&Register::A, opvalue);
                Ok(())
            }
            Opcode::LDX => {
                machine.set_reg(&Register::X, opvalue);
                Ok(())
            },
            Opcode::LDL => {
                machine.set_reg(&Register::L, opvalue);
                Ok(())
            },
            Opcode::STA => {
                let value = machine.get_reg(&Register::A);
                machine.write_word(use_addr, value);
                Ok(())
            },
            Opcode::STX => {
                let value = machine.get_reg(&Register::X);
                machine.write_word(use_addr, value);
                Ok(())
            },
            Opcode::STL => {
                let value = machine.get_reg(&Register::L);
                machine.write_word(use_addr, value);
                Ok(())
            },
            Opcode::ADD => {
                let a = machine.get_reg(&Register::A);
                machine.set_reg(&Register::A, a + opvalue);
                Ok(())
            },
            Opcode::SUB => {
                let a = machine.get_reg(&Register::A);
                machine.set_reg(&Register::A, a - opvalue);
                Ok(())
            },
            Opcode::MUL => {
                let a = machine.get_reg(&Register::A);
                machine.set_reg(&Register::A, a * opvalue);
                Ok(())
            },
            Opcode::DIV => {
                let a = machine.get_reg(&Register::A);
                machine.set_reg(&Register::A, a / opvalue);
                Ok(())
            },
            Opcode::COMP => {
                machine.cmp_reg(&Register::A, opvalue);
                Ok(())
            },
            Opcode::TIX => {
                // Increment X
                machine.inc_x();
                // Compare X to value
                machine.cmp_reg(&Register::X, opvalue);
                Ok(())
            },
            Opcode::JEQ => {
                if machine.get_reg(&Register::SW) == 0 {
                    machine.set_reg(&Register::PC, use_addr);
                }
                Ok(())
            },
            Opcode::JGT => {
                if machine.get_reg(&Register::SW) == 0x01 {
                    machine.set_reg(&Register::PC, use_addr);
                }
                Ok(())
            },
            Opcode::JLT => {
                if machine.get_reg(&Register::SW) == 0x10 {
                    machine.set_reg(&Register::PC, use_addr);
                }
                Ok(())
            },
            Opcode::J => {
                machine.set_reg(&Register::PC, use_addr);
                Ok(())
            },
            Opcode::AND => {
                let a = machine.get_reg(&Register::A);
                machine.set_reg(&Register::A, a & opvalue);
                Ok(())
            },
            Opcode::OR => {
                let a = machine.get_reg(&Register::A);
                machine.set_reg(&Register::A, a | opvalue);
                Ok(())
            },
            Opcode::JSUB => {
                let pc = machine.get_reg(&Register::PC);
                machine.set_reg(&Register::L, pc);
                machine.set_reg(&Register::PC, use_addr);
                Ok(())
            },
            Opcode::RSUB => {
                let l = machine.get_reg(&Register::L);
                machine.set_reg(&Register::PC, l);
                Ok(())
            },
            Opcode::LDCH => {
                machine.set_reg(&Register::A, opvalue);
                Ok(())
            },
            Opcode::STCH => {
                let value = machine.get_reg(&Register::A) as u8;
                machine.write_byte(use_addr, value);
                Ok(())
            },
            Opcode::ADDF => {
                let f = machine.get_f_reg();
                machine.set_f_reg(f + f32::from_bits(opvalue));
                Ok(())
            },
            Opcode::SUBF => {
                let f = machine.get_f_reg();
                machine.set_f_reg(f - f32::from_bits(opvalue));
                Ok(())
            },
            Opcode::MULF => {
                let f = machine.get_f_reg();
                machine.set_f_reg(f * f32::from_bits(opvalue));
                Ok(())
            },
            Opcode::DIVF => {
                let f = machine.get_f_reg();
                machine.set_f_reg(f / f32::from_bits(opvalue));
                Ok(())
            },
            Opcode::LDB => {
                machine.set_reg(&Register::B, opvalue);
                Ok(())
            },
            Opcode::LDS => {
                machine.set_reg(&Register::S, opvalue);
                Ok(())
            },
            Opcode::LDF => {
                machine.set_reg(&Register::F, opvalue);
                Ok(())
            },
            Opcode::LDT => {
                machine.set_reg(&Register::T, opvalue);
                Ok(())
            },
            Opcode::STB => {
                let value = machine.get_reg(&Register::B);
                machine.write_word(use_addr, value);
                Ok(())
            },
            Opcode::STS => {
                let value = machine.get_reg(&Register::S);
                machine.write_word(use_addr, value);
                Ok(())
            },
            Opcode::STF => {
                let value = machine.get_reg(&Register::F);
                machine.write_word(use_addr, value);
                Ok(())
            },
            Opcode::STT => {
                let value = machine.get_reg(&Register::T);
                machine.write_word(use_addr, value);
                Ok(())
            },
            Opcode::COMPF => {
                let f = machine.get_f_reg();
                machine.cmp_vals(f.to_bits(), opvalue);
                Ok(())
            },
            Opcode::LPS => {
                todo!("LPS not implemented yet.")
            },
            Opcode::STI => {
                todo!("STI not implemented yet.")
            },
            Opcode::RD => {
                // Use all bits in this case!
                let addr = (self.flags as u32) << 6 | self.address;
                let opvalue = machine.read_byte(addr);
                let value = machine.get_device(opvalue).read();
                machine.set_reg(&Register::A, value as u32);
                Ok(())
            },
            Opcode::WD => {
                let addr = (self.flags as u32) << 6 | self.address;
                let opvalue = machine.read_byte(addr);
                let value = machine.get_reg(&Register::A) as u8;
                machine.get_device(opvalue).write(value);
                Ok(())
            },
            Opcode::TD => {
                let result = machine.get_device(opvalue as u8).test();
                todo!("Test device @{opvalue} result: {result}");
            },
            Opcode::STSW => {
                let value = machine.get_reg(&Register::SW);
                machine.write_word(opvalue, value);
                Ok(())
            },
            Opcode::SSK => {
                todo!("SSK not implemented yet.")
            },
            _ => Err(format!("Invalid F3/F4 opcode {}.", self.opcode)),
        }
    }
}
