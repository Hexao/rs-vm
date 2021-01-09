pub mod component;
mod test;

use crate::component::cpu::CPU;
use arch::{
    instructions::*,
    register::{R1, R2, R3, R4},
};

fn main() {
    let mut cpu = CPU::new(0xFF);
    cpu.print_registers();

    let instructions = [
        MOV_LIT_REG, 0x11, 0x11, R1, // 0x0000
        MOV_LIT_REG, 0x33, 0x33, R3, // 0x0004
        PSH_LIT, 0x22, 0x22,         // 0x0008
        CALL_LIT, 0x00, 0x18,        // 0x000B
        POP_REG, R2,                 // 0x000E
        END,                         // 0x0010
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, // next octet is 0x0018
        PSH_LIT, 0xAB, 0xCD,         // 0x0018
        PSH_LIT, 0x12, 0x34,         // 0x001B
        MOV_LIT_REG, 0xFF, 0xFF, R2, // 0x001E
        MOV_LIT_REG, 0xFF, 0xFF, R4, // 0x0022
        RET,                         // 0x0026
    ];

    cpu.set_instruction(&instructions);
    while cpu.step() {
        cpu.print_registers();
        cpu.print_memory_chunk_u16(0xE1, 0xFF);
    }
}
