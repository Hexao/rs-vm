use std::fmt::{Display, Formatter};
use std::collections::HashMap;
use regex::Regex;

use crate::chunk::Chunk;

pub struct DataParser {
    order: Vec<String>,
    vars: HashMap<String, (Vec<u8>, u16)>,
}

macro_rules! unpack_capture {
    ($capture:expr) => {{
        let el = $capture;
        match el.get(1) {
            Some(data) => Ok(data.as_str()),
            None => match el.get(2) {
                Some(data) => Ok(data.as_str()),
                None => Err("Error while parsing data...".to_owned()),
            },
        }
    }};
}

impl DataParser {
    pub fn new(chunk: Chunk) -> Result<Self, String> {
        let reg = Regex::new(r#""(.+?)"\s*,?|([^\s,]+)\s*,?"#).unwrap();
        let mut vars = HashMap::new();
        let mut order = vec![];
        let mut location = 0;

        for (_id, line) in chunk.data() {
            let mut vec = vec![];
            let mut captures = reg.captures_iter(&line);
            let name = unpack_capture!(captures.next().unwrap())?;
            order.push(name.to_owned());

            loop {
                let next = match captures.next() {
                    Some(next) => next,
                    None => break,
                };

                let mut el = match next.get(1) {
                    Some(data) => {
                        let data = data.as_str().encode_utf16().collect::<Vec<u16>>();
                        let mut vec = Vec::with_capacity(data.len() * 2);

                        for el in data {
                            vec.push((el >> 8) as u8);
                            vec.push((el & 0xFF) as u8);
                        }

                        Ok(vec)
                    },
                    None => match next.get(2) {
                        Some(data) => {
                            let data = data.as_str();

                            if data.len() <= 2 {
                                let v = match u16::from_str_radix(data, 10) {
                                    Ok(v) => v,
                                    Err(e) => return Err(format!("Parse error on {}: {}", data, e)),
                                };

                                Ok(vec![(v >> 8) as u8, (v & 0xFF) as u8])
                            } else {
                                let v1 = data.get(0..2).unwrap();

                                let v = match v1 {
                                    "0x" => u16::from_str_radix(data.get(2..).unwrap(), 16).unwrap(),
                                    "0b" => u16::from_str_radix(data.get(2..).unwrap(), 2).unwrap(),
                                    _ => match u16::from_str_radix(data, 10) {
                                        Ok(v) => v,
                                        Err(e) => return Err(format!("Parse error on {}: {}", data, e)),
                                    },
                                };

                                Ok(vec![(v >> 8) as u8, (v & 0xFF) as u8])
                            }
                        }
                        None => Err("Error while parsing data...".to_owned()),
                    },
                }?;

                vec.append(&mut el);
            }

            let vlen = match vec.len() {
                0 => {
                    vec.append(&mut vec![0, 0]);
                    2
                }
                vlen => vlen as u16,
            };

            vars.insert(name.to_owned(), (vec, location));
            location += vlen;
        }

        Ok(Self { order, vars })
    }

    pub fn data_len(&self) -> usize {
        let (v, m) = self.vars.get(
            self.order.last().unwrap()
        ).unwrap();

        *m as usize + v.len()
    }

    pub fn vars(&self) -> &HashMap<String, (Vec<u8>, u16)> {
        &self.vars
    }

    pub fn to_vec(mut self) -> Vec<u8> {
        let data_len = self.data_len();
        let mut vec = Vec::with_capacity(data_len);

        for key in self.order {
            let (data, _) = self.vars.get_mut(&key).unwrap();
            vec.append(data);
        }

        vec
    }
}

impl Display for DataParser {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, ".data")?;

        for var in &self.order {
            let (data, loc) = self.vars.get(var).unwrap();
            writeln!(f, "{} @mem:{} => {:?}", var, loc, data)?;
        }

        Ok(())
    }
}
