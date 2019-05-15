extern crate structopt;

mod hex;

use std::io;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;
use hex::FileOrStdout;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: Option<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();

    let mut file_or_stdout = match opt.output {
        None => FileOrStdout::Stdout(io::BufWriter::new(io::stdout())),
        Some(pb) => {
            match fs::File::create(&pb) {
                Ok(f) => FileOrStdout::File(io::BufWriter::new(f)),
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
