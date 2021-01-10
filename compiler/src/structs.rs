use arch::{instructions::*, registers::*};
use std::{collections::HashMap, u16, u8};

#[derive(Debug)]
pub enum Ins {
    Flag(String),
    Mov(Param, Param),
    Jne(Param, Param),
    End,
}

impl Ins {
    pub fn build_with_line(line: String) -> Option<Self> {
        let mut seg = line.split_whitespace();
        match seg.next() {
            Some(ins) => {
                let ins = &*ins.to_lowercase();

                match ins {
                    "mov" => {
                        let p1 = Param::build_with_value(seg.next().unwrap());
                        let p2 = Param::build_with_value(seg.next().unwrap());

                        Some(Ins::Mov(p1, p2))
                    }
                    "jne" => {
                        let val = Param::build_with_value(seg.next().unwrap());
                        let add = Param::build_with_value(seg.next().unwrap());

                        Some(Ins::Jne(val, add))
                    }
                    "end" => Some(Ins::End),
                    _ => {
                        let ins_l = ins.len() - 1;
                        let vl = &*ins.get(ins_l..).unwrap().to_lowercase();

                        if vl == ":" {
                            Some(Ins::Flag(ins.get(0..ins_l).unwrap().to_owned()))
                        } else {
                            None
                        }
                    }
                }
            }
            None => None,
        }
    }

    pub fn get_code(&self, register: &HashMap<String, u16>) -> Result<Vec<u8>, String> {
        match self {
            // MOV_LIT_REG
            Ins::Mov(Param::Lit(lit), Param::Reg(reg)) => {
                Ok(vec![MOV_LIT_REG, (lit >> 8) as u8, (lit & 0xFF) as u8, *reg])
            }
            // MOV_REG_REG
            Ins::Mov(Param::Reg(r1), Param::Reg(r2)) => Ok(vec![MOV_REG_REG, *r1, *r2]),
            // MOV_REG_MEM
            Ins::Mov(Param::Reg(reg), Param::Mem(mem)) => {
                Ok(vec![MOV_REG_MEM, *reg, (mem >> 8) as u8, (mem & 0xFF) as u8])
            }
            // MOV_MEM_REG
            Ins::Mov(Param::Mem(mem), Param::Reg(reg)) => {
                Ok(vec![MOV_MEM_REG, (mem >> 8) as u8, (mem & 0xFF) as u8, *reg])
            }
            // Return an error if mov operation don't existe
            Ins::Mov(p1, p2) => Err(format!("no instruction MOV_{}_{} on this proc", p1, p2)),

            Ins::Jne(Param::Lit(lit), Param::Flag(flag)) => {
                match register.get(flag) {
                    Some(add) => {
                        Ok(vec![JMP_NOT_EQ, (lit >> 8) as u8, (lit & 0xFF) as u8, (add >> 8) as u8, (add & 0xFF) as u8])
                    }
                    None => Err(format!("JNE: the flag {} dosen't exist", flag))
                }

            }

            // END
            Ins::End => Ok(vec![END]),
            ins => {
                println!("{:?}", ins);
                Ok(vec![END])
            },
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Ins::Mov(p1, p2) => 1 + p1.len() + p2.len(),
            Ins::Jne(p1, p2) => 1 + p1.len() + p2.len(),
            Ins::Flag(_) => 0,
            Ins::End => 1,
        }
    }
}

#[derive(Debug)]
pub enum Param {
    Flag(String),
    Lit(u16),
    Mem(u16),
    Reg(u8),
}

impl Param {
    pub fn build_with_value(val: &str) -> Self {
        let val = val.to_lowercase();

        // if val has only one char, it's a base10 literal. for sure
        if val.len() < 2 {
            let v = u16::from_str_radix(&val, 10).unwrap();
            return Param::Lit(v);
        }

        // check if val is one registers
        for (id, r) in REGISTER_NAMES.iter().enumerate() {
            if *r == val {
                return Param::Reg(id as u8);
            }
        }

        let v0 = &*val.get(0..1).unwrap();
        let memory = v0 == "#";

        let v1_offset = if memory { 1 } else { 0 };
        let v1 = &*val.get(v1_offset..v1_offset + 2).unwrap();

        match v1 {
            "0x" => {
                let v = u16::from_str_radix(val.get(v1_offset + 2..).unwrap(), 16).unwrap();
                if memory {
                    Param::Mem(v)
                } else {
                    Param::Lit(v)
                }
            }
            "0b" => {
                let v = u16::from_str_radix(val.get(v1_offset + 2..).unwrap(), 2).unwrap();
                if memory {
                    Param::Mem(v)
                } else {
                    Param::Lit(v)
                }
            }
            _ => match u16::from_str_radix(val.get(v1_offset..).unwrap(), 10) {
                Ok(v) => {
                    if memory {
                        Param::Mem(v)
                    } else {
                        Param::Lit(v)
                    }
                }
                Err(_) => Param::Flag(val.to_owned()),
            },
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Param::Flag(_) => 2,
            Param::Reg(_) => 1,
            Param::Lit(_) | Param::Mem(_) => 2,
        }
    }
}

impl std::fmt::Display for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let param = match self {
            Param::Flag(_) => "FLAG",
            Param::Lit(_) => "LIT",
            Param::Mem(_) => "MEM",
            Param::Reg(_) => "REG",
        };

        write!(f, "{}", param)
    }
}
