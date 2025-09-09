use std::{fs::File, io::Write, path::PathBuf};

use clap::Parser;
use mediary_jpeg::RawJpeg;

#[derive(Debug, Parser)]
#[clap(version)]
struct Args {
    path: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let jpeg = RawJpeg::read(&args.path)?;
    let res = jpeg.decode();

    let basename = args
        .path
        .file_stem()
        .expect("path should have a file name")
        .to_string_lossy();
    let width = res.width;
    let height = res.height;
    let output_name = format!("{basename}-{width}x{height}.rgb");
    let mut output = File::create(&output_name)?;
    output.write_all(&res.data)?;

    println!("Wrote {} bytes in {output_name}", res.data.len());
    Ok(())
}
