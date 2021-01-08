pub mod component;
mod test;

use crate::component::cpu::CPU;
use arch::{
    instructions::*,
    register::{ACC, R1, R2},
};

fn main() {
    let mut cpu = CPU::new(0x100 * 0x100);
    cpu.print_registers();

    let instructions = [
        MOV_LIT_REG,  0x12, 0x34, R1,   // move 0xFFFF in r1 (16 bit)
        MOV_LIT_REG,  0xAB, 0xCD, R2,   // move 0x0001 in r2 (16 bit)
        MOV_LIT_REG,  0x00, 0x4F, ACC,  // fill ACC with non-null value
        ADD_REG_REG,    R1,   R2,       // add r1 and r2
        MOV_REG_MEM,   ACC, 0x01, 0x00, // store accumulator value in memory address 0x0100
        END,                            // stop the program
    ];

    cpu.set_instruction(&instructions);
    while cpu.step() {
        cpu.print_registers();
        cpu.print_memory_chunk_u16(0x0100, 0x0114);
    }
}
