pub mod instruction_logic;
pub mod instruction_map;

#[macro_export]
macro_rules! register {
    ($self:ident, $reg:expr => $data:ident) => {
        match SIZE_OF[$reg] {
            1 => $self
                .registers
                .set_memory_at_u8(ADDRESS_OF[$reg], $data as u8),
            2 => $self.registers.set_memory_at_u16(ADDRESS_OF[$reg], $data),
            x => Err(MemoryError::BadRegisterLen(x)),
        }
    };

    ($self:ident, $reg:expr) => {
        match SIZE_OF[$reg] {
            1 => Ok($self.registers.get_memory_at_u8(ADDRESS_OF[$reg])? as u16),
            2 => $self.registers.get_memory_at_u16(ADDRESS_OF[$reg]),
            x => Err(MemoryError::BadRegisterLen(x)),
        }
    };
}

#[macro_export]
macro_rules! flag {
    ($self:ident, $value:ident) => {
        $self.flags = 0;
        if $value == 0 {
            $self.flags |= F_ZERO_VAL;
        }
        if $value > 0x7F {
            $self.flags |= F_NEGATIF;
        }
    };

    ($self:ident, $value:ident, $carry:ident) => {
        $self.flags = 0;
        if $value == 0 {
            $self.flags |= F_ZERO_VAL;
        }
        if $value > 0x7F {
            $self.flags |= F_NEGATIF;
        }
        if $carry {
            $self.flags |= F_CARRY;
        }
    };
}
