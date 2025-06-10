use std::{fs::File, path::PathBuf};

use clap::Parser;
use mediary_bmp::reader::Bmp;

#[derive(Debug, Parser)]
#[clap(version)]
struct Args {
    path: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let file = File::open(&args.path)?;

    let bmp = Bmp::from_reader(file)?;
    println!("{bmp:#?}");

    Ok(())
}
