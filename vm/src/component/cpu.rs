use std::collections::HashMap;

use super::memory::Memory;
use super::memory_io::*;
use super::memory_map::MemoryMap;
use super::screen::Screen;

use arch::registers::*;

use crate::register;

#[allow(clippy::upper_case_acronyms)]
/// CPU struct that will be the "head" of the VM.
/// It handles everything from memory pointers to executing incomming instructions
pub struct CPU {
    pub(super) memory: MemoryMap,
    pub(super) registers: Memory,
    stack_frame_size: usize,
    register_map: HashMap<&'static str, usize>,
    pub(super) flags: u8,
}

impl CPU {
    pub fn get_memory_at_u8(&self, address: usize) -> Result<u8, MemoryError> {
        self.memory.get_memory_at_u8(address)
    }

    pub fn get_memory_at_u16(&self, address: usize) -> Result<u16, MemoryError> {
        self.memory.get_memory_at_u16(address)
    }

    pub fn get_register(&self, name: &'static str) -> Result<u16, MemoryError> {
        match self.register_map.get(name) {
            Some(reg) => register!(self, *reg),
            None => Err(MemoryError::NoRegister(name)),
        }
    }

    pub fn set_register(&mut self, name: &'static str, data: u16) -> Result<(), MemoryError> {
        match self.register_map.get(name) {
            Some(reg) => register!(self, *reg => data),
            None => Err(MemoryError::NoRegister(name)),
        }
    }

    pub fn print_registers(&self) {
        let regs = [
            "ip", "acc", "ax", "bx", "cx", "dx", "ex", "fx", "gx", "hx", "sp", "fp",
        ];

        print!("Label            : "); // gap to align text
        for label in regs.iter() {
            print!("{:<7}", label);
        }
        println!();

        self.registers.print_memory_chunk_u16(0, REGISTER_LEN);
    }

    pub(crate) fn fetch_reg(&mut self) -> Result<usize, ExecutionError> {
        Ok(self.fetch_u8()? as usize % REGISTER_NAMES.len())
    }

    /// Gets the 8bit instruction pointed to by the instruction pointer and increase himself by one
    fn fetch_u8(&mut self) -> Result<u8, ExecutionError> {
        let next_instruction = self.get_register("ip")?;
        let instruction = self.memory.get_memory_at_u8(next_instruction as usize)?;
        self.set_register("ip", next_instruction + 1)?;

        Ok(instruction)
    }

    /// Gets the instruction pointed to by the instruction pointer and increase himself by one
    fn fetch_u16(&mut self) -> Result<u16, ExecutionError> {
        let next_instruction = self.get_register("ip")?;
        let instruction = self.memory.get_memory_at_u16(next_instruction as usize)?;
        self.set_register("ip", next_instruction + 2)?;

        Ok(instruction)
    }

    pub(crate) fn fetch_literal_u16(&mut self) -> Result<u16, ExecutionError> {
        self.fetch_u16()
    }

    pub(crate) fn fetch_literal_u8(&mut self) -> Result<u8, ExecutionError> {
        self.fetch_u8()
    }

    pub(crate) fn fetch_mem(&mut self) -> Result<usize, ExecutionError> {
        self.fetch_u16().map(|x| x as usize)
    }

    pub(crate) fn push(&mut self, value: u16) -> Result<(), ExecutionError> {
        let sp_address = self.get_register("sp")?;
        self.memory.set_memory_at_u16(sp_address as usize, value)?;

        self.stack_frame_size += 2;
        Ok(self.set_register("sp", sp_address - 2)?)
    }

    pub(crate) fn pop(&mut self) -> Result<u16, ExecutionError> {
        let (head, carry) = self.get_register("sp")?.overflowing_add(2);
        if carry {
            return Err(ExecutionError::BadReturn);
        }
        self.set_register("sp", head)?;

        self.stack_frame_size -= 2;
        Ok(self.memory.get_memory_at_u16(head as usize)?)
    }

    // This methode save all registers in the stack and create a new stackframe.
    // Once the stackframe is created, the function jump to `address` given
    pub(crate) fn call(&mut self, address: u16) -> Result<(), ExecutionError> {
        let reg_to_save = ["ax", "bx", "cx", "dx", "ex", "fx", "gx", "hx", "ip"];

        // save all registers from R1 to R8 plus ip
        for reg in reg_to_save.iter() {
            self.push(self.get_register(reg)?)?;
        }

        // Save the size of the stackframe
        self.push(self.stack_frame_size as u16 + 2)?;

        // create a new stackframe
        self.set_register("fp", self.get_register("sp")?)?;
        self.stack_frame_size = 0;

        // jump to given address
        self.set_register("ip", address)?;
        Ok(())
    }

    // This methode save all registers in the stack and create a new stackframe.
    // Once the stackframe is created, the function jump to `address` given
    pub(crate) fn restor(&mut self) -> Result<(), ExecutionError> {
        // erase the current stackframe
        let fp_addr = self.get_register("fp")?;
        self.set_register("sp", fp_addr)?;

        // set stack_frame_size to 2 to avoid neg number on pop
        self.stack_frame_size = 2;

        // Restor the stackframe size and update start of stackframe
        let sf_size = self.pop()?;
        self.stack_frame_size = sf_size as usize;
        self.set_register("fp", sf_size)?;

        // Restor all registers, in reverse order than `call` do
        let reg_to_load = ["ip", "hx", "gx", "fx", "ex", "dx", "cx", "bx", "ax"];
        for reg in reg_to_load.iter() {
            let stack_value = self.pop()?;
            self.set_register(reg, stack_value)?;
        }

        // in ep004 at 13:33, we can see a part of code that pop "args"
        // 0x4F don't realy understand what that shit mean and decide
        // to not reproduce this code here. He expect that the
        // `Kink kryod` understand this and don't decide to kill him.

        Ok(())
    }

    pub fn step(&mut self) -> bool {
        match self.fetch_u8() {
            Ok(int) => match self.execute(int) {
                Ok(_) => true,
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
        for (id, ins) in instructions.iter().enumerate() {
            self.memory.set_memory_at_u8(id, *ins).unwrap();
        }
    }

    pub fn print_memory_chunk_u8(&self, start: usize, end: usize) {
        let memory_len = self.memory.len();
        let end = if end < memory_len { end } else { memory_len };

        print!("Memory at {:#06X} :", start);
        for address in start..end {
            match self.memory.get_memory_at_u8(address) {
                Ok(data) if data > 0 => print!(" {:#04X}", data),
                _ => print!(" 0x--"),
            }
        }
        println!();
    }

    pub fn print_memory_chunk_u16(&self, start: usize, end: usize) {
        let memory_len = self.memory.len();
        let end = if end < memory_len { end } else { memory_len };

        print!("Memory at {:#06X} :", start);
        for address in (start..end).step_by(2) {
            match self.memory.get_memory_at_u16(address) {
                Ok(data) if data > 0 => print!(" {:#06X}", data),
                _ => print!(" 0x----"),
            }
        }
        println!();
    }
}

impl Default for CPU {
    fn default() -> Self {
        let mut memory = MemoryMap::default();
        let screen = Screen::new(64, 64);
        memory.add_device(screen, 0x3000).unwrap();

        let mut registers = Memory::new(REGISTER_NAMES.len() * 2);
        registers
            .set_memory_at_u16(ADDRESS_OF[SP as usize], 0xFFFE)
            .unwrap();
        registers
            .set_memory_at_u16(ADDRESS_OF[FP as usize], 0xFFFE)
            .unwrap();

        // HashMap gives the register_id with the register name given
        let register_map = REGISTER_NAMES.iter().fold(HashMap::new(), |mut map, s| {
            let _ = map.insert(*s, map.len());
            map
        });

        Self {
            memory,
            registers,
            stack_frame_size: 0,
            register_map,
            flags: 0,
        }
    }
}

pub enum ExecutionError {
    InternalMemoryError(MemoryError),
    UnexpectedInstruction(u8),
    BadRegisterPtrLen,
    EndOfExecution,
    BadReturn,
}

impl From<MemoryError> for ExecutionError {
    fn from(error: MemoryError) -> Self {
        Self::InternalMemoryError(error)
    }
}

impl std::fmt::Debug for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            ExecutionError::InternalMemoryError(error) => {
                format!("Internal memory error: {:?}", error)
            }
            ExecutionError::UnexpectedInstruction(ins) => {
                format!("Instruction {:#04X} is not permitted", ins)
            }
            ExecutionError::BadRegisterPtrLen => {
                "Register of 8bit size can't be a memory ptr".to_owned()
            }
            ExecutionError::BadReturn => "Can't return outside of stackframe".to_owned(),
            ExecutionError::EndOfExecution => "CPU reaches end of executable code".to_owned(),
        };

        write!(f, "{}", error)
    }
}
