pub const REGISTER_NAMES: &[&str] = &[
    "ip", "acc",
    "ah", "al", "ax",
    "bh", "bl", "bx",
    "ch", "cl", "cx",
    "dh", "dl", "dx",
    "ex", "fx", "gx", "hx",
    "sp", "fp",
];

pub const SIZE_OF: &[u8] = &[
    2, 2,
    1, 1, 2,
    1, 1, 2,
    1, 1, 2,
    1, 1, 2,
    2, 2, 2, 2,
    2, 2,
];

pub const ADDRESS_OF: &[usize] = &[
    0, 2,
    4, 5, 4,
    6, 7, 6,
    8, 9, 8,
    10, 11, 10,
    12, 14, 16, 18,
    20, 22,
];

pub const REGISTER_LEN: usize = 24;

pub const IP : u8 = 0;
pub const ACC: u8 = 1;
pub const AH : u8 = 2;
pub const AL : u8 = 3;
pub const AX : u8 = 4;
pub const BH : u8 = 5;
pub const BL : u8 = 6;
pub const BX : u8 = 7;
pub const CH : u8 = 8;
pub const CL : u8 = 9;
pub const CX : u8 = 10;
pub const DH : u8 = 11;
pub const DL : u8 = 12;
pub const DX : u8 = 13;
pub const EX : u8 = 14;
pub const FX : u8 = 15;
pub const GX : u8 = 16;
pub const HX : u8 = 17;
pub const SP : u8 = 18;
pub const FP : u8 = 19;
