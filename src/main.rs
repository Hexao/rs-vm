pub mod component;

use crate::component::memory::Memory;

fn main() {
    let m = Memory::create_memory(0x40);
    let access: [usize; 2] = [0x00, 0x4f];

    for address in access.iter() {
        match m.get_memory_at(*address) {
            Ok(value) => println!("memory at {:#04X} is {}", address, value),
            Err(memory_error) => println!("{:?}", memory_error),
        }
    }
}
