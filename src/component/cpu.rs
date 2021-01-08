use std::collections::HashMap;

use crate::component::memory::{Memory, MemoryError};

const REGISTER_NAMES: &'static [&'static str] =
    &["ip", "acc", "r1", "r2", "r3", "r4", "r5", "r6", "r7", "r8"];

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

        Self {
            memory: Memory::new(memory),
            registers: Memory::new(REGISTER_NAMES.len() * 2),
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

    pub fn execute(&mut self, instruction: u8) -> Result<(), MemoryError> {
        #[cfg(debug_assertions)]
        print!("\nInstruction      : ");

        match instruction {
            // move literal into the r1 register
            0x10 => {
                let literal = self.fetch_u16()?;

                #[cfg(debug_assertions)]
                println!("Move {:#06X} in R1", literal);

                self.set_register("r1", literal)?;
            }
            // move literal into the r2 register
            0x11 => {
                let literal = self.fetch_u16()?;

                #[cfg(debug_assertions)]
                println!("Move {:#06X} in R2", literal);

                self.set_register("r2", literal)?;
            }
            // Add register to register
            0x12 => {
                #[cfg(debug_assertions)]
                println!("Add R1 and R2, store result in ACC");

                let r1 = self.fetch_u8()? as usize;
                let r2 = self.fetch_u8()? as usize;
                let register_value1 = self.registers.get_memory_at_u16(r1 * 2)?;
                let register_value2 = self.registers.get_memory_at_u16(r2 * 2)?;
                self.set_register("acc", register_value1.overflowing_add(register_value2).0)?;
            }
            _ => {}
        }

        Ok(())
    }

    pub fn step(&mut self) -> Result<(), MemoryError> {
        let int = self.fetch_u8()?;
        self.execute(int)?;
        Ok(())
    }

    // DEBUG FUNCTION DO NOT LEAVE IN RELEASE
    pub fn set_instruction(&mut self, instructions: &[u8]) {
        let mut pointer = 0;
        for i in instructions {
            let _ = self.memory.set_memory_at_u8(pointer, *i);
            pointer += 1;
        }
    }
}
