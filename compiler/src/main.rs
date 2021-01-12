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
    let mut jumps_pts = HashMap::new();
    let mut cmds = Vec::with_capacity(10);
    let mut ptr = 0;

    for line in file {
        if let Some(cmd) = Ins::build_with_line(line.unwrap()) {
            if let Ins::Flag(flag) = &cmd {
                jumps_pts.insert(flag.to_owned(), ptr as u16);
                continue;
            }

            ptr += cmd.len();
            cmds.push(cmd);
        }
    }

    let mut res = Vec::with_capacity(ptr);
    for cmd in cmds {
        match cmd.get_code(&jumps_pts) {
            Ok(mut byts) => res.append(&mut byts),
            Err(s) => {
                eprintln!("Error while compiling : {}", s);
                return;
            }
        }
    }

    let out_file = args.out.unwrap_or(args.input);
    std::fs::create_dir_all(out_dir).expect(&format!("can't create dir '{}'", out_dir));
    let mut out_file = File::create(format!("{}{}.vmo", out_dir, out_file)).unwrap();
    out_file.write_all(&res).unwrap();
}
