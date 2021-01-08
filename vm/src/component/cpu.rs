use std::collections::HashMap;

use crate::component::memory::{Memory, MemoryError};
use arch::instructions::*;

const REGISTER_NAMES: &'static [&'static str] = &[
    "ip", "acc",
    "r1", "r2", "r3", "r4",
    "r5", "r6", "r7", "r8",
    "sp", "fp",
];

/// CPU struct that will be the "head" of the VM.
/// It handles everything from memory pointers to executing incomming instructions
pub struct CPU {
    memory: Memory,
    registers: Memory,
    register_map: HashMap<&'static str, usize>,
}

impl CPU {
    pub fn new(memory: usize) -> Self {
        let register_map = REGISTER_NAMES
            .to_vec()
            .iter()
            .fold(HashMap::new(), |mut map, s| {
                let _ = map.insert(*s, map.len() * 2);
                map
            });

        let mut registers = Memory::new(REGISTER_NAMES.len() * 2);
        registers.set_memory_at_u16(10*2, (memory - 2) as u16).unwrap(); // 10 is the index of SP; - 1 for the length and -1 because 2 bytes
        registers.set_memory_at_u16(11*2, (memory - 2) as u16).unwrap(); // 11 is the index of FP; - 1 for the length and -1 because 2 bytes

        Self {
            memory: Memory::new(memory),
            registers,
            register_map,
        }
    }

    pub fn get_register(&self, name: &'static str) -> Result<u16, MemoryError> {
        let reg_pointer = self
            .register_map
            .get(name)
            .expect("Register name does not exist");
        self.registers.get_memory_at_u16(*reg_pointer)
    }

    pub fn set_register(&mut self, name: &'static str, data: u16) -> Result<(), MemoryError> {
        let reg_pointer = self
            .register_map
            .get(name)
            .expect("Register name does not exist");
        self.registers.set_memory_at_u16(*reg_pointer, data)
    }

    pub fn print_registers(&self) {
        print!("Label            : "); // gap to align text
        for label in REGISTER_NAMES {
            print!("{: <7}", label);
        }
        print!("\n");

        self.registers
            .print_memory_chunk_u16(0, REGISTER_NAMES.len() * 2);
    }

    pub fn fetch_reg(&mut self) -> Result<usize, MemoryError> {
        Ok(self.fetch_u8()? as usize % REGISTER_NAMES.len())
    }

    /// Gets the 8bit instruction pointed to by the instruction pointer and increase himself by one
    pub fn fetch_u8(&mut self) -> Result<u8, MemoryError> {
        let next_instruction = self.get_register("ip")?;
        let instruction = self.memory.get_memory_at_u8(next_instruction as usize)?;
        self.set_register("ip", next_instruction + 1)?;

        Ok(instruction)
    }

    /// Gets the instruction pointed to by the instruction pointer and increase himself by one
    pub fn fetch_u16(&mut self) -> Result<u16, MemoryError> {
        let next_instruction = self.get_register("ip")?;
        let instruction = self.memory.get_memory_at_u16(next_instruction as usize)?;
        self.set_register("ip", next_instruction + 2)?;

        Ok(instruction)
    }

    fn execute(&mut self, instruction: u8) -> Result<(), ExecutionError> {
        #[cfg(debug_assertions)]
        print!("\nInstruction      : ");

        match instruction {
            // Move literal into a specific register
            MOV_LIT_REG => {
                let literal = self.fetch_u16()?;
                let reg = self.fetch_reg()?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!("Move {:#06X} (literal) in {}", literal, reg_name);
                }

                self.registers.set_memory_at_u16(reg * 2, literal)?;
                Ok(())
            }
            // Move register value into a specific register
            MOV_REG_REG => {
                let reg_from = self.fetch_reg()?;
                let reg_to = self.fetch_reg()?;

                #[cfg(debug_assertions)]
                {
                    let reg_from_name = REGISTER_NAMES[reg_from];
                    let reg_to_name = REGISTER_NAMES[reg_to];
                    println!("Move {} in {}", reg_from_name, reg_to_name);
                }

                let value = self.registers.get_memory_at_u16(reg_from * 2)?;
                self.registers.set_memory_at_u16(reg_to * 2, value)?;
                Ok(())
            }
            // Move register value into a specific memory address
            MOV_REG_MEM => {
                let reg = self.fetch_reg()?;
                let memory_address = self.fetch_u16()?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!("Move {} at {:#06X} (memory)", reg_name, memory_address);
                }

                let reg_value = self.registers.get_memory_at_u16(reg * 2)?;
                self.memory.set_memory_at_u16(memory_address as usize, reg_value)?;
                Ok(())
            }
            // Move memory value into a specific register
            MOV_MEM_REG => {
                let memory_address = self.fetch_u16()? as usize;
                let reg = self.fetch_reg()?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!("Move {:#06X} (memory) in {}", memory_address, reg_name);
                }

                let mem_value = self.memory.get_memory_at_u16(memory_address)?;
                self.registers.set_memory_at_u16(reg * 2, mem_value)?;
                Ok(())
            }
            // Jump to provided memory address if literal not equal to accumulator value
            JMP_NOT_EQ => {
                let literal = self.fetch_u16()?;
                let address_to_jmp = self.fetch_u16()?;
                let acc_value = self.get_register("acc")?;

                #[cfg(debug_assertions)]
                println!(
                    "Jump to {:#06X} (memory) if {:#06X} (literal) != to {:#06X} (acc)",
                    address_to_jmp, literal, acc_value
                );

                if acc_value != literal {
                    self.set_register("ip", address_to_jmp)?;
                }
                Ok(())
            }
            // Add register to register
            ADD_REG_REG => {
                let r1 = self.fetch_reg()?;
                let r2 = self.fetch_reg()?;

                #[cfg(debug_assertions)]
                {
                    let r1n = REGISTER_NAMES[r1];
                    let r2n = REGISTER_NAMES[r2];
                    println!("Add {} and {}, store result in ACC", r1n, r2n);
                }

                let r1_value = self.registers.get_memory_at_u16(r1 * 2)?;
                let r2_value = self.registers.get_memory_at_u16(r2 * 2)?;

                self.set_register("acc", r1_value.overflowing_add(r2_value).0)?;
                Ok(())
            }
            // Push Literal on Stack
            PSH_LIT => {
                let value = self.fetch_u16()?;
                
                #[cfg(debug_assertions)]
                println!("Push {:#06X} (literal) on stack, decrement stack pointer", value);

                self.push(value)
            }
            // Push register on stack
            PSH_REG => {
                let register_index = self.fetch_reg()?;
                let value = self.registers.get_memory_at_u16(register_index * 2)?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[register_index];
                    println!("Push {:#06X} (value on {}) on stack, decrement stack pointer", value, reg_name);
                }

                self.push(value)
            }
            // Pop stack head to given register
            POP_REG => {
                let reg = self.fetch_reg()?;
                let value = self.pop()?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!("Pop {:#06X} (value on stack) to {}, increment stack pointer", value, reg_name);
                }

                self.registers.set_memory_at_u16(reg * 2, value)?;
                Ok(())
            }
            // Xor register with other register
            XOR_REG_REG => {
                let r1 = self.fetch_reg()?;
                let r2 = self.fetch_reg()?;

                #[cfg(debug_assertions)]
                {
                    let r1n = REGISTER_NAMES[r1];
                    let r2n = REGISTER_NAMES[r2];
                    println!("Xor {} and {}, in {}", r1n, r2n, r1n);
                }

                let r1_value = self.registers.get_memory_at_u16(r1 * 2)?;
                let r2_value = self.registers.get_memory_at_u16(r2 * 2)?;
                self.registers.set_memory_at_u16(r1 * 2, r1_value ^ r2_value)?;
                Ok(())
            }
            // Xor register with literal
            XOR_REG_LIT => {
                let r1 = self.fetch_reg()?;
                let literal = self.fetch_u16()?;

                #[cfg(debug_assertions)]
                {
                    let r1n = REGISTER_NAMES[r1];
                    println!("Xor {} and {:#06X}, in {}", r1n, literal, r1n);
                }

                let r1_value = self.registers.get_memory_at_u16(r1 * 2)?;
                self.registers.set_memory_at_u16(r1 * 2, r1_value ^ literal)?;
                Ok(())
            }
            // End execution
            END => {
                #[cfg(debug_assertions)]
                println!("End of execution");

                Err(ExecutionError::EndOfExecution)
            }
            code => {
                #[cfg(debug_assertions)]
                println!("<ERROR> => The instruction {:#04X} is not known by this CPU\n", code);

                Err(ExecutionError::UnexpectedInstruction(code))
            }
        }
    }

    fn push(&mut self, value: u16) -> Result<(), ExecutionError> {
        let sp_address = self.get_register("sp")?;
        self.memory.set_memory_at_u16(sp_address as usize, value)?;

        self.set_register("sp", sp_address - 2)?;
        Ok(())  
    }

    fn pop(&mut self) -> Result<u16, MemoryError> {
        let head = self.get_register("sp")? + 2;
        self.set_register("sp", head)?;

        Ok(self.memory.get_memory_at_u16(head as usize)?)
    }

    pub fn step(&mut self) -> bool {
        match self.fetch_u8() {
            Ok(int) => match self.execute(int) {
                Ok(_ok) => true,
                Err(err) => {
                    match err {
                        ExecutionError::EndOfExecution => (),
                        _ => println!("{:?}", err),
                    }

                    false
                }
            },
            Err(err) => {
                println!("{:?}", err);
                false
            }
        }
    }

    // DEBUG FUNCTION DO NOT LEAVE IN RELEASE
    pub fn set_instruction(&mut self, instructions: &[u8]) {
        let mut pointer = 0;
        for i in instructions {
            let _ = self.memory.set_memory_at_u8(pointer, *i);
            pointer += 1;
        }
    }

    pub fn print_memory_chunk_u8(&self, start: usize, end: usize) {
        self.memory.print_memory_chunk_u8(start, end);
    }

    pub fn print_memory_chunk_u16(&self, start: usize, end: usize) {
        self.memory.print_memory_chunk_u16(start, end);
    }
}

enum ExecutionError {
    BadMemoryAccess,
    UnexpectedInstruction(u8),
    EndOfExecution,
}

impl From<MemoryError> for ExecutionError {
    fn from(_: MemoryError) -> Self {
        Self::BadMemoryAccess
    }
}

impl std::fmt::Debug for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            ExecutionError::BadMemoryAccess => format!("CPU try to access not allowed memory chunk !"),
            ExecutionError::UnexpectedInstruction(ins) => format!("Instruction {:#04X} is not permitted", ins),
            ExecutionError::EndOfExecution => format!("CPU reaches end of executable code"),
        };

        write!(f, "{}", error)
    }
}
