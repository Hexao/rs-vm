use std::collections::HashMap;

use crate::component::memory::{ Memory, MemoryError };

const REGISTER_NAMES: &'static [&'static str] = &["ip", "acc", "r1", "r2", "r3", "r4", "r5", "r6", "r7", "r8"];

/// CPU struct that will be the "head" of the VM.
/// It handles everything from memory pointers to executing incomming instructions
pub struct CPU {
    memory: Memory,
    registers: Memory,
    register_map: HashMap<&'static str, usize>,
}

impl CPU {
    pub fn new(memory: usize) -> Self {
        let register_map = REGISTER_NAMES.to_vec().iter().fold(HashMap::new(), |mut map, s| {
            let _ = map.insert(*s, map.len()*2);
            map
        });

        Self {
            memory: Memory::new(memory),
            registers: Memory::new(REGISTER_NAMES.len()*2),
            register_map
        }
    }

    pub fn get_register(&self, name: &'static str) -> Result<u16, MemoryError> {

        let reg_pointer = self.register_map.get(name).expect("Register name does not exist");
        let left = self.registers.get_memory_at(*reg_pointer)?;
        let right = self.registers.get_memory_at(*reg_pointer+1)?;

        Ok(((left as u16) << 8) + (right as u16))
    }

    pub fn set_register(&mut self, name: &'static str, data: u16) -> Result<(), MemoryError> {

        let reg_pointer = self.register_map.get(name).expect("Register name does not exist");
        let left = (data >> 8) as u8;
        let right = (data % 0x100) as u8;
        self.registers.set_memory_at(*reg_pointer, left)?;
        self.registers.set_memory_at(*reg_pointer+1, right)?;

        Ok(())
    }
    
    pub fn print_registers(&self) {
        for (reg, pointer) in &self.register_map {
            println!("register: {}, data: {} {}", *reg, self.registers.get_memory_at(*pointer).unwrap(), self.registers.get_memory_at(*pointer +1).unwrap());
        }
    }

    /// Gets the 8bit instruction pointed to by the instruction pointer and increase himself by one
    pub fn fetch_u8(&mut self) -> Result<u8, MemoryError> {
        let next_instruction = self.get_register("ip")?;
        let instruction = self.memory.get_memory_at(next_instruction as usize)?;
        self.set_register("ip", next_instruction+1)?;

        Ok(instruction)
    }

    /// Gets the instruction pointed to by the instruction pointer and increase himself by one
    pub fn fetch_u16(&mut self) -> Result<u8, MemoryError> {
        let next_instruction = self.get_register("ip")?;
        let instruction = self.memory.get_memory_at(next_instruction as usize)?;
        self.set_register("ip", next_instruction+2)?;

        Ok(instruction)
    }

    pub fn execute(instruction: u8) {
        match instruction {
            0x10 => {}
            _ => {}
        }
    }
}