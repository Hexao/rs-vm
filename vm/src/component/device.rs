use super::{
    memory::Memory,
    memory_io::{MemoryError, MemoryIO},
    screen::Screen,
};

pub enum Device {
    Screen(Screen),
    Memory(Memory),
}

impl From<Memory> for Device {
    fn from(memory: Memory) -> Self {
        Self::Memory(memory)
    }
}

impl From<Screen> for Device {
    fn from(screen: Screen) -> Self {
        Self::Screen(screen)
    }
}

impl Default for Device {
    fn default() -> Self {
        Self::Memory(Memory::new(0x1_0000))
    }
}

impl MemoryIO for Device {
    fn get_memory_at_u8(&self, location: usize) -> Result<u8, MemoryError> {
        match self {
            Device::Screen(s) => s.get_memory_at_u8(location),
            Device::Memory(m) => m.get_memory_at_u8(location),
        }
    }

    fn get_memory_at_u16(&self, location: usize) -> Result<u16, MemoryError> {
        match self {
            Device::Screen(s) => s.get_memory_at_u16(location),
            Device::Memory(m) => m.get_memory_at_u16(location),
        }
    }

    fn set_memory_at_u8(&mut self, location: usize, data: u8) -> Result<(), MemoryError> {
        match self {
            Device::Screen(s) => s.set_memory_at_u8(location, data),
            Device::Memory(m) => m.set_memory_at_u8(location, data),
        }
    }

    fn set_memory_at_u16(&mut self, location: usize, data: u16) -> Result<(), MemoryError> {
        match self {
            Device::Screen(s) => s.set_memory_at_u16(location, data),
            Device::Memory(m) => m.set_memory_at_u16(location, data),
        }
    }

    fn len(&self) -> usize {
        match self {
            Device::Screen(s) => s.len(),
            Device::Memory(m) => m.len(),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Device::Screen(s) => s.is_empty(),
            Device::Memory(m) => m.is_empty(),
        }
    }
}
