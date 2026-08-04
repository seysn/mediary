use std::path::PathBuf;

use clap::Parser;
use mediary_jpeg::RawJpeg;

#[derive(Debug, Parser)]
#[clap(version)]
struct Args {
    path: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let jpeg = RawJpeg::read(&args.path)?;
    println!("{jpeg:#?}");

    Ok(())
}
