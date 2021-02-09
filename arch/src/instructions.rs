pub const MOV_LIT_REG   : u8 = 0x10;
pub const MOV_LIT_MEM8  : u8 = 0x11;
pub const MOV_LIT_MEM16 : u8 = 0x12;
pub const MOV_REG_REG   : u8 = 0x13;
pub const MOV_REG_MEM   : u8 = 0x14;
pub const MOV_MEM_REG   : u8 = 0x15;
pub const MOV_MEM_MEM_8 : u8 = 0x16;
pub const MOV_MEM_MEM_16: u8 = 0x17;
pub const MOV_PTRREG_REG: u8 = 0x18;
pub const MOV_REG_PTRREG: u8 = 0x19;

pub const ADD_REG_REG   : u8 = 0x20;
pub const ADD_REG_LIT   : u8 = 0x21;
pub const CMP_REG_REG   : u8 = 0x22;
pub const CMP_REG_LIT   : u8 = 0x23;
pub const INC_REG       : u8 = 0x24;
pub const DEC_REG       : u8 = 0x25;

pub const JMP_LIT       : u8 = 0x30;
pub const JEQ_LIT       : u8 = 0x31; // jump to second lit if first lit == acc
pub const JNE_LIT       : u8 = 0x32; // jump to second lit if first lit != acc
pub const JGT_LIT       : u8 = 0x33; // jump to second lit if first lit >  acc
pub const JGE_LIT       : u8 = 0x34; // jump to second lit if first lit >= acc
pub const JLT_LIT       : u8 = 0x35; // jump to second lit if first lit <  acc
pub const JLE_LIT       : u8 = 0x36; // jump to second lit if first lit <= acc

pub const PSH_LIT       : u8 = 0x40;
pub const PSH_REG       : u8 = 0x41;
pub const PSH_MEM8      : u8 = 0x42;
pub const PSH_MEM16     : u8 = 0x43;
pub const PSH_PTRREG8   : u8 = 0x44;
pub const PSH_PTRREG16  : u8 = 0x45;
pub const POP_REG       : u8 = 0x46;
pub const POP_MEM8      : u8 = 0x47;
pub const POP_MEM16     : u8 = 0x48;
pub const POP_PTRREG8   : u8 = 0x49;
pub const POP_PTRREG16  : u8 = 0x4A;

pub const CALL_LIT      : u8 = 0x50;
pub const CALL_REG      : u8 = 0x51;
pub const RET           : u8 = 0x52;

pub const XOR_REG_REG   : u8 = 0x60;
pub const XOR_REG_LIT   : u8 = 0x61;

pub const END           : u8 = 0xFF;
