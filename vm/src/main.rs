pub mod component;
mod test;

use crate::component::cpu::CPU;
use arch::{
    instructions::*,
    register::{ACC, R1, R2},
};

fn main() {
    let mut cpu = CPU::new(0xFF);
    cpu.print_registers();

    let instructions = [
        MOV_LIT_REG,  0x00, 0x01, R2,         // move 0x0001 in r2 (16 bit)
        MOV_REG_REG,  ACC,  R1,               // store accumulator value in memory address 0x0080
        ADD_REG_REG,  R1,   R2,               // add r1 and r2
        JMP_NOT_EQ,   0x00, 0x02, 0x00, 0x04, // Jump to address 0x0000 in memory if accumulator not equal to 0x0004
        XOR_REG_REG,  R1,   R1,               // XOR register with himself to set to 0
        XOR_REG_REG,  R2,   R2,               // XOR register with himself to set to 0
        END,                                  // stop the program
    ];

    cpu.set_instruction(&instructions);
    while cpu.step() {
        cpu.print_registers();
        // cpu.print_memory_chunk_u16(0x0080, 0x0094);
    }
}
