use structopt::StructOpt;
use structs::Ins;
use std::{collections::HashMap, fs::File};
use std::io::{self, BufRead};
mod structs;

#[derive(StructOpt)]
pub struct Args {
    pub input: String,

    pub out: Option<String>,
}

fn main() {
    // let args: Args = Args::from_args();
    let input_dir = "data/scripts/";
    let out_dir = "data/output/";
    let file_name = "mv_reg";

    let file = File::open(format!("{}{}.vms", input_dir, file_name)).unwrap();
    let file = io::BufReader::new(file).lines();
    let mut jumps_pts = HashMap::new();
    let mut cmds = Vec::with_capacity(10);
    let mut ptr = 0;

    for line in file {
        if let Some(cmd) = Ins::build_with_line(line.unwrap()) {
            // println!("{:?}", cmd);

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

    println!("{:?}", res);
}
