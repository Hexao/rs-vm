pub mod component;
mod test;

use crate::component::cpu::CPU;
use arch::{
    instructions::*,
    register::{R1, R2},
};

fn main() {
    let mut cpu = CPU::new(0xFF);
    cpu.print_registers();

    let instructions = [
        MOV_LIT_REG, 0x12, 0x34, R1, // move 0x1234 in r1 (16 bit)
        MOV_LIT_REG, 0xAB, 0xCD, R2, // move 0xABCD in r2 (16 bit)
        PSH_REG,     R1,             // push value on R1 on the stack
        PSH_REG,     R2,             // push value on R2 on the stack
        POP_REG,     R1,             // pop value from the stack to R1
        POP_REG,     R2,             // pop value from the stack to R2
        END,                         // stop program
    ];

    cpu.set_instruction(&instructions);
    while cpu.step() {
        cpu.print_registers();
        cpu.print_memory_chunk_u16(0xF7, 0xFF);
    }
}
