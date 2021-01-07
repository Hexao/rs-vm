/// Memory struct is the physical representation of the VM
pub struct Memory {
    /// The data vector that is our memory
    data: Vec<u8>,
}

impl Memory {
    /// Creates a new Memory struct with a provided size of memory
    /// 
    /// # Arguments
    /// 
    /// * `size` - The total size of the memory data
    /// 
    /// # Examples
    /// 
    /// ```
    /// pub mod component;
    /// use crate::component::memory::Memory;
    /// 
    /// let m = Memory::new(0x40);
    /// assert_eq!(m.len(), 64);
    /// ```
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }

    /// Get the memory cell from the `data` given a `location`
    pub fn get_memory_at(&self, location: usize) -> Result<u8, MemoryError> {
        #[cfg(debug_assertions)]
        if location >= self.data.len() {
            return Err(MemoryError::OutOfBounds(location));
        }

        Ok(self.data[location])
    }

    /// Set the memory cell from the `data` given a `location`
    pub fn set_memory_at(&mut self, location: usize, value: u8) -> Result<(), MemoryError> {
        #[cfg(debug_assertions)]
        if location >= self.data.len() {
            return Err(MemoryError::OutOfBounds(location));
        }

        self.data[location] = value;
        Ok(())
    }

    /// Return the size of the memory allocated
    pub fn len(&self) -> usize {
        self.data.len()
    }
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
