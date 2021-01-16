use crate::component::memory_io::MemoryIO;
use crate::component::memory::Memory;
use super::memory_io::MemoryError;

struct Region {
    device: Box<dyn MemoryIO>,
    start: usize,
    end: usize,
}

impl Region {
    fn new(device: Box<dyn MemoryIO>, start: usize) -> Self {
        let len = device.len();
        let end = start + len;

        if end - 1 > 0xFFFF {
            panic!(
                "You can't create a region how will overflow memory.\nRegion maped from {:#X} to {:#X}",
                start, end
            );
        }

        Self { device, start, end }
    }

    fn contain(&self, address: usize) -> Option<usize> {
        if address >= self.start && address < self.end {
            Some(address - self.start)
        } else {
            None
        }
    }
}

pub struct MemoryMap {
    regions: Vec<Region>,
}

impl MemoryMap {
    pub fn add_device(&mut self, device: Box<dyn MemoryIO>, start: usize) {
        let reg = Region::new(device, start);
        self.regions.push(reg);
    }

    pub fn get_memory_at_u8(&self, location: usize) -> Result<u8, MemoryError> {
        let (reg, address) = self.find_region(location)?;
        reg.device.get_memory_at_u8(address)
    }

    pub fn get_memory_at_u16(&self, location: usize) -> Result<u16, MemoryError> {
        let (reg, address) = self.find_region(location)?;
        reg.device.get_memory_at_u16(address)
    }

    pub fn set_memory_at_u8(&mut self, location: usize, data: u8) -> Result<(), MemoryError> {
        let (reg, address) = self.find_region_mut(location)?;
        reg.device.set_memory_at_u8(address, data)
    }

    pub fn set_memory_at_u16(&mut self, location: usize, data: u16) -> Result<(), MemoryError> {
        let (reg, address) = self.find_region_mut(location)?;
        reg.device.set_memory_at_u16(address, data)
    }

    fn find_region(&self, address: usize) -> Result<(&Region, usize), MemoryError> {
        for reg in self.regions.iter().rev() {
            match reg.contain(address) {
                Some(address) => return Ok((reg, address)),
                _ => (),
            }
        }

        Err(MemoryError::OutOfBounds(address))
    }

    fn find_region_mut(&mut self, address: usize) -> Result<(&mut Region, usize), MemoryError> {
        for reg in self.regions.iter_mut().rev() {
            match reg.contain(address) {
                Some(address) => return Ok((reg, address)),
                _ => (),
            }
        }

        Err(MemoryError::OutOfBounds(address))
    }

    pub fn len(&self) -> usize {
        0x1_0000
    }

    pub fn is_empty(&self) -> bool {
        false
    }
}

impl Default for MemoryMap {
    fn default() -> Self {
        // memory covers all addressable bytes
        let memory = Memory::new(0x1_0000);

        Self {
            regions: vec![Region::new(
                Box::new(memory), 0x00
            )]
        }
    }
}
