use arch::registers::{ADDRESS_OF, REGISTER_NAMES};

use crate::{
    component::{
        cpu::{ExecutionError, CPU},
        memory_io::MemoryIO,
    },
    flag,
};

use super::register::RegisterKind;

pub struct RegisterPtr(usize, usize);

impl RegisterPtr {
    pub fn new(cpu: &mut CPU, reg: usize) -> Result<Self, ExecutionError> {
        let value = cpu.registers.get_memory_at_u16(ADDRESS_OF[reg])?;
        Ok(RegisterPtr(reg, value as usize))
    }
}

impl From<RegisterPtr> for MemoryKind {
    fn from(ptr: RegisterPtr) -> Self {
        let reg = ptr.0;
        let value = ptr.1;
        println!(
            "\tGot memory location {value:#06X} from {}",
            super::REGISTER_NAMES[reg]
        );
        MemoryKind(value)
    }
}

pub struct MemoryKind(usize);

impl MemoryKind {
    pub fn new(cpu: &mut CPU) -> Result<Self, ExecutionError> {
        Ok(MemoryKind(cpu.fetch_mem()?))
    }

    pub fn from_u16(mem: u16) -> Self {
        MemoryKind(mem as usize)
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
                println!("\tSet register {} to {:#04X}", REGISTER_NAMES[reg], val);
                cpu.set_register_u8(reg, val)
            }
            RegisterKind::U16(_, reg) => {
                let val = cpu.memory.get_memory_at_u16(mem)?;
                flag!(cpu, val);
                println!("\tSet register {} to {:#06X}", REGISTER_NAMES[reg], val);
                cpu.set_register_u16(reg, val)
            }
            RegisterKind::PTR(_, _) => unimplemented!("PTR register"),
        }
    }
}
