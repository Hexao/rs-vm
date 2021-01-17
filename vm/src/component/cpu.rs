use std::collections::HashMap;

use crate::component::memory::Memory;
use arch::registers::REGISTER_NAMES;
use crate::component::memory_io::*;
use super::memory_map::MemoryMap;
use super::screen::Screen;
use arch::instructions::*;


/// CPU struct that will be the "head" of the VM.
/// It handles everything from memory pointers to executing incomming instructions
pub struct CPU {
    memory: MemoryMap,
    registers: Memory,
    stack_frame_size: usize,
    register_map: HashMap<&'static str, usize>,
}

impl CPU {
    pub fn get_register(&self, name: &'static str) -> Result<u16, MemoryError> {
        let reg_pointer = self
            .register_map
            .get(name)
            .unwrap_or_else(|| panic!("Register {} does not exist", name));
        self.registers.get_memory_at_u16(*reg_pointer)
    }

    pub fn set_register(&mut self, name: &'static str, data: u16) -> Result<(), MemoryError> {
        let reg_pointer = self
            .register_map
            .get(name)
            .unwrap_or_else(|| panic!("Register {} does not exist", name));
        self.registers.set_memory_at_u16(*reg_pointer, data)
    }

    pub fn print_registers(&self) {
        print!("Label            : "); // gap to align text
        for label in REGISTER_NAMES {
            print!("{:<7}", label);
        }
        println!();

        self.registers.print_memory_chunk_u16(0, REGISTER_NAMES.len() * 2);
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
            // Move literal directly in the memory
            MOV_LIT_MEM => {
                let literal = self.fetch_u16()?;
                let memory = self.fetch_u16()?;

                #[cfg(debug_assertions)]
                println!("Move {:#06X} (literal) in {:#06X} (memory)", literal, memory);

                self.memory.set_memory_at_u16(memory as usize, literal)?;
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
            // Move a memory address pointed by register in register
            MOV_PTRREG_REG => {
                let r1 = self.fetch_reg()?;
                let r2 = self.fetch_reg()?;

                let memory_loc = self.registers.get_memory_at_u16(r1 * 2)?;
                let memory_val = self.memory.get_memory_at_u16(memory_loc as usize)?;

                #[cfg(debug_assertions)]
                {
                    let r1_name = REGISTER_NAMES[r1];
                    let r2_name = REGISTER_NAMES[r2];
                    println!("Move value {:#06X} from memory {:#06X} pointed by {} to register {}",
                        memory_val, memory_loc, r1_name, r2_name
                    );
                }

                self.registers.set_memory_at_u16(r2 * 2, memory_val)?;
                Ok(())
            }
            // Move value from register to memory address pointed by register
            MOV_REG_PTRREG => {
                let r1 = self.fetch_reg()?;
                let r2 = self.fetch_reg()?;

                let val = self.registers.get_memory_at_u16(r1 * 2)?;
                let memory_loc = self.registers.get_memory_at_u16(r2 * 2)?;

                #[cfg(debug_assertions)]
                {
                    let r1_name = REGISTER_NAMES[r1];
                    let r2_name = REGISTER_NAMES[r2];
                    println!("Move {} into memory {:#06X} pointed by {}",
                        r1_name, memory_loc, r2_name
                    );
                }

                self.memory.set_memory_at_u16(memory_loc as usize, val)?;
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

                self.push(value)?;
                Ok(())
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

                self.push(value)?;
                Ok(())
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
            // call a function with literal address
            CALL_LIT => {
                let address = self.fetch_u16()?;

                #[cfg(debug_assertions)]
                println!("Call a subroutine at {:#06X} with literal", address);

                self.call(address)?;
                Ok(())
            }
            // call a function with a register value
            CALL_REG => {
                let reg = self.fetch_reg()?;
                let address = self.registers.get_memory_at_u16(reg * 2)?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!("Call a subroutine at {:#06X} (stored in register {})", address, reg_name);
                }

                self.call(address)?;
                Ok(())
            }
            // return from subroutine
            RET => {
                #[cfg(debug_assertions)]
                println!("Return from a subroutine");

                self.restor()?;
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

    fn push(&mut self, value: u16) -> Result<(), MemoryError> {
        let sp_address = self.get_register("sp")?;
        self.memory.set_memory_at_u16(sp_address as usize, value)?;

        self.stack_frame_size += 2;
        self.set_register("sp", sp_address - 2)?;
        Ok(())  
    }

    fn pop(&mut self) -> Result<u16, MemoryError> {
        let head = self.get_register("sp")? + 2;
        self.set_register("sp", head)?;

        self.stack_frame_size -= 2;
        Ok(self.memory.get_memory_at_u16(head as usize)?)
    }

    // This methode save all registers in the stack and create a new stackframe.
    // Once the stackframe is created, the function jump to `address` given
    fn call(&mut self, address: u16) -> Result<(), MemoryError> {
        let reg_to_save = ["r1", "r2", "r3", "r4", "r5", "r6", "r7", "r8", "ip"];

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
    fn restor(&mut self) -> Result<(), MemoryError> {
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
        let reg_to_load = ["ip", "r8", "r7", "r6", "r5", "r4", "r3", "r2", "r1"];
        for reg in reg_to_load.iter() {
            let stack_value  = self.pop()?;
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
        memory.add_device(Box::new(screen), 0x3000);

        let mut registers = Memory::new(REGISTER_NAMES.len() * 2);
        // 10 is the index of SP; - 1 for the length and -1 because 2 bytes
        registers.set_memory_at_u16(10 * 2, (0xFFFF - 1) as u16).unwrap();
        // 11 is the index of FP; - 1 for the length and -1 because 2 bytes
        registers.set_memory_at_u16(11 * 2, (0xFFFF - 1) as u16).unwrap();
        
        let register_map = REGISTER_NAMES
            .to_vec()
            .iter()
            .fold(HashMap::new(), |mut map, s| {
                let _ = map.insert(*s, map.len() * 2);
                map
            }
        );

        Self {
            memory,
            registers,
            stack_frame_size: 0,
            register_map,
        }
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
            ExecutionError::BadMemoryAccess => "CPU try to access not allowed memory chunk !".to_owned(),
            ExecutionError::UnexpectedInstruction(ins) => format!("Instruction {:#04X} is not permitted", ins),
            ExecutionError::EndOfExecution => "CPU reaches end of executable code".to_owned(),
        };

        write!(f, "{}", error)
    }
}
