pub trait MemoryIO {
    fn get_memory_at_u8(&self, location: usize) -> Result<u8, MemoryError>;
    fn get_memory_at_u16(&self, location: usize) -> Result<u16, MemoryError>;
    fn set_memory_at_u8(&mut self, location: usize, data: u8) -> Result<(), MemoryError>;
    fn set_memory_at_u16(&mut self, location: usize, data: u16) -> Result<(), MemoryError>;
}

/// Enumeration of every type of memory error
pub enum MemoryError {
    OutOfBounds(usize),
}

impl std::fmt::Debug for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            MemoryError::OutOfBounds(address) => {
                format!("The address {:#04X} is not in the memory", address)
            }
        };

        write!(f, "{}", error)
    }
}
