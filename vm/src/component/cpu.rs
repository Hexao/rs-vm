use std::collections::HashMap;

use super::memory_map::MemoryMap;
use super::screen::Screen;
use super::memory::Memory;
use super::memory_io::*;

use arch::instructions::*;
use arch::registers::*;

macro_rules! register {
    ($self:ident, $reg:expr => $data:ident) => {
        match SIZE_OF[$reg] {
            1 => $self.registers.set_memory_at_u8(ADDRESS_OF[$reg], $data as u8),
            2 => $self.registers.set_memory_at_u16(ADDRESS_OF[$reg], $data),
            x => Err(MemoryError::BadRegisterLen(x)),
        }
    };

    ($self:ident, $reg:expr) => {
        match SIZE_OF[$reg] {
            1 => Ok($self.registers.get_memory_at_u8(ADDRESS_OF[$reg])? as u16),
            2 => $self.registers.get_memory_at_u16(ADDRESS_OF[$reg]),
            x => Err(MemoryError::BadRegisterLen(x)),
        }
    };
}

macro_rules! flag {
    ($self:ident, $value:ident) => {
        $self.flags = 0;
        if $value == 0 { $self.flags |= CPU::F_ZERO_VAL; }
        if $value > 0x7F { $self.flags |= CPU::F_NEGATIF; }
    };

    ($self:ident, $value:ident, $carry:ident) => {
        $self.flags = 0;
        if $value == 0 { $self.flags |= CPU::F_ZERO_VAL; }
        if $value > 0x7F { $self.flags |= CPU::F_NEGATIF; }
        if $carry { $self.flags |= CPU::F_CARRY; }
    };
}

/// CPU struct that will be the "head" of the VM.
/// It handles everything from memory pointers to executing incomming instructions
pub struct CPU {
    memory: MemoryMap,
    registers: Memory,
    stack_frame_size: usize,
    register_map: HashMap<&'static str, usize>,
    flags: u8,
}

impl CPU {
    const F_ZERO_VAL: u8 = 1; // bit0
    const F_NEGATIF : u8 = 2; // bit1
    const F_CARRY   : u8 = 4; // bit2

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
            "ip", "acc", "ax", "bx", "cx", "dx",
            "ex", "fx", "gx", "hx", "sp", "fp"
        ];

        print!("Label            : "); // gap to align text
        for label in regs.iter() {
            print!("{:<7}", label);
        }
        println!();

        self.registers.print_memory_chunk_u16(0, REGISTER_LEN);
    }

    fn fetch_reg(&mut self) -> Result<usize, ExecutionError> {
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

                flag!(self, literal);
                Ok(register!(self, reg => literal)?)
            }
            // Move literal directly in the memory
            MOV_LIT_MEM8 => {
                let literal = self.fetch_u16()? as u8;
                let memory = self.fetch_u16()? as usize;

                #[cfg(debug_assertions)]
                println!(
                    "Move {:#04X} (literal) in {:#06X} (memory)",
                    literal, memory
                );

                flag!(self, literal);
                Ok(self.memory.set_memory_at_u8(memory, literal)?)
            }
            // Move literal directly in the memory
            MOV_LIT_MEM16 => {
                let literal = self.fetch_u16()?;
                let memory = self.fetch_u16()?;

                #[cfg(debug_assertions)]
                println!(
                    "Move {:#06X} (literal) in {:#06X} (memory)",
                    literal, memory
                );

                flag!(self, literal);
                Ok(self.memory.set_memory_at_u16(memory as usize, literal)?)
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

                let value = register!(self, reg_from)?;
                flag!(self, value);

                Ok(register!(self, reg_to => value)?)
            }
            // Move register value into a specific memory address
            MOV_REG_MEM => {
                let reg = self.fetch_reg()?;
                let memory_address = self.fetch_u16()? as usize;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!("Move {} at {:#06X} (memory)", reg_name, memory_address);
                }

                match SIZE_OF[reg] {
                    1 => {
                        let value = self.registers.get_memory_at_u8(ADDRESS_OF[reg])?;
                        flag!(self, value);

                        Ok(self.memory.set_memory_at_u8(memory_address, value)?)
                    }
                    2 => {
                        let value = self.registers.get_memory_at_u16(ADDRESS_OF[reg])?;
                        flag!(self, value);

                        Ok(self.memory.set_memory_at_u16(memory_address, value)?)
                    }
                    x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x))),
                }
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

                match SIZE_OF[reg] {
                    1 => {
                        let value = self.memory.get_memory_at_u8(memory_address)?;
                        flag!(self, value);

                        Ok(self.registers.set_memory_at_u8(ADDRESS_OF[reg], value)?)
                    }
                    2 => {
                        let value = self.memory.get_memory_at_u16(memory_address)?;
                        flag!(self, value);

                        Ok(self.registers.set_memory_at_u16(ADDRESS_OF[reg], value)?)
                    }
                    x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x))),
                }
            }
            // Move memory value to another memory address

            // Move memory value to another memory address

            // Move a memory address pointed by register in register
            MOV_PTRREG_REG => {
                let r1 = self.fetch_reg()?;
                let r2 = self.fetch_reg()?;

                match SIZE_OF[r1] {
                    1 => Err(ExecutionError::BadRegisterPtrLen),
                    2 => {
                        let mem_loc = self.registers.get_memory_at_u16(ADDRESS_OF[r1])? as usize;
                        let mem_val = match SIZE_OF[r2] {
                            1 => self.memory.get_memory_at_u8(mem_loc)? as u16,
                            2 => self.memory.get_memory_at_u16(mem_loc)?,
                            x => return Err(ExecutionError::from(MemoryError::BadRegisterLen(x))),
                        };

                        #[cfg(debug_assertions)]
                        {
                            let r1_name = REGISTER_NAMES[r1];
                            let r2_name = REGISTER_NAMES[r2];
                            println!(
                                "Move value {:#06X} from memory {:#06X} pointed by {} to register {}",
                                mem_val, mem_loc, r1_name, r2_name
                            );
                        }

                        flag!(self, mem_val);
                        Ok(register!(self, r2 => mem_val)?)
                    }
                    x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x)))
                }
            }
            // Move value from register to memory address pointed by register
            MOV_REG_PTRREG => {
                let r1 = self.fetch_reg()?;
                let r2 = self.fetch_reg()?;

                match SIZE_OF[r2] {
                    1 => Err(ExecutionError::BadRegisterPtrLen),
                    2 => {
                        let val = register!(self, r1)?;
                        let mem_loc = self.registers.get_memory_at_u16(ADDRESS_OF[r2])? as usize;

                        #[cfg(debug_assertions)]
                        {
                            let r1_name = REGISTER_NAMES[r1];
                            let r2_name = REGISTER_NAMES[r2];
                            println!(
                                "Move value {:#06X} from {} into memory {:#06X} pointed by {}",
                                val, r1_name, mem_loc, r2_name
                            );
                        }

                        flag!(self, val);
                        match SIZE_OF[r1] {
                            1 => Ok(self.memory.set_memory_at_u8(mem_loc, val as u8)?),
                            2 => Ok(self.memory.set_memory_at_u16(mem_loc, val)?),
                            x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x)))
                        }
                    }
                    x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x)))
                }
            }
            // Move value from memory address = [literal + register] to register
            MOV_LITOFF_REG => {
                let base_address = self.fetch_u16()? as usize;
                let r1 = self.fetch_reg()?;
                let r2 = self.fetch_reg()?;

                match SIZE_OF[r1] {
                    1 => Err(ExecutionError::BadRegisterPtrLen),
                    2 => {
                        
                        let offset = self.registers.get_memory_at_u16(r1)? as usize;
                        let val = self.memory.get_memory_at_u16( base_address + offset)?;

                        #[cfg(debug_assertions)]
                        {
                            let r2_name = REGISTER_NAMES[r2];
                            println!(
                                "Move value {:#06X} from {:#06X} in memory to {}",
                                val, base_address + offset, r2_name
                            );
                        }

                        flag!(self, val);
                        match SIZE_OF[r2] {
                            1 => Ok(self.registers.set_memory_at_u8(ADDRESS_OF[r2], val as u8)?),
                            2 => Ok(self.registers.set_memory_at_u16(ADDRESS_OF[r2], val)?),
                            x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x)))
                        }
                    }
                    x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x)))
                }
            }
            JMP_LIT => {
                let address_to_jmp = self.fetch_u16()?;

                #[cfg(debug_assertions)]
                println!("Jump to {:#06X} (memory)", address_to_jmp);

                self.set_register("ip", address_to_jmp)?;
                Ok(())
            }
            // Jump to provided memory address if literal equal to accumulator value
            JEQ_LIT => {
                let add = self.fetch_u16()?;

                #[cfg(debug_assertions)]
                println!("Jump to {:#06X} (memory) if flag ZERO is set to true", add);

                if (self.flags & CPU::F_ZERO_VAL) != 0 { // flag f_zero_val is on
                    self.set_register("ip", add)?;
                }
                Ok(())
            }
            // Jump to provided memory address if literal not equal to accumulator value
            JNE_LIT => {
                let add = self.fetch_u16()?;

                #[cfg(debug_assertions)]
                println!("Jump to {:#06X} (memory) if flag ZERO is set to false", add);

                if (self.flags & CPU::F_ZERO_VAL) == 0 { // flag f_zero_val is off
                    self.set_register("ip", add)?;
                }
                Ok(())
            }
            // Jump to provided memory address if literal is greater than accumulator value
            JGT_LIT => {
                let add = self.fetch_u16()?;

                #[cfg(debug_assertions)]
                println!("Jump to {:#06X} (memory) if flags ZERO and NEGATIF are set to false", add);

                if (self.flags & (CPU::F_ZERO_VAL | CPU::F_NEGATIF)) == 0 {
                    self.set_register("ip", add)?;
                }
                Ok(())
            }
            // Jump to provided memory address if literal is greater or equal to accumulator value
            JGE_LIT => {
                let add = self.fetch_u16()?;

                #[cfg(debug_assertions)]
                println!("Jump to {:#06X} (memory) if flag NEGATIF is set to false", add);

                if (self.flags & CPU::F_NEGATIF) == 0 {
                    self.set_register("ip", add)?;
                }
                Ok(())
            }
            // Jump to provided memory address if literal is less than accumulator value
            JLT_LIT => {
                let add = self.fetch_u16()?;

                #[cfg(debug_assertions)]
                println!("Jump to {:#06X} (memory) if flag ZERO is set to false and flag NEGATIF is set to true", add);

                if (self.flags & (CPU::F_ZERO_VAL | CPU::F_NEGATIF)) == CPU::F_NEGATIF { // not equal + neg
                    self.set_register("ip", add)?;
                }
                Ok(())
            }
            // Jump to provided memory address if literal is less or equal to accumulator value
            JLE_LIT => {
                let add = self.fetch_u16()?;

                #[cfg(debug_assertions)]
                println!("Jump to {:#06X} (memory) if flag NEGATIF is set to true", add);

                if (self.flags & CPU::F_NEGATIF) != 0 {
                    self.set_register("ip", add)?;
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

                let r1_value = register!(self, r1)?;
                let r2_value = register!(self, r2)?;

                let (res, carry) = r1_value.overflowing_add(r2_value);
                flag!(self, res, carry);

                Ok(self.set_register("acc", res)?)
            }
            // Add register with literal
            ADD_REG_LIT => {
                let reg = self.fetch_reg()?;
                let val = self.fetch_u16()?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!("Add {} and {:#06X}, store result in ACC", reg_name, val);
                }

                let reg_val = register!(self, reg)?;
                let (res, carry) = val.overflowing_add(reg_val);
                flag!(self, res, carry);

                Ok(self.set_register("acc", res)?)
            }
            // Substract register to register
            SUB_REG_REG => {
                let r1 = self.fetch_reg()?;
                let r2 = self.fetch_reg()?;

                #[cfg(debug_assertions)]
                {
                    let r1n = REGISTER_NAMES[r1];
                    let r2n = REGISTER_NAMES[r2];
                    println!("Substract {} from {}, store result in ACC", r1n, r2n);
                }

                let r1_value = register!(self, r1)?;
                let r2_value = register!(self, r2)?;

                let (res, carry) = r2_value.overflowing_sub(r1_value);
                flag!(self, res, carry);

                Ok(self.set_register("acc", res)?)
            }
            // Substract register with literal
            SUB_REG_LIT => {
                let reg = self.fetch_reg()?;
                let val = self.fetch_u16()?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!("Substract {} from {:#06X}, store result in ACC", reg_name, val);
                }

                let reg_val = register!(self, reg)?;
                let (res, carry) = val.overflowing_sub(reg_val);
                flag!(self, res, carry);

                Ok(self.set_register("acc", res)?)
            }
            // Substract register with literal
            SUB_LIT_REG => {
                let val = self.fetch_u16()?;
                let reg = self.fetch_reg()?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!("Substract {:#06X} from {}, store result in ACC", val, reg_name);
                }

                let reg_val = register!(self, reg)?;
                let (res, carry) = reg_val.overflowing_sub(val);
                flag!(self, res, carry);

                Ok(self.set_register("acc", res)?)
            }
            // Multiply register to register
            MUL_REG_REG => {
                let r1 = self.fetch_reg()?;
                let r2 = self.fetch_reg()?;

                #[cfg(debug_assertions)]
                {
                    let r1n = REGISTER_NAMES[r1];
                    let r2n = REGISTER_NAMES[r2];
                    println!("Multiply {} and {}, store result in ACC", r1n, r2n);
                }

                let r1_value = register!(self, r1)?;
                let r2_value = register!(self, r2)?;

                let (res, carry) = r1_value.overflowing_mul(r2_value);
                flag!(self, res, carry);

                Ok(self.set_register("acc", res)?)
            }
            // Multiply register with literal
            MUL_REG_LIT => {
                let reg = self.fetch_reg()?;
                let val = self.fetch_u16()?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!("Multiply {} and {:#06X}, store result in ACC", reg_name, val);
                }

                let reg_val = register!(self, reg)?;
                let (res, carry) = val.overflowing_mul(reg_val);
                flag!(self, res, carry);

                Ok(self.set_register("acc", res)?)
            }
            CMP_REG_REG => {
                let r1 = self.fetch_reg()?;
                let r2 = self.fetch_reg()?;

                #[cfg(debug_assertions)]
                {
                    let r1_name = REGISTER_NAMES[r1];
                    let r2_name = REGISTER_NAMES[r2];
                    println!("Compare {} and {} values", r1_name, r2_name);
                }

                let r1_val = register!(self, r1)?;
                let r2_val = register!(self, r2)?;
                let (res, carry) = r1_val.overflowing_sub(r2_val);
                flag!(self, res, carry);
                Ok(())
            }
            CMP_REG_LIT => {
                let reg = self.fetch_reg()?;
                let lit = self.fetch_u16()?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!("Compare {} value with {:#06X}", reg_name, lit);
                }

                let reg_val = register!(self, reg)?;
                let (res, carry) = reg_val.overflowing_sub(lit);
                flag!(self, res, carry);
                Ok(())
            }
            // Increment register value by one
            INC_REG => {
                let reg = self.fetch_reg()?;
                let add = ADDRESS_OF[reg];

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!("Increment {} value by one", reg_name);
                }

                match SIZE_OF[reg] {
                    1 => {
                        let val = self.registers.get_memory_at_u8(add)?;
                        let (res, carry) = val.overflowing_add(1);
                        flag!(self, res, carry);

                        Ok(self.registers.set_memory_at_u8(add, res)?)
                    }
                    2 => {
                        let val = self.registers.get_memory_at_u16(add)?;
                        let (res, carry) = val.overflowing_add(1);
                        flag!(self, res, carry);

                        Ok(self.registers.set_memory_at_u16(add, res)?)
                    }
                    x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x))),
                }
            }
            // Decrement register value by one
            DEC_REG => {
                let reg = self.fetch_reg()?;
                let add = ADDRESS_OF[reg];

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!("Increment {} value by one", reg_name);
                }

                match SIZE_OF[reg] {
                    1 => {
                        let val = self.registers.get_memory_at_u8(add)?;
                        let (res, carry) = val.overflowing_sub(1);
                        flag!(self, res, carry);

                        Ok(self.registers.set_memory_at_u8(add, res)?)
                    }
                    2 => {
                        let val = self.registers.get_memory_at_u16(add)?;
                        let (res, carry) = val.overflowing_sub(1);
                        flag!(self, res, carry);

                        Ok(self.registers.set_memory_at_u16(add, res)?)
                    }
                    x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x))),
                }
            }
            // Push Literal on Stack
            PSH_LIT => {
                let value = self.fetch_u16()?;

                #[cfg(debug_assertions)]
                println!(
                    "Push {:#06X} (literal) on stack, decrement stack pointer",
                    value
                );

                flag!(self, value);
                self.push(value)
            }
            // Push register on stack
            PSH_REG => {
                let register_index = self.fetch_reg()?;
                let value = register!(self, register_index)?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[register_index];
                    println!(
                        "Push {:#06X} (value on {}) on stack, decrement stack pointer",
                        value, reg_name
                    );
                }

                flag!(self, value);
                self.push(value)
            }
            // Push memory on stack
            PSH_MEM8 => {
                let memory_add = self.fetch_u16()? as usize;
                let value = self.memory.get_memory_at_u8(memory_add)?;

                #[cfg(debug_assertions)]
                println!(
                    "Push {:#06X} (value on memory {:#06X}) on stack, decrement stack pointer",
                    value, memory_add
                );

                flag!(self, value);
                self.push(value as u16)
            }
            // Push memory on stack
            PSH_MEM16 => {
                let memory_add = self.fetch_u16()? as usize;
                let value = self.memory.get_memory_at_u16(memory_add)?;

                #[cfg(debug_assertions)]
                println!(
                    "Push {:#06X} (value on memory {:#06X}) on stack, decrement stack pointer",
                    value, memory_add
                );

                flag!(self, value);
                self.push(value)
            }
            // Push memory poinyed by register on stack
            PSH_PTRREG8 => {
                let reg = self.fetch_reg()?;

                match SIZE_OF[reg] {
                    1 => Err(ExecutionError::BadRegisterPtrLen),
                    2 => {
                        let add = self.registers.get_memory_at_u16(ADDRESS_OF[reg])? as usize;
                        let value = self.memory.get_memory_at_u8(add)? as u16;

                        #[cfg(debug_assertions)]
                        {
                            let reg_name = REGISTER_NAMES[reg];
                            println!(
                                "Push {:#04X} (value on memory {:#06X} pointed by {}) on stack, decrement stack pointer",
                                value, add, reg_name
                            );
                        }

                        flag!(self, value);
                        self.push(value)
                    }
                    x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x)))
                }
            }
            // Push memory poinyed by register on stack
            PSH_PTRREG16 => {
                let reg = self.fetch_reg()?;

                match SIZE_OF[reg] {
                    1 => Err(ExecutionError::BadRegisterPtrLen),
                    2 => {
                        let add = self.registers.get_memory_at_u16(ADDRESS_OF[reg])? as usize;
                        let value = self.memory.get_memory_at_u16(add)?;

                        #[cfg(debug_assertions)]
                        {
                            let reg_name = REGISTER_NAMES[reg];
                            println!(
                                "Push {:#06X} (value on memory {:#06X} pointed by {}) on stack, decrement stack pointer",
                                value, add, reg_name
                            );
                        }

                        flag!(self, value);
                        self.push(value)
                    }
                    x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x)))
                }
            }
            // Pop stack head to given register
            POP_REG => {
                let reg = self.fetch_reg()?;
                let value = self.pop()?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!(
                        "Pop {:#06X} (value on stack) to {}, increment stack pointer",
                        value, reg_name
                    );
                }

                flag!(self, value);
                register!(self, reg => value)?;
                Ok(())
            }
            // Pop stack head to given memory address
            POP_MEM8 => {
                let memory_add = self.fetch_u16()? as usize;
                let value = self.pop()?;

                #[cfg(debug_assertions)]
                println!(
                    "Pop {:#06X} (value on stack) to memory {:#06X}, increment stack pointer",
                    value, memory_add
                );

                flag!(self, value);
                self.memory.set_memory_at_u8(memory_add, value as u8)?;
                Ok(())
            }
            POP_MEM16 => {
                let memory_add = self.fetch_u16()? as usize;
                let value = self.pop()?;

                #[cfg(debug_assertions)]
                println!(
                    "Pop {:#06X} (value on stack) to memory {:#06X}, increment stack pointer",
                    value, memory_add
                );

                flag!(self, value);
                self.memory.set_memory_at_u16(memory_add, value)?;
                Ok(())
            }
            // Pop stack head to memory address pointed by register
            POP_PTRREG8 => {
                let reg = self.fetch_reg()?;

                match SIZE_OF[reg] {
                    1 => Err(ExecutionError::BadRegisterPtrLen),
                    2 => {
                        let add = self.registers.get_memory_at_u16(ADDRESS_OF[reg])?;
                        let value = self.pop()? as u8;

                        #[cfg(debug_assertions)]
                        {
                            let reg_name = REGISTER_NAMES[reg];
                            println!(
                                "Pop {:#04X} (value on stack) to memory {:#06X} pointed by {}, increment stack pointer",
                                value, add, reg_name
                            );
                        }

                        flag!(self, value);
                        Ok(self.memory.set_memory_at_u8(add as usize, value)?)
                    }
                    x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x))),
                }
            }
            // Pop stack head to memory address pointed by register
            POP_PTRREG16 => {
                let reg = self.fetch_reg()?;

                match SIZE_OF[reg] {
                    1 => Err(ExecutionError::BadRegisterPtrLen),
                    2 => {
                        let add = self.registers.get_memory_at_u16(ADDRESS_OF[reg])?;
                        let value = self.pop()?;

                        #[cfg(debug_assertions)]
                        {
                            let reg_name = REGISTER_NAMES[reg];
                            println!(
                                "Pop {:#06X} (value on stack) to memory {:#06X} pointed by {}, increment stack pointer",
                                value, add, reg_name
                            );
                        }

                        flag!(self, value);
                        Ok(self.memory.set_memory_at_u16(add as usize, value)?)
                    }
                    x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x))),
                }
            }
            // call a function with literal address
            CALL_LIT => {
                let address = self.fetch_u16()?;

                #[cfg(debug_assertions)]
                println!("Call a subroutine at {:#06X} with literal", address);

                self.call(address)
            }
            // call a function with a register value
            CALL_REG => {
                let reg = self.fetch_reg()?;

                match SIZE_OF[reg] {
                    1 => Err(ExecutionError::BadRegisterPtrLen),
                    2 => {
                        let address = self.registers.get_memory_at_u16(ADDRESS_OF[reg])?;

                        #[cfg(debug_assertions)]
                        {
                            let reg_name = REGISTER_NAMES[reg];
                            println!(
                                "Call a subroutine at {:#06X} (stored in register {})",
                                address, reg_name
                            );
                        }

                        self.call(address)
                    }
                    x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x))),
                }
            }
            // return from subroutine
            RET => {
                #[cfg(debug_assertions)]
                println!("Return from a subroutine");

                self.restor()
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

                let r1_value = register!(self, r1)?;
                let r2_value = register!(self, r2)?;
                let res = r1_value ^ r2_value;

                flag!(self, res);
                Ok(register!(self, r1 => res)?)
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

                let val = register!(self, r1)?;
                let res = val ^ literal;

                flag!(self, res);
                Ok(register!(self, r1 => res)?)
            }
            // End execution
            END => {
                #[cfg(debug_assertions)]
                println!("End of execution");

                Err(ExecutionError::EndOfExecution)
            }
            code => {
                #[cfg(debug_assertions)]
                println!(
                    "<ERROR> => The instruction {:#04X} is not known by this CPU\n",
                    code
                );

                Err(ExecutionError::UnexpectedInstruction(code))
            }
        }
    }

    fn push(&mut self, value: u16) -> Result<(), ExecutionError> {
        let sp_address = self.get_register("sp")?;
        self.memory.set_memory_at_u16(sp_address as usize, value)?;

        self.stack_frame_size += 2;
        Ok(self.set_register("sp", sp_address - 2)?)
    }

    fn pop(&mut self) -> Result<u16, ExecutionError> {
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
    fn call(&mut self, address: u16) -> Result<(), ExecutionError> {
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
    fn restor(&mut self) -> Result<(), ExecutionError> {
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
        memory.add_device(Box::new(screen), 0x3000).unwrap();

        let mut registers = Memory::new(REGISTER_NAMES.len() * 2);
        registers.set_memory_at_u16(ADDRESS_OF[SP as usize], 0xFFFE).unwrap();
        registers.set_memory_at_u16(ADDRESS_OF[FP as usize], 0xFFFE).unwrap();

        // HashMap gives the register_id with the register name given
        let register_map = REGISTER_NAMES.iter()
            .fold(HashMap::new(), |mut map, s| {
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

enum ExecutionError {
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
            ExecutionError::InternalMemoryError(error) => format!("Internal memory error: {:?}", error),
            ExecutionError::UnexpectedInstruction(ins) => format!("Instruction {:#04X} is not permitted", ins),
            ExecutionError::BadRegisterPtrLen => "Register of 8bit size can't be a memory ptr".to_owned(),
            ExecutionError::BadReturn => "Can't return outside of stackframe".to_owned(),
            ExecutionError::EndOfExecution => "CPU reaches end of executable code".to_owned(),
        };

        write!(f, "{}", error)
    }
}
