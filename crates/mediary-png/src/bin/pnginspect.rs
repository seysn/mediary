use std::{env, fs::File, io::BufReader, process};

use mediary_png::reader::PngReader;

fn main() {
    let mut args = env::args();
    let exe = args.next().expect("should have executable name");
    let Some(path) = args.next() else {
        eprintln!("usage: {exe} <path>");
        process::exit(1);
    };

    let Ok(file) = File::open(&path) else {
        eprintln!("{path} not found");
        process::exit(1);
    };

    let reader = PngReader::new(BufReader::new(file));
    if let Err(err) = reader.read() {
        eprintln!("Error while reading: {err}");
    }
}
