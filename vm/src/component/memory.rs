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
    pub fn get_memory_at_u8(&self, location: usize) -> Result<u8, MemoryError> {
        #[cfg(debug_assertions)]
        if location >= self.data.len() {
            return Err(MemoryError::OutOfBounds(location));
        }

        Ok(self.data[location])
    }

    /// Get two memory cell from the `data` given a `location`
    pub fn get_memory_at_u16(&self, location: usize) -> Result<u16, MemoryError> {
        let left = self.get_memory_at_u8(location)?;
        let right = self.get_memory_at_u8(location + 1)?;

        Ok(((left as u16) << 8) + (right as u16))
    }

    /// Set the memory cell from the `data` given a `location`
    pub fn set_memory_at_u8(&mut self, location: usize, value: u8) -> Result<(), MemoryError> {
        #[cfg(debug_assertions)]
        if location >= self.data.len() {
            return Err(MemoryError::OutOfBounds(location));
        }

        self.data[location] = value;
        Ok(())
    }

    /// Set two memory cell from the `data` given a `location`
    pub fn set_memory_at_u16(&mut self, location: usize, value: u16) -> Result<(), MemoryError> {
        let left = (value >> 8) as u8;
        let right = (value % 0x100) as u8;

        self.set_memory_at_u8(location + 1, right)?;
        self.set_memory_at_u8(location, left)
    }

    /// Return the size of the memory allocated
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn print_memory_chunk_u8(&self, start: usize, end: usize) {
        let memory_len = self.data.len();
        let end = if end < memory_len { end } else { memory_len };

        print!("Memory at {:#06X} :", start);
        for address in start..end {
            match self.get_memory_at_u8(address) {
                Ok(data) if data > 0 => print!(" {:#04X}", data),
                _ => print!(" 0x--"),
            }
        }
        print!("\n")
    }

    pub fn print_memory_chunk_u16(&self, start: usize, end: usize) {
        let memory_len = self.data.len();
        let end = if end < memory_len { end } else { memory_len };

        print!("Memory at {:#06X} :", start);
        for address in (start..end).step_by(2) {
            match self.get_memory_at_u16(address) {
                Ok(data) if data > 0 => print!(" {:#06X}", data),
                _ => print!(" 0x----"),
            }
        }
        print!("\n")
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
