use std::io::{self, BufRead, Write};
use structopt::StructOpt;
use std::fs::File;
use chunk::Chunk;

use mainparser::MainParser;
use dataparser::DataParser;

pub mod instructions;
pub mod mainparser;
pub mod dataparser;
pub mod chunk;

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
    let mut chunks = vec![];

    for (id, line) in file.enumerate() {
        if let Ok(line) = line {
            let line = line.trim().to_owned();

            if line.starts_with(';') || line.is_empty() {
                continue;
            }

            if line.starts_with('.') {
                let chunk = Chunk::new(line.get(1..).unwrap().to_owned());

                chunks.push(chunk);
            } else if let Some(chunk) = chunks.last_mut() {
                chunk.insert_line(line, id + 1);
            }
        }
    }

    let mut data = None;
    let mut main = None;

    for chunk in chunks {
        match &**chunk.name() { // &** => majik
            "code" => main = match MainParser::new(chunk) {
                Ok(main) => Some(main),
                Err(e) => {
                    eprintln!("{}", e);
                    return;
                }
            },
            "data" => data = match DataParser::new(chunk) {
                Ok(data) => Some(data),
                Err(e) => {
                    eprintln!("{}", e);
                    return;
                }
            },
            seg => {
                eprintln!("Unexpected segment: {}", seg);
                return;
            }
        }
    }

    let res = match main {
        Some(main) => match main.get_vec(data) {
            Ok(ok) => ok,
            Err(s) => {
                eprintln!("{}", s);
                return;
            }
        }
        None => {
            eprintln!("Error on compilation: '.main' segment is required!");
            return;
        }
    };

    let out_file = args.out.unwrap_or(args.input);
    std::fs::create_dir_all(out_dir).unwrap_or_else(|_|
        panic!("can't create dir '{}'", out_dir)
    );
    let mut out_file = File::create(format!("{}{}.vmo", out_dir, out_file)).unwrap();
    out_file.write_all(&res).unwrap();
}
