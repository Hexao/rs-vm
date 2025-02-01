use crate::component::cpu::ExecutionError;
use crate::component::cpu::CPU;
use crate::component::memory_io::{MemoryError, MemoryIO};

use arch::flags::*;
use arch::registers::*;

use crate::{flag, register};

pub enum Kind {
    U8,
    U16,
    Usize,
}

enum FilledParameters {
    Reg(usize),
    Mem(usize),
    Lit(Value),
}

enum Value {
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

    pub fn set_memory(self, cpu: &mut CPU, mem: usize) -> Result<(), ExecutionError> {
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

    pub fn set_register(self, cpu: &mut CPU, reg: usize) -> Result<(), ExecutionError> {
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

impl Kind {
    fn fetch(&self, cpu: &mut CPU) -> Result<Value, ExecutionError> {
        Value::new(self, cpu)
    }
}

pub enum Parameters {
    Reg,
    Mem,
    Lit(Kind),
}

impl Parameters {
    fn fetch(&self, cpu: &mut CPU) -> Result<FilledParameters, ExecutionError> {
        Ok(match self {
            Parameters::Reg => {
                let reg = cpu.fetch_reg()?;
                let reg_name = REGISTER_NAMES[reg];
                print!("{reg_name}");
                FilledParameters::Reg(reg)
            }
            Parameters::Mem => {
                let mem = cpu.fetch_mem()?;
                print!("{mem:#06X} (memory)");
                FilledParameters::Mem(mem)
            }
            Parameters::Lit(kind) => {
                let value = kind.fetch(cpu)?;
                print!("(literal)");
                FilledParameters::Lit(value)
            }
        })
    }
}

enum Action {
    Mov(FilledParameters, FilledParameters),
}

pub enum Instructions {
    Mov(Parameters, Parameters),
}

struct MemoryKind(usize);

impl MemoryKind {
    pub fn set_register(self, cpu: &mut CPU, reg: RegisterKind) -> Result<(), ExecutionError> {
        match reg {
            RegisterKind::U8(_, reg) => {
                let val = cpu.memory.get_memory_at_u8(self.0)?;
                flag!(cpu, val);
                println!("Set register {} with {:#04X}", REGISTER_NAMES[reg], val);
                cpu.set_register_u8(reg, val)
            }
            RegisterKind::U16(_, reg) => {
                let val = cpu.memory.get_memory_at_u16(self.0)?;
                flag!(cpu, val);
                println!("Set register {} with {:#06X}", REGISTER_NAMES[reg], val);
                cpu.set_register_u16(reg, val)
            }
        }
    }
}

pub enum RegisterKind {
    U8(u8, usize),
    U16(u16, usize),
}

impl RegisterKind {
    pub fn set_memory(self, cpu: &mut CPU, mem: usize) -> Result<(), ExecutionError> {
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

    pub fn set_register(self, cpu: &mut CPU, reg: usize) -> Result<(), ExecutionError> {
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

impl Instructions {
    fn fetch(&self, cpu: &mut CPU) -> Result<Action, ExecutionError> {
        Ok(match self {
            Instructions::Mov(a, b) => {
                print!("Move ");
                let a = a.fetch(cpu)?;
                print!(" in ");
                let b = b.fetch(cpu)?;
                println!();
                Action::Mov(a, b)
            }
        })
    }
}

impl CPU {
    pub fn get_register_kind(&self, reg: usize) -> Result<RegisterKind, ExecutionError> {
        match SIZE_OF[reg] {
            1 => Ok(RegisterKind::U8(
                self.registers.get_memory_at_u8(ADDRESS_OF[reg])?,
                reg,
            )),
            2 => Ok(RegisterKind::U16(
                self.registers.get_memory_at_u16(ADDRESS_OF[reg])?,
                reg,
            )),
            x => Err(MemoryError::BadRegisterLen(x).into()),
        }
    }

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

    fn mov(&mut self, a: FilledParameters, b: FilledParameters) -> Result<(), ExecutionError> {
        match (a, b) {
            (FilledParameters::Lit(value), FilledParameters::Reg(reg)) => {
                value.set_register(self, reg)
            }
            (FilledParameters::Lit(value), FilledParameters::Mem(mem)) => {
                value.set_memory(self, mem)
            }
            (FilledParameters::Reg(reg), FilledParameters::Mem(mem)) => {
                let value = self.get_register_kind(reg)?;
                value.set_memory(self, mem)
            }
            (FilledParameters::Mem(mem), FilledParameters::Reg(reg)) => {
                let mem = MemoryKind(mem);
                mem.set_register(self, self.get_register_kind(reg)?)
            }
            (FilledParameters::Reg(reg), FilledParameters::Reg(reg2)) => {
                let value = register!(self, reg)?;
                flag!(self, value);
                register!(self, reg2 => value).map_err(Into::into)
            }
            _ => Ok(()),
        }
    }
}
