use arch::registers::REGISTER_NAMES;

use crate::{
    component::cpu::{ExecutionError, CPU},
    flag,
};

use super::register::RegisterKind;

pub struct MemoryKind(usize);

impl MemoryKind {
    pub fn new(cpu: &mut CPU) -> Result<Self, ExecutionError> {
        Ok(MemoryKind(cpu.fetch_mem()?))
    }

    /// Get the location of the memory
    pub fn location(&self) -> usize {
        self.0
    }

    /// Set the register with the value in memory
    pub fn set_register(self, cpu: &mut CPU, reg: RegisterKind) -> Result<(), ExecutionError> {
        let mem = self.0;
        match reg {
            RegisterKind::U8(_, reg) => {
                let val = cpu.memory.get_memory_at_u8(mem)?;
                flag!(cpu, val);
                println!("Set register {} with {:#04X}", REGISTER_NAMES[reg], val);
                cpu.set_register_u8(reg, val)
            }
            RegisterKind::U16(_, reg) => {
                let val = cpu.memory.get_memory_at_u16(mem)?;
                flag!(cpu, val);
                println!("Set register {} with {:#06X}", REGISTER_NAMES[reg], val);
                cpu.set_register_u16(reg, val)
            }
        }
    }
}
