use crate::component::cpu::ExecutionError;
use crate::component::cpu::CPU;
use crate::component::memory_io::{MemoryError, MemoryIO};

use arch::registers::*;

use super::composition::Action;
use super::composition::Instructions;
use super::composition::Operand;

impl CPU {
    pub fn set_register_u16(&mut self, reg: usize, data: u16) -> Result<(), ExecutionError> {
        match SIZE_OF[reg] {
            1 => self.registers.set_memory_at_u8(ADDRESS_OF[reg], data as u8),
            2 => self.registers.set_memory_at_u16(ADDRESS_OF[reg], data),
            x => Err(MemoryError::BadRegisterLen(x)),
        }
        .map_err(Into::into)
    }

    pub fn set_register_u8(&mut self, reg: usize, data: u8) -> Result<(), ExecutionError> {
        match SIZE_OF[reg] {
            1 => self.registers.set_memory_at_u8(ADDRESS_OF[reg], data),
            2 => self
                .registers
                .set_memory_at_u16(ADDRESS_OF[reg], data as u16),
            x => Err(MemoryError::BadRegisterLen(x)),
        }
        .map_err(Into::into)
    }

    pub(super) fn interpret(&mut self, instruction: Instructions) -> Result<(), ExecutionError> {
        let action = instruction.fetch(self)?;

        match action {
            Action::Mov(a, b) => self.mov(a, b),
        }
    }

    fn mov(&mut self, a: Operand, b: Operand) -> Result<(), ExecutionError> {
        match (a, b) {
            (Operand::Lit(value), Operand::Reg(reg)) => value.set_register(self, reg),
            (Operand::Lit(value), Operand::Mem(mem)) => value.set_memory(self, mem),
            (Operand::Reg(reg), Operand::Mem(mem)) => reg.set_memory(self, mem),
            (Operand::Mem(mem), Operand::Reg(reg)) => mem.set_register(self, reg),
            (Operand::Reg(reg), Operand::Reg(reg2)) => reg.set_register(self, reg2),
            _ => Ok(()),
        }
    }
}
