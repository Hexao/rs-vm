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
}