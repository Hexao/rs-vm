use arch::registers::{ADDRESS_OF, SIZE_OF};

use crate::{
    component::{
        cpu::{ExecutionError, CPU},
        memory_io::{MemoryError, MemoryIO},
    },
    flag,
};

use super::{memory::RegisterPtr, MemoryKind};

pub enum RegisterKind {
    U8(u8, usize),
    U16(u16, usize),
    PTR(RegisterPtr, usize),
}

impl RegisterKind {
    pub fn new(cpu: &mut CPU, is_ptr: bool) -> Result<Self, ExecutionError> {
        let reg = cpu.fetch_reg()?;
        match SIZE_OF[reg] {
            1 => {
                if is_ptr {
                    Err(ExecutionError::BadRegisterPtrLen)
                } else {
                    Ok(RegisterKind::U8(
                        cpu.registers.get_memory_at_u8(ADDRESS_OF[reg])?,
                        reg,
                    ))
                }
            }
            2 => {
                if is_ptr {
                    Ok(RegisterKind::PTR(RegisterPtr::new(cpu, reg)?, reg))
                } else {
                    let value = cpu.registers.get_memory_at_u16(ADDRESS_OF[reg])?;
                    Ok(RegisterKind::U16(value, reg))
                }
            }
            x => Err(MemoryError::BadRegisterLen(x).into()),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            RegisterKind::U8(_, reg) => super::REGISTER_NAMES[*reg],
            RegisterKind::U16(_, reg) => super::REGISTER_NAMES[*reg],
            RegisterKind::PTR(_, reg) => super::REGISTER_NAMES[*reg],
        }
    }

    pub fn register(&self) -> usize {
        match self {
            RegisterKind::U8(_, reg) => *reg,
            RegisterKind::U16(_, reg) => *reg,
            RegisterKind::PTR(_, reg) => *reg,
        }
    }

    /// Set the memory with the value in the register
    pub fn set_memory(self, cpu: &mut CPU, mem: MemoryKind) -> Result<(), ExecutionError> {
        let mem = mem.location();
        match self {
            RegisterKind::U8(val, _) => {
                flag!(cpu, val);
                println!("\tSet memory at {:#06X} to {:#04X}", mem, val);
                cpu.memory.set_memory_at_u8(mem, val)
            }
            RegisterKind::U16(val, _) => {
                flag!(cpu, val);
                println!("\tSet memory at {:#06X} to {:#06X}", mem, val);
                cpu.memory.set_memory_at_u16(mem, val)
            }
            RegisterKind::PTR(_, _) => unimplemented!("PTR register"),
        }
        .map_err(Into::into)
    }

    /// Set the other register with the value in the current register
    pub fn set_register(self, cpu: &mut CPU, reg: RegisterKind) -> Result<(), ExecutionError> {
        let reg_nb = reg.register();
        match self {
            RegisterKind::U8(val, _) => {
                flag!(cpu, val);
                println!("\tSet register {} to {:#04X}", reg.name(), val);
                cpu.set_register_u8(reg_nb, val)
            }
            RegisterKind::U16(val, _) => {
                flag!(cpu, val);
                println!("\tSet register {} to {:#06X}", reg.name(), val);
                cpu.set_register_u16(reg_nb, val)
            }
            RegisterKind::PTR(reg_ptr, _) => {
                let mem: MemoryKind = reg_ptr.into();
                println!(
                    "\tSet register {} to value pointed by memory {:#06X}",
                    reg.name(),
                    mem.location()
                );
                mem.set_register(cpu, reg)
            }
        }
    }
}
