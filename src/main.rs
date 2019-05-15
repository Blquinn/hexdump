extern crate structopt;

mod hex;

use std::io;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;
use hex::Writers;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: Option<PathBuf>,
    
    #[structopt(short = "b", long = "buffer")]
    buffer: bool,
}

fn main() {
    let opt = Opt::from_args();

    let mut file_or_stdout = match opt.output {
        None => if opt.buffer {
            Writers::StdoutBuf(io::BufWriter::new(io::stdout()))
        } else {
            Writers::Stdout(io::stdout())
        },
        Some(pb) => {
            match fs::File::create(&pb) {
                Ok(f) => if opt.buffer {
                    Writers::FileBuf(io::BufWriter::new(f))
                } else {
                    Writers::File(f)
                },
                Err(err) => {
                    eprintln!("Error opening output file {:?} {}", pb, &err);
                    std::process::exit(10);
                }
            }
        }
    };

    let mut dumper = hex::HexDumper::new(&mut file_or_stdout);

    if let Err(err) = io::copy(&mut io::stdin(), &mut dumper) {
        eprintln!("Got err while dumping {}.", err);
    }
    if let Err(err) = dumper.close() {
        eprintln!("Got err while flushing dumper {}.", err);
    }
}
