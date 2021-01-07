pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    pub fn create_memory(capacity: usize) -> Self {
        let data = vec![0; capacity];
        Memory { data }
    }

    pub fn get_memory_at(&self, location: usize) -> Result<u8, MemoryError> {
        #[cfg(debug_assertions)]
        if location > self.data.len() {
            return Err(MemoryError::OutOfBounds(location));
        }

        Ok(self.data[location])
    }

    pub fn set_memory_at(&mut self, location: usize, value: u8) -> Result<(), MemoryError> {
        #[cfg(debug_assertions)]
        if location > self.data.len() {
            return Err(MemoryError::OutOfBounds(location));
        }

        self.data[location] = value;
        Ok(())
    }
}

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
