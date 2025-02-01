use crate::{
    component::cpu::{ExecutionError, CPU},
    flag,
};

use super::{Kind, MemoryKind, RegisterKind};

pub enum Value {
    U8(u8),
    U16(u16),
    Usize(usize),
}

impl Value {
    pub fn new(kind: &Kind, cpu: &mut CPU) -> Result<Self, ExecutionError> {
        Ok(match kind {
            Kind::U8 => {
                let value = cpu.fetch_literal_u8()?;
                print!("{value:#04X}");
                Value::U8(value)
            }
            Kind::U16 => {
                let value = cpu.fetch_literal_u16()?;
                print!("{value:#06X}");
                Value::U16(value)
            }
            Kind::Usize => Value::Usize(cpu.fetch_mem()?),
        })
    }

    /// Set the memory with the literal value
    pub fn set_memory(self, cpu: &mut CPU, mem: MemoryKind) -> Result<(), ExecutionError> {
        let mem = mem.location();
        match self {
            Value::U8(val) => {
                flag!(cpu, val);
                cpu.memory.set_memory_at_u8(mem, val)
            }
            Value::U16(val) => {
                flag!(cpu, val);
                cpu.memory.set_memory_at_u16(mem, val)
            }
            Value::Usize(val) => {
                flag!(cpu, val);
                cpu.memory.set_memory_at_u16(mem, val as u16)
            }
        }
        .map_err(Into::into)
    }

    /// Set the register with the literal value
    pub fn set_register(self, cpu: &mut CPU, reg: RegisterKind) -> Result<(), ExecutionError> {
        let reg = reg.register();
        match self {
            Value::U8(val) => {
                flag!(cpu, val);
                cpu.set_register_u8(reg, val)
            }
            Value::U16(val) => {
                flag!(cpu, val);
                cpu.set_register_u16(reg, val)
            }
            Value::Usize(val) => {
                flag!(cpu, val);
                let val = val as u16;
                cpu.set_register_u16(reg, val)
            }
        }
    }
}
