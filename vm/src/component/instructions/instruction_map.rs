use crate::component::cpu::{ExecutionError, CPU};

use super::keywords::*;

use crate::component::memory_io::{MemoryError, MemoryIO};

use crate::{flag, register};

use arch::flags::*;
use arch::instructions::*;
use arch::registers::*;

impl CPU {
    pub(crate) fn execute(&mut self, instruction: u8) -> Result<(), ExecutionError> {
        #[cfg(debug_assertions)]
        print!("\nInstruction      : ");

        match instruction {
            // Move literal into a specific register
            MOV_LIT_REG => self.interpret(Mov(Lit(U16), Reg)),
            // Move literal directly in the memory
            MOV_LIT_MEM8 => self.interpret(Mov(Lit(U8), Mem)),
            // Move literal directly in the memory
            MOV_LIT_MEM16 => self.interpret(Mov(Lit(U16), Mem)),
            // Move register value into a specific register
            MOV_REG_REG => self.interpret(Mov(Reg, Reg)),
            // Move register value into a specific memory address
            MOV_REG_MEM => self.interpret(Mov(Reg, Mem)),
            // Move memory value into a specific register
            MOV_MEM_REG => self.interpret(Mov(Mem, Reg)),
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
                    x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x))),
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
                            x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x))),
                        }
                    }
                    x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x))),
                }
            }
            // Move value from memory address = [literal + register] to register
            MOV_LITOFF_REG => {
                let base_address = self.fetch_mem()?;
                let r1 = self.fetch_reg()?;
                let r2 = self.fetch_reg()?;

                match SIZE_OF[r1] {
                    1 => Err(ExecutionError::BadRegisterPtrLen),
                    2 => {
                        let offset = self.registers.get_memory_at_u16(r1)? as usize;
                        let val = self.memory.get_memory_at_u16(base_address + offset)?;

                        #[cfg(debug_assertions)]
                        {
                            let r2_name = REGISTER_NAMES[r2];
                            println!(
                                "Move value {:#06X} from {:#06X} in memory to {}",
                                val,
                                base_address + offset,
                                r2_name
                            );
                        }

                        flag!(self, val);
                        match SIZE_OF[r2] {
                            1 => Ok(self.registers.set_memory_at_u8(ADDRESS_OF[r2], val as u8)?),
                            2 => Ok(self.registers.set_memory_at_u16(ADDRESS_OF[r2], val)?),
                            x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x))),
                        }
                    }
                    x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x))),
                }
            }
            // unconditional jump to literal (label)
            JMP_LIT => {
                let add = self.fetch_literal_u16()?;

                #[cfg(debug_assertions)]
                println!("Jump to {:#06X} (literal)", add);

                self.set_register("ip", add)?;
                Ok(())
            }
            // unconditional jump to register value
            JMP_REG => {
                let reg = self.fetch_reg()?;
                let add = register!(self, reg)?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!("Jump to {:#06X} (value of {})", add, reg_name);
                }

                self.set_register("ip", add)?;
                Ok(())
            }
            // Jump to provided memory address if Zero_f is true
            JEQ_LIT => {
                let add = self.fetch_literal_u16()?;

                #[cfg(debug_assertions)]
                println!("Jump to {:#06X} (literal) if flag ZERO is set to true", add);

                if (self.flags & F_ZERO_VAL) != 0 {
                    // flag f_zero_val is on
                    self.set_register("ip", add)?;
                }
                Ok(())
            }
            // Jump to the value in register if Zero_f is true
            JEQ_REG => {
                let reg = self.fetch_reg()?;
                let add = register!(self, reg)?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!(
                        "Jump to {:#06X} (value of {}) if flag ZERO is set to true",
                        add, reg_name
                    );
                }

                if (self.flags & F_ZERO_VAL) != 0 {
                    // flag f_zero_val is on
                    self.set_register("ip", add)?;
                }
                Ok(())
            }
            // Jump to provided memory address if Zero_f is false
            JNE_LIT => {
                let add = self.fetch_literal_u16()?;

                #[cfg(debug_assertions)]
                println!(
                    "Jump to {:#06X} (literal) if flag ZERO is set to false",
                    add
                );

                if (self.flags & F_ZERO_VAL) == 0 {
                    // flag f_zero_val is off
                    self.set_register("ip", add)?;
                }
                Ok(())
            }
            // Jump to the value in register if Zero_f is false
            JNE_REG => {
                let reg = self.fetch_reg()?;
                let add = register!(self, reg)?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!(
                        "Jump to {:#06X} (value of {}) if flag ZERO is set to false",
                        add, reg_name
                    );
                }

                if (self.flags & F_ZERO_VAL) == 0 {
                    // flag f_zero_val is off
                    self.set_register("ip", add)?;
                }
                Ok(())
            }
            // Jump to provided memory address if Zero_f and Neg_f are false
            JGT_LIT => {
                let add = self.fetch_literal_u16()?;

                #[cfg(debug_assertions)]
                println!(
                    "Jump to {:#06X} (literal) if flags ZERO and NEGATIF are set to false",
                    add
                );

                if (self.flags & (F_ZERO_VAL | F_NEGATIF)) == 0 {
                    self.set_register("ip", add)?;
                }
                Ok(())
            }
            // Jump to the value in register if Zero_f and Neg_f are false
            JGT_REG => {
                let reg = self.fetch_reg()?;
                let add = register!(self, reg)?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!(
                        "Jump to {:#06X} (value of {}) if flags ZERO and NEGATIF are set to false",
                        add, reg_name
                    );
                }

                if (self.flags & (F_ZERO_VAL | F_NEGATIF)) == 0 {
                    self.set_register("ip", add)?;
                }
                Ok(())
            }
            // Jump to provided memory address if Neg_f is false
            JGE_LIT => {
                let add = self.fetch_literal_u16()?;

                #[cfg(debug_assertions)]
                println!(
                    "Jump to {:#06X} (literal) if flag NEGATIF is set to false",
                    add
                );

                if (self.flags & F_NEGATIF) == 0 {
                    self.set_register("ip", add)?;
                }
                Ok(())
            }
            // Jump to the value in register if Neg_f is false
            JGE_REG => {
                let reg = self.fetch_reg()?;
                let add = register!(self, reg)?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!(
                        "Jump to {:#06X} (value of {}) if flag NEGATIF is set to false",
                        add, reg_name
                    );
                }

                if (self.flags & F_NEGATIF) == 0 {
                    self.set_register("ip", add)?;
                }
                Ok(())
            }
            // Jump to provided memory address if Zero_f is false and Neg_f is true
            JLT_LIT => {
                let add = self.fetch_literal_u16()?;

                #[cfg(debug_assertions)]
                println!("Jump to {:#06X} (literal) if flag ZERO is set to false and flag NEGATIF is set to true", add);

                if (self.flags & (F_ZERO_VAL | F_NEGATIF)) == F_NEGATIF {
                    // not equal + neg
                    self.set_register("ip", add)?;
                }
                Ok(())
            }
            // Jump to the value in register if Zero_f is false and Neg_f is true
            JLT_REG => {
                let reg = self.fetch_reg()?;
                let add = register!(self, reg)?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!("Jump to {:#06X} (value of {}) if flag ZERO is set to false and flag NEGATIF is set to true", add, reg_name);
                }

                if (self.flags & (F_ZERO_VAL | F_NEGATIF)) == F_NEGATIF {
                    // not equal + neg
                    self.set_register("ip", add)?;
                }
                Ok(())
            }
            // Jump to provided memory address if Zero_f and Neg_f are true
            JLE_LIT => {
                let add = self.fetch_literal_u16()?;

                #[cfg(debug_assertions)]
                println!(
                    "Jump to {:#06X} (literal) if flag NEGATIF and ZERO are set to true",
                    add
                );

                if (self.flags & (F_NEGATIF | F_ZERO_VAL)) != 0 {
                    self.set_register("ip", add)?;
                }
                Ok(())
            }
            // Jump to the value in register address if Zero_f and Neg_f are true
            JLE_REG => {
                let reg = self.fetch_reg()?;
                let add = register!(self, reg)?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!(
                        "Jump to {:#06X} (value of {}) if flag NEGATIF and ZERO are set to true",
                        add, reg_name
                    );
                }

                if (self.flags & (F_NEGATIF | F_ZERO_VAL)) != 0 {
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
                let val = self.fetch_literal_u16()?;

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
                let val = self.fetch_literal_u16()?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!(
                        "Substract {} from {:#06X}, store result in ACC",
                        reg_name, val
                    );
                }

                let reg_val = register!(self, reg)?;
                let (res, carry) = val.overflowing_sub(reg_val);
                flag!(self, res, carry);

                Ok(self.set_register("acc", res)?)
            }
            // Substract register with literal
            SUB_LIT_REG => {
                let val = self.fetch_literal_u16()?;
                let reg = self.fetch_reg()?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!(
                        "Substract {:#06X} from {}, store result in ACC",
                        val, reg_name
                    );
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
                let val = self.fetch_literal_u16()?;

                #[cfg(debug_assertions)]
                {
                    let reg_name = REGISTER_NAMES[reg];
                    println!(
                        "Multiply {} and {:#06X}, store result in ACC",
                        reg_name, val
                    );
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
                let lit = self.fetch_literal_u16()?;

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
                let value = self.fetch_literal_u16()?;

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
                let memory_add = self.fetch_mem()?;
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
                let memory_add = self.fetch_mem()?;
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
                    x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x))),
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
                    x => Err(ExecutionError::from(MemoryError::BadRegisterLen(x))),
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
                let memory_add = self.fetch_mem()?;
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
                let memory_add = self.fetch_mem()?;
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
                let address = self.fetch_literal_u16()?;

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
            // Left shift register with other register
            LSF_REG_REG => {
                let r1 = self.fetch_reg()?;
                let r2 = self.fetch_reg()?;

                #[cfg(debug_assertions)]
                {
                    let r1n = REGISTER_NAMES[r1];
                    let r2n = REGISTER_NAMES[r2];
                    println!("Left shift {} and {}, in {}", r1n, r2n, r1n);
                }

                let r1_value = register!(self, r1)?;
                let r2_value = register!(self, r2)?;
                let res = r1_value << r2_value;

                flag!(self, res);
                Ok(register!(self, r1 => res)?)
            }
            // Left shift register with literal
            LSF_REG_LIT => {
                let r1 = self.fetch_reg()?;
                let literal = self.fetch_literal_u16()?;

                #[cfg(debug_assertions)]
                {
                    let r1n = REGISTER_NAMES[r1];
                    println!("Left shift {} and {:#06X}, in {}", r1n, literal, r1n);
                }

                let val = register!(self, r1)?;
                let res = val << literal;

                flag!(self, res);
                Ok(register!(self, r1 => res)?)
            }
            // Right shift register with other register
            RSF_REG_REG => {
                let r1 = self.fetch_reg()?;
                let r2 = self.fetch_reg()?;

                #[cfg(debug_assertions)]
                {
                    let r1n = REGISTER_NAMES[r1];
                    let r2n = REGISTER_NAMES[r2];
                    println!("Right shift {} and {}, in {}", r1n, r2n, r1n);
                }

                let r1_value = register!(self, r1)?;
                let r2_value = register!(self, r2)?;
                let res = r1_value >> r2_value;

                flag!(self, res);
                Ok(register!(self, r1 => res)?)
            }
            // Right shift register with literal
            RSF_REG_LIT => {
                let r1 = self.fetch_reg()?;
                let literal = self.fetch_literal_u16()?;

                #[cfg(debug_assertions)]
                {
                    let r1n = REGISTER_NAMES[r1];
                    println!("Right shift {} and {:#06X}, in {}", r1n, literal, r1n);
                }

                let val = register!(self, r1)?;
                let res = val >> literal;

                flag!(self, res);
                Ok(register!(self, r1 => res)?)
            }
            // AND register with other register
            AND_REG_REG => {
                let r1 = self.fetch_reg()?;
                let r2 = self.fetch_reg()?;

                #[cfg(debug_assertions)]
                {
                    let r1n = REGISTER_NAMES[r1];
                    let r2n = REGISTER_NAMES[r2];
                    println!("AND {} and {}, in {}", r1n, r2n, r1n);
                }

                let r1_value = register!(self, r1)?;
                let r2_value = register!(self, r2)?;
                let res = r1_value & r2_value;

                flag!(self, res);
                Ok(register!(self, r1 => res)?)
            }
            // AND register with literal
            AND_REG_LIT => {
                let r1 = self.fetch_reg()?;
                let literal = self.fetch_literal_u16()?;

                #[cfg(debug_assertions)]
                {
                    let r1n = REGISTER_NAMES[r1];
                    println!("AND {} and {:#06X}, in {}", r1n, literal, r1n);
                }

                let val = register!(self, r1)?;
                let res = val & literal;

                flag!(self, res);
                Ok(register!(self, r1 => res)?)
            }
            // OR register with other register
            OR_REG_REG => {
                let r1 = self.fetch_reg()?;
                let r2 = self.fetch_reg()?;

                #[cfg(debug_assertions)]
                {
                    let r1n = REGISTER_NAMES[r1];
                    let r2n = REGISTER_NAMES[r2];
                    println!("OR {} and {}, in {}", r1n, r2n, r1n);
                }

                let r1_value = register!(self, r1)?;
                let r2_value = register!(self, r2)?;
                let res = r1_value | r2_value;

                flag!(self, res);
                Ok(register!(self, r1 => res)?)
            }
            // OR register with literal
            OR_REG_LIT => {
                let r1 = self.fetch_reg()?;
                let literal = self.fetch_literal_u16()?;

                #[cfg(debug_assertions)]
                {
                    let r1n = REGISTER_NAMES[r1];
                    println!("OR {} and {:#06X}, in {}", r1n, literal, r1n);
                }

                let val = register!(self, r1)?;
                let res = val | literal;

                flag!(self, res);
                Ok(register!(self, r1 => res)?)
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
                let literal = self.fetch_literal_u16()?;

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
            // NOT register in place
            NOT => {
                let r1 = self.fetch_reg()?;

                #[cfg(debug_assertions)]
                {
                    let r1n = REGISTER_NAMES[r1];
                    println!("NOT {}, in {}", r1n, r1n);
                }

                let val = register!(self, r1)?;
                let res = !val;

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
}
