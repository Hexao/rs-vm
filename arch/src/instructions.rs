pub const MOV_LIT_REG   : u8 = 0x10;
pub const MOV_LIT_MEM   : u8 = 0x11;
pub const MOV_REG_REG   : u8 = 0x12;
pub const MOV_REG_MEM   : u8 = 0x13;
pub const MOV_MEM_REG   : u8 = 0x14;
pub const MOV_PTRREG_REG: u8 = 0x15;
pub const MOV_REG_PTRREG: u8 = 0x16;

pub const ADD_REG_REG   : u8 = 0x20;

pub const JMP_LIT       : u8 = 0x30;
pub const JNE_LIT_LIT   : u8 = 0x31; // jump to second lit if first lit != acc

pub const PSH_LIT       : u8 = 0x40;
pub const PSH_REG       : u8 = 0x41;
pub const POP_REG       : u8 = 0x42;

pub const CALL_LIT      : u8 = 0x50;
pub const CALL_REG      : u8 = 0x51;
pub const RET           : u8 = 0x52;

pub const XOR_REG_REG   : u8 = 0x60;
pub const XOR_REG_LIT   : u8 = 0x61;

pub const END           : u8 = 0xFF;
