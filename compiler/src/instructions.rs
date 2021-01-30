use arch::{instructions::*, registers::*};
use std::{collections::HashMap, u16, u8};

#[derive(Debug)]
pub enum Ins {
    Flag(String),
    Mov(Param, Param),
    Add(Param, Param),
    Inc(Param),
    Dec(Param),
    Jmp(Param),
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
                    "inc" => {
                        let p1 = Param::build_with_value(seg.next().unwrap());

                        Ok(Ins::Inc(p1))
                    }
                    "dec" => {
                        let p1 = Param::build_with_value(seg.next().unwrap());

                        Ok(Ins::Dec(p1))
                    }
                    "jmp" => {
                        let add = Param::build_with_value(seg.next().unwrap());

                        Ok(Ins::Jmp(add))
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

    pub fn get_code(
        &self,
        jmps: &HashMap<String, u16>,
        vars: Option<&HashMap<String, (Vec<u8>, u16)>>,
        vars_add: u16,
    ) -> Result<Vec<u8>, String> {
        match self {
            // MOV_LIT_REG
            Ins::Mov(Param::Lit(lit), Param::Reg(reg)) => Ok(vec![
                MOV_LIT_REG, (lit >> 8) as u8, (lit & 0xFF) as u8, *reg
            ]),
            // MOV_LIT_MEM
            Ins::Mov(Param::Lit(lit), Param::Mem(mem)) => Ok(vec![
                MOV_LIT_MEM16, (lit >> 8) as u8, (lit & 0xFF) as u8, (mem >> 8) as u8, (mem & 0xFF) as u8
            ]),
            // MOV_LIT_PTR{}
            Ins::Mov(Param::Lit(lit), Param::Ptr(ptr)) => match ptr.as_ref() {
                // MOV_LIT_PTRflag => MOV_LIT_MEM
                Param::Flag(flag) => match vars {
                    Some(vars) => match vars.get(flag) {
                        Some((_, add)) => {
                            let var_add = vars_add + *add;
                            Ok(vec![
                                MOV_LIT_MEM16, (lit >> 8) as u8, (lit & 0xFF) as u8, (var_add >> 8) as u8, (var_add & 0xFF) as u8
                            ])
                        }
                        None => Err(format!("No variable with name {}", flag)),
                    },
                    None => Err(format!("No variable with name {}", flag)),
                },
                p => Err(format!("Found an unknow instructions : MOV_LIT_PTR{}", p)),
            },
            // MOV_REG_REG
            Ins::Mov(Param::Reg(r1), Param::Reg(r2)) => Ok(vec![MOV_REG_REG, *r1, *r2]),
            // MOV_REG_MEM
            Ins::Mov(Param::Reg(reg), Param::Mem(mem)) => Ok(vec![
                MOV_REG_MEM, *reg, (mem >> 8) as u8, (mem & 0xFF) as u8
            ]),
            // MOV_MEM_REG
            Ins::Mov(Param::Mem(mem), Param::Reg(reg)) => Ok(vec![
                MOV_MEM_REG, (mem >> 8) as u8, (mem & 0xFF) as u8, *reg
            ]),
            // MOV_flag_REG
            Ins::Mov(Param::Flag(flag), Param::Reg(reg)) => match vars {
                Some(vars) => match vars.get(flag) {
                    Some((_, add)) => {
                        let var_add = vars_add + *add;
                        Ok(vec![
                            MOV_LIT_REG,
                            (var_add >> 8) as u8,
                            (var_add & 0xFF) as u8,
                            *reg,
                        ])
                    }
                    None => Err(format!("No variable with name {}", flag)),
                },
                None => Err(format!("No variable with name {}", flag)),
            },
            // MOV_PTR{}_REG
            Ins::Mov(Param::Ptr(ptr), Param::Reg(r2)) => match ptr.as_ref() {
                // MOV_PTRREG_REG
                Param::Reg(r1) => Ok(vec![MOV_PTRREG_REG, *r1, *r2]),
                // MOV_PTR{var}_REG => MOV_MEM_REG
                Param::Flag(flag) => match vars {
                    Some(vars) => match vars.get(flag) {
                        Some((_, add)) => {
                            let var_add = vars_add + *add;
                            Ok(vec![
                                MOV_MEM_REG, (var_add >> 8) as u8, (var_add & 0xFF) as u8, *r2
                            ])
                        }
                        None => Err(format!("No variable with name {}", flag)),
                    },
                    None => Err(format!("No variable with name {}", flag)),
                },
                p => Err(format!("Found an unknow instructions : MOV_PTR{}_REG", p)),
            },
            // MOV_REG_PTR{}
            Ins::Mov(Param::Reg(r1), Param::Ptr(ptr)) => match ptr.as_ref() {
                // MOV_REG_PTRREG
                Param::Reg(r2) => Ok(vec![MOV_REG_PTRREG, *r1, *r2]),
                p => Err(format!("Found an unknow instructions : MOV_REG_PTR{}", p)),
            },

            // ADD_REG_REG
            Ins::Add(Param::Reg(r1), Param::Reg(r2)) => Ok(vec![ADD_REG_REG, *r1, *r2]),
            // ADD_REG_LIT
            Ins::Add(Param::Reg(reg), Param::Lit(lit)) => Ok(vec![ADD_REG_LIT, *reg, (lit >> 8) as u8, (lit & 0xFF) as u8]),
            // INC_REG
            Ins::Inc(Param::Reg(reg)) => Ok(vec![INC_REG, *reg]),
            // DEC_REG
            Ins::Dec(Param::Reg(reg)) => Ok(vec![DEC_REG, *reg]),

            // JMP_flag
            Ins::Jmp(Param::Flag(flag)) => match jmps.get(flag) {
                Some(add) => Ok(vec![JMP_LIT, (add >> 8) as u8, (add & 0xFF) as u8]),
                None => Err(format!("The flag {} dosen't exist", flag)),
            },
            // JMP_LIT
            Ins::Jmp(Param::Lit(add)) => Ok(vec![JMP_LIT, (add >> 8) as u8, (add & 0xFF) as u8]),

            // JNE_LIT_flag
            Ins::Jne(Param::Lit(lit), Param::Flag(flag)) => match jmps.get(flag) {
                Some(add) => Ok(vec![
                    JNE_LIT_LIT, (lit >> 8) as u8, (lit & 0xFF) as u8, (add >> 8) as u8, (add & 0xFF) as u8
                ]),
                None => Err(format!("The flag {} dosen't exist", flag)),
            },
            // JNE_LIT_LIT
            Ins::Jne(Param::Lit(lit), Param::Lit(add)) => Ok(vec![
                JNE_LIT_LIT, (lit >> 8) as u8, (lit & 0xFF) as u8, (add >> 8) as u8, (add & 0xFF) as u8
            ]),

            // PSH_LIT
            Ins::Psh(Param::Lit(lit)) => Ok(vec![PSH_LIT, (lit >> 8) as u8, (lit & 0xFF) as u8]),
            // PSH_REG
            Ins::Psh(Param::Reg(reg)) => Ok(vec![PSH_REG, *reg]),
            // PSH_MEM
            Ins::Psh(Param::Mem(mem)) => Ok(vec![PSH_LIT, (mem >> 8) as u8, (mem & 0xFF) as u8]),
            // PSH_PTR{}
            Ins::Psh(Param::Ptr(ptr)) => match ptr.as_ref() {
                // PSH_PTRREG
                Param::Reg(reg) => Ok(vec![PSH_PTRREG, *reg]),
                p => Err(format!("Found an unknow instructions : PSH_PTR{}", p)),
            },

            // POP_REG
            Ins::Pop(Param::Reg(reg)) => Ok(vec![POP_REG, *reg]),
            // POP_PTR{}
            Ins::Pop(Param::Ptr(ptr)) => match ptr.as_ref() {
                // PSH_PTRREG
                Param::Reg(reg) => Ok(vec![POP_PTRREG, *reg]),
                p => Err(format!("Found an unknow instructions : POP_PTR{}", p)),
            },

            // CAL_flag
            Ins::Cal(Param::Flag(flag)) => match jmps.get(flag) {
                Some(add) => Ok(vec![CALL_LIT, (add >> 8) as u8, (add & 0xFF) as u8]),
                None => Err(format!("The flag {} dosen't exist", flag)),
            },
            // CAL_LIT
            Ins::Cal(Param::Lit(lit)) => Ok(vec![CALL_LIT, (lit >> 8) as u8, (lit & 0xFF) as u8]),
            // CAL_REG
            Ins::Cal(Param::Reg(reg)) => Ok(vec![CALL_REG, *reg]),

            // RET
            Ins::Ret => Ok(vec![RET]),

            // XOR_REG_REG
            Ins::Xor(Param::Reg(r1), Param::Reg(r2)) => Ok(vec![XOR_REG_REG, *r1, *r2]),
            // XOR_REG_LIT
            Ins::Xor(Param::Reg(reg), Param::Lit(lit)) => Ok(vec![
                XOR_REG_LIT,
                *reg,
                (lit >> 8) as u8,
                (lit & 0xFF) as u8,
            ]),

            // END
            Ins::End => Ok(vec![END]),
            Ins::Flag(_) => Ok(vec![]),
            ins => Err(format!("Found an unknow instructions : {}", ins)),
        }
    }

    pub fn ins_len(&self) -> usize {
        match self {
            Ins::Flag(_) => 0,
            Ins::Mov(p1, p2) => 1 + p1.param_len() + p2.param_len(),
            Ins::Add(p1, p2) => 1 + p1.param_len() + p2.param_len(),
            Ins::Inc(p1) => 1 + p1.param_len(),
            Ins::Dec(p1) => 1 + p1.param_len(),
            Ins::Jmp(p1) => 1 + p1.param_len(),
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

impl std::fmt::Display for Ins {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ins::Flag(name) => write!(f, "FLAG{{{}}}", name),
            Ins::Mov(p1, p2) => write!(f, "MOV_{}_{}", p1, p2),
            Ins::Add(p1, p2) => write!(f, "ADD_{}_{}", p1, p2),
            Ins::Inc(p1) => write!(f, "INC_{}", p1),
            Ins::Dec(p1) => write!(f, "DEC_{}", p1),
            Ins::Jmp(p1) => write!(f, "JMP_{}", p1),
            Ins::Jne(p1, p2) => write!(f, "JNE_{}_{}", p1, p2),
            Ins::Psh(p1) => write!(f, "PSH_{}", p1),
            Ins::Pop(p1) => write!(f, "POP_{}", p1),
            Ins::Cal(p1) => write!(f, "CAL_{}", p1),
            Ins::Ret => write!(f, "RET"),
            Ins::Xor(p1, p2) => write!(f, "XOR_{}_{}", p1, p2),
            Ins::End => write!(f, "END"),
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
            return Param::Ptr(Box::from(Param::build_with_value(val.get(1..).unwrap())));
        }

        // if val has only one char, it's a base10 literal or flag. for sure
        if val.len() < 2 {
            return match u16::from_str_radix(&val, 10) {
                Ok(v) => Param::Lit(v),
                Err(_) => Param::Flag(val),
            };
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
