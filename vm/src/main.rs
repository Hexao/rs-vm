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
    let mut cpu = CPU::default();
    let memory_capacity = 0xFF;
    let mut instructions = Vec::with_capacity(memory_capacity);
    
    let dir = "data/output/";
    let args: Args = Args::from_args();
    let mut file = File::open(format!("{}{}.vmo", dir, args.source)).unwrap();
    file.read_to_end(&mut instructions).unwrap();

    // cpu.print_registers();

    cpu.set_instruction(&instructions);
    while cpu.step() {
        // cpu.print_registers();
        // cpu.print_memory_chunk_u16(0x3000, 0x3020);
    }
}
