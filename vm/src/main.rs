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
    let path = format!("{}{}.vmo", dir, args.source);

    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Can't open file {path}: {e}");
            return;
        }
    };

    if let Err(e) = file.read_to_end(&mut instructions) {
        eprintln!("Error while reading file {path}: {e}");
        return;
    };

    // cpu.print_registers();
    let start = std::time::Instant::now();

    cpu.set_instruction(&instructions);
    while cpu.step() {
        // cpu.print_registers();
        // cpu.print_memory_chunk_u16(0x3000, 0x3020);
    }

    let dur = start.elapsed().as_secs_f32();
    println!("\nExecuted in {:.3} sec", dur);
}
