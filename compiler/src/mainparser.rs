use std::fmt::{Display, Formatter};
use std::collections::HashMap;

use crate::dataparser::DataParser;
use crate::instructions::Ins;
use crate::chunk::Chunk;

pub struct MainParser {
    start_address: usize,
    cmds: Vec<(Ins, usize)>,
    jumps_pts: HashMap<String, u16>,
}

impl MainParser {
    pub fn new(chunk: Chunk) -> Result<Self, String> {
        let mut cmds = Vec::with_capacity(10);
        let mut start_address = None;

        for (id, line) in chunk.data() {
            if let Some(cmd) = Ins::build_with_line(line) {
                if let Ins::Flag(flag) = &cmd {
                    if start_address == None && flag == "start" {
                        start_address = Some(cmds.len());
                    }
                }
                cmds.push((cmd, id));
            }
        }

        let start_address = match start_address {
            Some(add) => add,
            None => return Err(format!("Flag start is required to start execution")),
        };

        let mut ptr = 0;
        let cmds_len = cmds.len();
        let mut jumps_pts = HashMap::new();

        // parse Ins and find address for all flags
        for id in 0..cmds_len {
            let id = (start_address + id) % cmds_len;
            let (cmd, line) = &cmds[id];

            if let Ins::Flag(flag) = cmd {
                match jumps_pts.insert(flag.to_owned(), ptr as u16) {
                    Some(_) => return Err(format!("Duplicate flag {} on line {}", flag, line)),
                    None => (),
                }
            }

            ptr += cmd.len();
        }

        Ok(Self{ start_address, cmds, jumps_pts })
    }

    pub fn ins_len(&self) -> usize {
        let mut len = 0;

        for (ins, _) in &self.cmds {
            len += ins.len();
        }

        len
    }

    pub fn to_vec(self, data: Option<DataParser>) -> Result<Vec<u8>, String> {
        let ins_len = self.ins_len();
        let (data_len, vars) = match &data {
            Some(data) => (data.data_len(), Some(data.vars())),
            None => (0, None),
        };

        let mut vec = Vec::with_capacity(ins_len + data_len);
        let cmds_len = self.cmds.len();

        for id in 0..cmds_len {
            let id = (self.start_address + id) % cmds_len;
            let (ins, line) = &self.cmds[id];

            match ins.get_code(&self.jumps_pts, vars, ins_len as u16) {
                Ok(v) => for el in v {
                    vec.push(el);
                },
                Err(s) => return Err(format!("Error while compiling on line {} : {}", line, s)),
            }
        }

        if let Some(data) = data {
            vec.append(&mut data.to_vec());
        }

        Ok(vec)
    }
}

impl Display for MainParser {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, ".main")?;

        for (ins, line) in &self.cmds {
            writeln!(f, "{:>3} : {:?}", line, ins)?;
        }

        Ok(())
    }
}
