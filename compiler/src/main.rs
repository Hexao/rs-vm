use std::io::{self, BufRead, prelude::*};
use std::{collections::HashMap, fs::File};
use structopt::StructOpt;
use structs::Ins;
mod structs;

#[derive(StructOpt)]
pub struct Args {
    pub input: String,

    pub out: Option<String>,
}

fn main() {
    let args: Args = Args::from_args();
    let input_dir = "data/scripts/";
    let out_dir = "data/output/";

    let file = File::open(format!("{}{}.vms", input_dir, args.input)).unwrap();
    let file = io::BufReader::new(file).lines();
    let mut cmds = Vec::with_capacity(10);
    let mut stard_address = 0;

    for (id, line) in file.enumerate() {
        if let Some(cmd) = Ins::build_with_line(line.unwrap()) {
            if let Ins::Flag(flag) = &cmd {
                if stard_address == 0 && flag == "start" {
                    stard_address = cmds.len();
                }
            }
            cmds.push((cmd, id + 1));
        }
    }

    let mut ptr = 0;
    let cmds_len = cmds.len();
    let mut jumps_pts = HashMap::new();

    // parse Ins and find address for all flags
    for id in 0..cmds_len {
        let id = (stard_address + id) % cmds_len;
        let (cmd, line) = &cmds[id];

        if let Ins::Flag(flag) = cmd {
            match jumps_pts.insert(flag.to_owned(), ptr as u16) {
                None => (),
                Some(_) => {
                    eprintln!("Duplicate flag {} on line {}", flag, line);
                    return;
                }
            }
        }

        ptr += cmd.len();
    }

    let mut res = Vec::with_capacity(ptr);

    // iteratr trough Ins and produce exec code
    for id in 0..cmds_len {
        let id = (stard_address + id) % cmds_len;
        let (cmd, line) = &cmds[id];

        match cmd.get_code(&jumps_pts) {
            Ok(mut byts) => res.append(&mut byts),
            Err(s) => {
                eprintln!("Error while compiling on line {} : {}", line, s);
                println!("{:?}", res);
                return;
            }
        }
    }

    let out_file = args.out.unwrap_or(args.input);
    std::fs::create_dir_all(out_dir).unwrap_or_else(|_|
        panic!("can't create dir '{}'", out_dir)
    );
    let mut out_file = File::create(format!("{}{}.vmo", out_dir, out_file)).unwrap();
    out_file.write_all(&res).unwrap();
}
