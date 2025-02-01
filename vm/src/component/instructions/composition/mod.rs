use crate::component::cpu::{ExecutionError, CPU};

pub mod memory;
pub mod register;
pub mod value;

use arch::registers::REGISTER_NAMES;
pub use memory::MemoryKind;
pub use register::RegisterKind;
pub use value::Value;

pub enum Kind {
    U8,
    U16,
    Usize,
}

impl Kind {
    fn fetch(&self, cpu: &mut CPU) -> Result<Value, ExecutionError> {
        Value::new(self, cpu)
    }
}

pub enum Operand {
    Reg(RegisterKind),
    Mem(MemoryKind),
    Lit(Value),
}

pub enum Parameters {
    Reg,
    Mem,
    Lit(Kind),
}

impl Parameters {
    fn fetch(&self, cpu: &mut CPU) -> Result<Operand, ExecutionError> {
        Ok(match self {
            Parameters::Reg => {
                let reg = RegisterKind::new(cpu)?;
                print!("{}", reg.name());
                Operand::Reg(reg)
            }
            Parameters::Mem => {
                let mem = MemoryKind::new(cpu)?;
                print!("{:#06X} (memory)", mem.location());
                Operand::Mem(mem)
            }
            Parameters::Lit(kind) => {
                let value = kind.fetch(cpu)?;
                print!("(literal)");
                Operand::Lit(value)
            }
        })
    }
}

pub enum Action {
    Mov(Operand, Operand),
}

pub enum Instructions {
    Mov(Parameters, Parameters),
}

impl Instructions {
    pub fn fetch(&self, cpu: &mut CPU) -> Result<Action, ExecutionError> {
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
