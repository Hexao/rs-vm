pub mod component;

//use crate::component::memory::Memory;
use crate::component::cpu::CPU;

fn main() {
    let mut cpu = CPU::new(0x40);

    cpu.set_register("r1", 0x8574).unwrap();
    cpu.set_register("r6", 0x20).unwrap();

    cpu.print_registers();
    assert_eq!(cpu.get_register("r1").unwrap(), 36000);
    /*let m = Memory::new(0x40);
    let access: [usize; 2] = [0x00, 0x39];

    for address in access.iter() {
        match m.get_memory_at(*address) {
            Ok(value) => println!("memory at {:#04X} is {}", address, value),
            Err(memory_error) => println!("{:?}", memory_error),
        }
    }*/
}
