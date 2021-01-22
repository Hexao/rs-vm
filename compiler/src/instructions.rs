use arch::{instructions::*, registers::*};
use std::{collections::HashMap, u16, u8};

#[derive(Debug)]
pub enum Ins {
    Flag(String),
    Mov(Param, Param),
    Add(Param, Param),
    Jne(Param, Param),
    Psh(Param),
    Pop(Param),
    Cal(Param),
    Ret,
    Xor(Param, Param),
    End,
}

impl Ins {
    pub fn build_with_line(line: String) -> Result<Self, String> {
        let mut seg = line.split_whitespace();
        match seg.next() {
            Some(ins) => {
                let lower = ins.to_lowercase();
                let ins = lower.as_str();

                match ins {
                    "mov" => {
                        let p1 = Param::build_with_value(seg.next().unwrap());
                        let p2 = Param::build_with_value(seg.next().unwrap());

                        Ok(Ins::Mov(p1, p2))
                    }
                    "add" => {
                        let p1 = Param::build_with_value(seg.next().unwrap());
                        let p2 = Param::build_with_value(seg.next().unwrap());

                        Ok(Ins::Add(p1, p2))
                    }
                    "jne" => {
                        let val = Param::build_with_value(seg.next().unwrap());
                        let add = Param::build_with_value(seg.next().unwrap());

                        Ok(Ins::Jne(val, add))
                    }
                    "psh" => {
                        let val = Param::build_with_value(seg.next().unwrap());

                        Ok(Ins::Psh(val))
                    }
                    "pop" => {
                        let val = Param::build_with_value(seg.next().unwrap());

                        Ok(Ins::Pop(val))
                    }
                    "cal" => {
                        let val = Param::build_with_value(seg.next().unwrap());

                        Ok(Ins::Cal(val))
                    }
                    "ret" => Ok(Ins::Ret),
                    "xor" => {
                        let p1 = Param::build_with_value(seg.next().unwrap());
                        let p2 = Param::build_with_value(seg.next().unwrap());

                        Ok(Ins::Xor(p1, p2))
                    }
                    "end" => Ok(Ins::End),
                    _ => {
                        let ins_l = ins.len() - 1;
                        let vl = ins.get(ins_l..).unwrap();

                        if vl == ":" {
                            Ok(Ins::Flag(ins.get(0..ins_l).unwrap().to_owned()))
                        } else {
                            Err(format!("unknown keyword '{}'", ins))
                        }
                    }
                }
            }
            None => Err("expected instruction, found nothing".to_owned()),
        }
    }

    pub fn get_code(&self, jmps: &HashMap<String, u16>, vars: Option<&HashMap<String, (Vec<u8>, u16)>>, vars_add: u16) -> Result<Vec<u8>, String> {
        match self {
            // MOV_LIT_REG
            Ins::Mov(Param::Lit(lit), Param::Reg(reg)) => {
                Ok(vec![MOV_LIT_REG, (lit >> 8) as u8, (lit & 0xFF) as u8, *reg])
            }
            // MOV_LIT_MEM
            Ins::Mov(Param::Lit(lit), Param::Mem(mem)) => {
                Ok(vec![MOV_LIT_MEM, (lit >> 8) as u8, (lit & 0xFF) as u8, (mem >> 8) as u8, (mem & 0xFF) as u8])
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
            Ins::Mov(Param::Flag(flag), Param::Reg(reg)) => {
                if let Some(vars) = vars {
                    match vars.get(flag) {
                        Some((_, add)) => {
                            let var_add = vars_add + *add;
                            Ok(vec![MOV_LIT_REG, (var_add >> 8) as u8, (var_add & 0xFF) as u8, *reg])
                        }
                        None => Err(format!("No variable with name {}", flag))
                    }
                } else {
                    Err(format!("No variable with name {}", flag))
                }
            }
            // MOV_PTR{}_REG
            Ins::Mov(Param::Ptr(ptr), Param::Reg(r2)) => match ptr.as_ref() {
                // MOV_PTRREG_REG
                Param::Reg(r1) => Ok(vec![MOV_PTRREG_REG, *r1, *r2]),
                // MOV_PTR{var}_REG => MOV_MEM_REG
                Param::Flag(flag) => {
                    if let Some(vars) = vars {
                        match vars.get(flag) {
                            Some((_, add)) => {
                                let var_add = vars_add + *add;
                                Ok(vec![MOV_MEM_REG, (var_add >> 8) as u8, (var_add & 0xFF) as u8, *r2])
                            }
                            None => Err(format!("No variable with name {}", flag))
                        }
                    } else {
                        Err(format!("No variable with name {}", flag))
                    }
                },
                p => Err(format!("no instruction MOV_PTR{}_REG on this proc", p)),
            },
            // MOV_REG_PTRREG
            Ins::Mov(Param::Reg(r1), Param::Ptr(ptr)) => match ptr.as_ref() {
                Param::Reg(r2) => Ok(vec![MOV_REG_PTRREG, *r1, *r2]),
                p => Err(format!("no instruction MOV_REG_PTR{} on this proc", p)),
            },
            // Return an error if mov operation don't existe
            Ins::Mov(p1, p2) => Err(format!("no instruction MOV_{}_{} on this proc", p1, p2)),

            // ADD_REG_REG
            Ins::Add(Param::Reg(r1), Param::Reg(r2)) => Ok(vec![ADD_REG_REG, *r1, *r2]),
            // Return an error if add operation don't existe
            Ins::Add(p1, p2) => Err(format!("no instruction ADD_{}_{} on this proc", p1, p2)),

            // JNE_LIT_flag
            Ins::Jne(Param::Lit(lit), Param::Flag(flag)) => {
                match jmps.get(flag) {
                    Some(add) => {
                        Ok(vec![JMP_NOT_EQ, (lit >> 8) as u8, (lit & 0xFF) as u8, (add >> 8) as u8, (add & 0xFF) as u8])
                    }
                    None => Err(format!("JNE: the flag {} dosen't exist", flag))
                }
            }
            // JNE_LIT_LIT
            Ins::Jne(Param::Lit(lit), Param::Lit(add)) => Ok(vec![JMP_NOT_EQ, (lit >> 8) as u8, (lit & 0xFF) as u8, (add >> 8) as u8, (add & 0xFF) as u8]),

            // PSH_LIT
            Ins::Psh(Param::Lit(lit)) => Ok(vec![PSH_LIT, (lit >> 8) as u8, (lit & 0xFF) as u8]),
            // PSH_REG
            Ins::Psh(Param::Reg(reg)) => Ok(vec![PSH_REG, *reg]),
            // Return an error if psh operation don't existe
            Ins::Psh(p) => Err(format!("no instruction PSH_{} on this proc", p)),

            // POP_REG
            Ins::Pop(Param::Reg(reg)) => Ok(vec![POP_REG, *reg]),
            // Return an error if pop operation don't existe
            Ins::Pop(p) => Err(format!("no instruction POP_{} on this proc", p)),

            // CAL_flag
            Ins::Cal(Param::Flag(flag)) => {
                match jmps.get(flag) {
                    Some(add) => {
                        Ok(vec![CALL_LIT, (add >> 8) as u8, (add & 0xFF) as u8])
                    }
                    None => Err(format!("CAL: the flag {} dosen't exist", flag))
                }
            }
            // CAL_LIT
            Ins::Cal(Param::Lit(lit)) => Ok(vec![CALL_LIT, (lit >> 8) as u8, (lit & 0xFF) as u8]),
            // CAL_REG
            Ins::Cal(Param::Reg(reg)) => Ok(vec![CALL_REG, *reg]),
            // Return an error if cal operation don't existe
            Ins::Cal(p) => Err(format!("no instruction PSH_{} on this proc", p)),

            // RET
            Ins::Ret => Ok(vec![RET]),

            // XOR_REG_REG
            Ins::Xor(Param::Reg(r1), Param::Reg(r2)) => Ok(vec![XOR_REG_REG, *r1, *r2]),
            // XOR_REG_LIT
            Ins::Xor(Param::Reg(reg), Param::Lit(lit)) => Ok(vec![XOR_REG_LIT, *reg, (lit >> 8) as u8, (lit & 0xFF) as u8]),
            // Return an error if xor operation don't existe
            Ins::Xor(p1, p2) => Err(format!("no instruction XOR_{}_{} on this proc", p1, p2)),

            // END
            Ins::End => Ok(vec![END]),
            Ins::Flag(_) => Ok(vec![]),
            ins => Err(format!("found an unknow instructions : {:?}", ins)),
        }
    }

    pub fn ins_len(&self) -> usize {
        match self {
            Ins::Flag(_) => 0,
            Ins::Mov(p1, p2) => 1 + p1.param_len() + p2.param_len(),
            Ins::Add(p1, p2) => 1 + p1.param_len() + p2.param_len(),
            Ins::Jne(p1, p2) => 1 + p1.param_len() + p2.param_len(),
            Ins::Psh(p1) => 1 + p1.param_len(),
            Ins::Pop(p1) => 1 + p1.param_len(),
            Ins::Cal(p1) => 1 + p1.param_len(),
            Ins::Ret => 1,
            Ins::Xor(p1, p2) => 1 + p1.param_len() + p2.param_len(),
            Ins::End => 1,
        }
    }
}

#[derive(Debug)]
pub enum Param {
    Flag(String),
    Ptr(Box<Param>),
    Lit(u16),
    Mem(u16),
    Reg(u8),
}

impl Param {
    pub fn build_with_value(val: &str) -> Self {
        let val = val.to_lowercase();

        let v0 = val.get(0..1).unwrap();
        let memory = v0 == "#";

        if v0 == "*" {
            return Param::Ptr(
                Box::from(Param::build_with_value(val.get(1..).unwrap()))
            )
        }

        // if val has only one char, it's a base10 literal or flag. for sure
        if val.len() < 2 {
            return match u16::from_str_radix(&val, 10) {
                Ok(v) => Param::Lit(v),
                Err(_) => Param::Flag(val),
            }
        }

        // check if val is one registers
        for (id, r) in REGISTER_NAMES.iter().enumerate() {
            if *r == val {
                return Param::Reg(id as u8);
            }
        }

        let v1_offset = if memory { 1 } else { 0 };
        let v1 = val.get(v1_offset..v1_offset + 2).unwrap();

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

    pub fn param_len(&self) -> usize {
        match self {
            Param::Flag(_) => 2,
            Param::Reg(_) => 1,
            Param::Ptr(p) => p.param_len(),
            Param::Lit(_) | Param::Mem(_) => 2,
        }
    }
}

impl std::fmt::Display for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let param = match self {
            Param::Flag(_) => "FLAG".to_owned(),
            Param::Ptr(p) => format!("PTR{}", p),
            Param::Lit(_) => "LIT".to_owned(),
            Param::Mem(_) => "MEM".to_owned(),
            Param::Reg(_) => "REG".to_owned(),
        };

        write!(f, "{}", param)
    }
}