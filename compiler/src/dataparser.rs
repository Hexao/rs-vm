use std::fmt::{Display, Formatter};
use std::collections::HashMap;
use regex::Regex;

use crate::{chunk::Chunk, variable::Type};
use crate::variable::Var;

enum Expect {
    Name,
    Type,
    Value,
    Separator,
}

impl Expect {
    pub fn match_with(&self, value: usize) -> bool {
        match &self {
            Expect::Name => value == DataParser::VAR_NAME,
            Expect::Type => value == DataParser::TYPE,
            Expect::Value => matches!(value, DataParser::BINAIRY | DataParser::HEXA | DataParser::DECIMAL | DataParser::STRING),
            Expect::Separator => value == DataParser::SEPARATOR,
        }
    }
}

impl Display for Expect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Expect::Name => write!(f, "VAR_NAME"),
            Expect::Type => write!(f, "VAR_TYPE"),
            Expect::Value => write!(f, "VALUE"),
            Expect::Separator => write!(f, "SEPARATOR"),
        }
    }
}

pub struct DataParser {
    order: Vec<String>,
    vars: HashMap<String, Var>,
}

impl DataParser {
    const SEPARATOR: usize = 1;
    const BINAIRY  : usize = 2;
    const HEXA     : usize = 3;
    const DECIMAL  : usize = 4;
    const STRING   : usize = 5;
    const TYPE     : usize = 6;
    const VAR_NAME : usize = 7;

    pub fn new(chunk: Chunk) -> Result<Self, String> {
        let reg = Regex::new(r#"\s*(?:(,)|(?:0[bB]([01]+))|(?:0[xX])([0-9a-fA-F]+)|(\d+)|(?:")(.+?)(?:")|([uU](?:8|16))|([a-zA-z]\w*))"#).unwrap();
        let mut vars = HashMap::new();
        let mut order = vec![];
        let mut location = 0;

        for (id, line) in chunk.data() {
            let mut name = "";
            let mut var = Var::default();
            let mut expect = Expect::Name;

            for next in reg.captures_iter(&line) {
                if next.get(DataParser::SEPARATOR).is_some() {
                    if expect.match_with(DataParser::SEPARATOR) {
                        expect = Expect::Value;
                    } else {
                        return Err(format!("On line {}, expected {}, found SEPARATOR", id, expect));
                    }
                } else if let Some(bin) = next.get(DataParser::BINAIRY) {
                    if expect.match_with(DataParser::BINAIRY) {
                        expect = Expect::Separator;
                        let bin = bin.as_str();
                        let val = u16::from_str_radix(bin, 2).unwrap();

                        match var.type_len() {
                            1 => var.add_data(&mut vec![(val & 0xFF) as u8]),
                            2 => var.add_data(&mut vec![(val >> 8) as u8, (val & 0xFF) as u8]),
                            _ => (),
                        }
                    } else {
                        return Err(format!("On line {}, expected {}, found VALUE", id, expect));
                    }
                } else if let Some(hex) = next.get(DataParser::HEXA) {
                    if expect.match_with(DataParser::HEXA) {
                        expect = Expect::Separator;
                        let hex = hex.as_str();
                        let val = u16::from_str_radix(hex, 16).unwrap();

                        match var.type_len() {
                            1 => var.add_data(&mut vec![(val & 0xFF) as u8]),
                            2 => var.add_data(&mut vec![(val >> 8) as u8, (val & 0xFF) as u8]),
                            _ => (),
                        }
                    } else {
                        return Err(format!("On line {}, expected {}, found VALUE", id, expect));
                    }
                } else if let Some(dec) = next.get(DataParser::DECIMAL) {
                    if expect.match_with(DataParser::DECIMAL) {
                        expect = Expect::Separator;
                        let dec = dec.as_str();
                        let val = dec.parse::<u16>().unwrap();

                        match var.type_len() {
                            1 => var.add_data(&mut vec![(val & 0xFF) as u8]),
                            2 => var.add_data(&mut vec![(val >> 8) as u8, (val & 0xFF) as u8]),
                            _ => (),
                        }
                    } else {
                        return Err(format!("On line {}, expected {}, found VALUE", id, expect));
                    }
                } else if let Some(string) = next.get(DataParser::STRING) {
                    if expect.match_with(DataParser::STRING) {
                        expect = Expect::Separator;
                        let string = string.as_str();
                        let vec = string.encode_utf16().collect::<Vec<u16>>();

                        match var.type_len() {
                            1 => for el in vec {
                                var.add_data(&mut vec![(el & 0xFF) as u8]);
                            }
                            2 => for el in vec {
                                var.add_data(&mut vec![(el >> 8) as u8, (el & 0xFF) as u8]);
                            }
                            _ => (),
                        }
                    } else {
                        return Err(format!("On line {}, expected {}, found VALUE", id, expect));
                    }
                } else if let Some(data_type) = next.get(DataParser::TYPE) {
                    if expect.match_with(DataParser::TYPE) {
                        let data_type = data_type.as_str();

                        match Type::from(data_type) {
                            Type::None => return Err(format!("On line {}, found an unknown type: {}", id, data_type)),
                            data_type => {
                                var.set_type(data_type);
                                expect = Expect::Value;
                            }
                        }
                    } else {
                        return Err(format!("On line {}, expected {}, found VAR_TYPE", id, expect));
                    }
                } else if let Some(var_name) = next.get(DataParser::VAR_NAME) {
                    if expect.match_with(DataParser::VAR_NAME) {
                        name = var_name.as_str();
                        expect = Expect::Type;
                    } else if expect.match_with(DataParser::TYPE) {
                        return Err(format!("On line {}, found an unknown type: {}", id, var_name.as_str()));
                    } else {
                        return Err(format!("On line {}, expected {}, found VAR_NAME", id, expect));
                    }
                }
            }

            match expect {
                Expect::Separator => (),
                _ => return Err(format!("On line {}, expected {}, found nothing", id, expect)),
            }

            let vlen = var.data_len();
            var.set_location(location);
            location += vlen as u16;

            order.push(name.to_owned());
            vars.insert(name.to_owned(), var);
        }

        Ok(Self { order, vars })
    }

    pub fn data_len(&self) -> usize {
        match self.order.last() {
            Some(key) => {
                let v = self.vars.get(key).unwrap();
                *v.get_location() as usize + v.data_len()
            }
            None => 0,
        }
    }

    pub fn vars(&self) -> &HashMap<String, Var> {
        &self.vars
    }

    pub fn get_vec(mut self) -> Vec<u8> {
        let data_len = self.data_len();
        let mut vec = Vec::with_capacity(data_len);

        for key in self.order {
            let var = self.vars.get_mut(&key).unwrap();
            vec.append(var.get_data_mut());
        }

        vec
    }
}

impl Display for DataParser {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, ".data")?;

        for var in &self.order {
            let data = self.vars.get(var).unwrap();
            writeln!(f, "{} @mem:{} => {:?}", var, data.get_location(), data.get_data())?;
        }

        Ok(())
    }
}
