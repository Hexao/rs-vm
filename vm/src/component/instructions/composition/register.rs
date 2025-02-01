use arch::registers::{ADDRESS_OF, SIZE_OF};

use crate::{
    component::{
        cpu::{ExecutionError, CPU},
        memory_io::{MemoryError, MemoryIO},
    },
    flag,
};

use super::MemoryKind;

pub enum RegisterKind {
    U8(u8, usize),
    U16(u16, usize),
}

impl RegisterKind {
    pub fn new(cpu: &mut CPU) -> Result<Self, ExecutionError> {
        let reg = cpu.fetch_reg()?;
        match SIZE_OF[reg] {
            1 => Ok(RegisterKind::U8(
                cpu.registers.get_memory_at_u8(ADDRESS_OF[reg])?,
                reg,
            )),
            2 => Ok(RegisterKind::U16(
                cpu.registers.get_memory_at_u16(ADDRESS_OF[reg])?,
                reg,
            )),
            x => Err(MemoryError::BadRegisterLen(x).into()),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            RegisterKind::U8(_, reg) => super::REGISTER_NAMES[*reg],
            RegisterKind::U16(_, reg) => super::REGISTER_NAMES[*reg],
        }
    }

    pub fn register(&self) -> usize {
        match self {
            RegisterKind::U8(_, reg) => *reg,
            RegisterKind::U16(_, reg) => *reg,
        }
    }

    /// Set the memory with the value in the register
    pub fn set_memory(self, cpu: &mut CPU, mem: MemoryKind) -> Result<(), ExecutionError> {
        let mem = mem.location();
        match self {
            RegisterKind::U8(val, _) => {
                flag!(cpu, val);
                println!("Set memory at {:#06X} with {:#04X}", mem, val);
                cpu.memory.set_memory_at_u8(mem, val)
            }
            RegisterKind::U16(val, _) => {
                flag!(cpu, val);
                println!("Set memory at {:#06X} with {:#06X}", mem, val);
                cpu.memory.set_memory_at_u16(mem, val)
            }
        }
        .map_err(Into::into)
    }

    /// Set the other register with the value in the current register
    pub fn set_register(self, cpu: &mut CPU, reg: RegisterKind) -> Result<(), ExecutionError> {
        let reg = reg.register();
        match self {
            RegisterKind::U8(val, _) => {
                flag!(cpu, val);
                cpu.set_register_u8(reg, val)
            }
            RegisterKind::U16(val, _) => {
                flag!(cpu, val);
                cpu.set_register_u16(reg, val)
            }
        }
    }
}
