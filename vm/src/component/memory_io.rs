pub trait MemoryIO {
    fn get_memory_at_u8(&self, location: usize) -> Result<u8, MemoryError>;
    fn get_memory_at_u16(&self, location: usize) -> Result<u16, MemoryError>;
    fn set_memory_at_u8(&mut self, location: usize, data: u8) -> Result<(), MemoryError>;
    fn set_memory_at_u16(&mut self, location: usize, data: u16) -> Result<(), MemoryError>;

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

/// Enumeration of every type of memory error
pub enum MemoryError {
    OutOfBounds(usize),
    BadRegisterLen(u8),
    NoRegister(&'static str),
    UnaddressableRegion(usize),
    UnexpectedData(u16, usize),
}

impl std::fmt::Debug for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            MemoryError::UnaddressableRegion(add) => format!("You can't create a region that will overflow memory.\nMax region address is {add:#X} instead of 0xFFFF"),
            MemoryError::UnexpectedData(data, address) => format!("Fail to write {data:#06X} to memory address {address:#06X}"),
            MemoryError::OutOfBounds(address) => format!("The address {address:#06X} is not in the memory"),
            MemoryError::BadRegisterLen(len) => format!("Expected register size 1 or 2, found {len}"),
            MemoryError::NoRegister(name) => format!("Register {name} does not exist"),
        };

        write!(f, "{error}")
    }
}
