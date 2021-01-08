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
        MOV_MEM_REG,  0x01, 0x00, R1,           // move from 0x0100 address in memory to r1 (16 bit)
        MOV_LIT_REG,  0x00, 0x01, R2,           // move 0x0001 in r2 (16 bit)
        ADD_REG_REG,    R1,   R2,               // add r1 and r2
        MOV_REG_MEM,   ACC, 0x01, 0x00,         // store accumulator value in memory address 0x0100
        JMP_NOT_EQ,   0x00, 0x03, 0x00, 0x00,   // Jump to address 0x0000 in memory if accumulator not equal to 0x0003
        END,                                    // stop the program
    ];

    cpu.set_instruction(&instructions);
    while cpu.step() {
        cpu.print_registers();
        cpu.print_memory_chunk_u16(0x0100, 0x0114);
    }
}
