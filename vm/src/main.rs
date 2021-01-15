pub mod component;
mod test;

use std::fs::File;
use std::io::prelude::*;
use structopt::StructOpt;
use crate::component::cpu::CPU;

#[derive(StructOpt)]
pub struct Args {
    pub source: String,
}

fn main() {
    let mut cpu = CPU::new();
    let memory_capacity = 0xFF;
    let mut instructions = Vec::with_capacity(memory_capacity);
    
    let dir = "data/output/";
    let args: Args = Args::from_args();
    let mut file = File::open(format!("{}{}.vmo", dir, args.source)).unwrap();
    file.read_to_end(&mut instructions).unwrap();

    cpu.print_registers();

    // let instructions = [
    //     MOV_LIT_REG, 0x11, 0x11, R1, // 0x0000
    //     MOV_LIT_REG, 0x33, 0x33, R3, // 0x0004
    //     PSH_LIT, 0x22, 0x22,         // 0x0008
    //     CALL_LIT, 0x00, 0x18,        // 0x000B
    //     POP_REG, R2,                 // 0x000E
    //     END,                         // 0x0010
    //     0x00, 0x00, 0x00, 0x00,
    //     0x00, 0x00, 0x00, // next octet is 0x0018
    //     PSH_LIT, 0xAB, 0xCD,         // 0x0018
    //     PSH_LIT, 0x12, 0x34,         // 0x001B
    //     MOV_LIT_REG, 0xFF, 0xFF, R2, // 0x001E
    //     MOV_LIT_REG, 0xFF, 0xFF, R4, // 0x0022
    //     RET,                         // 0x0026
    // ];

    cpu.set_instruction(&instructions);
    while cpu.step() {
        cpu.print_registers();
        cpu.print_memory_chunk_u16(0xF0, 0xFF);
    }
}
