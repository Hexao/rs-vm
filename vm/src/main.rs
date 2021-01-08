pub mod component;

//use crate::component::memory::Memory;
use crate::component::cpu::CPU;
use arch::{
    instructions::*,
    register::{ACC, R1, R2},
};

fn main() {
    let mut cpu = CPU::new(0x40);
    cpu.print_registers();

    let instructions = [
        MOV_LIT_REG, 0xFF, 0xFF, R1,  // move 0xFFFF in r1 (16 bit)
        MOV_LIT_REG, 0x00, 0x01, R2,  // move 0x0001 in r2 (16 bit)
        MOV_LIT_REG, 0x00, 0x4F, ACC, // fill ACC with non-null value
        ADD_REG_REG, R1,   R2,        // add r1 and r2
        0x76,                         // stop the program
    ];

    cpu.set_instruction(&instructions);
    while cpu.step() {
        cpu.print_registers();
    }
}
