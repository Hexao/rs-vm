pub mod component;

//use crate::component::memory::Memory;
use crate::component::cpu::CPU;

fn main() {
    let mut cpu = CPU::new(0x40);

    cpu.set_register("r1", 0x8574).unwrap();
    cpu.set_register("r6", 0x20).unwrap();

    cpu.print_registers();
    assert_eq!(cpu.get_register("r1").unwrap(), 0x8574);

    let instructions = [
        0x10, 0x00, 0x02, //move 0x02 in r1 (16 bit)
        0x11, 0x00, 0x03, //move 0x03 in r2 (16 bit)
        0x12, 0x02, 0x03 // add r1 and r2
    ];

    cpu.set_instruction(&instructions);
    for _ in 0..3 {
        if let Err(e) = cpu.step() {
            println!("An error has occured: {:?}", e);
        }
        cpu.print_registers();
    }
    /*let m = Memory::new(0x40);
    let access: [usize; 2] = [0x00, 0x39];

    for address in access.iter() {
        match m.get_memory_at(*address) {
            Ok(value) => println!("memory at {:#04X} is {}", address, value),
            Err(memory_error) => println!("{:?}", memory_error),
        }
    }*/
}
