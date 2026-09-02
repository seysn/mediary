use std::{env, fs::File, io::BufReader, process};

use mediary_png::{SIGNATURE, chunk::{ImageData, PngChunk}, reader::PngReader, zlib::ZLibHeader};

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

    let mut reader = PngReader::new(BufReader::new(file));
    let signature = match reader.read_signature() {
        Ok(signature) => signature,
        Err(err) => {
            eprintln!("Couldn't read signature: {err}");
            process::exit(1);
        }
    };

    if signature != SIGNATURE {
        eprintln!("Invalid signature, found {signature:02x?}, expected {SIGNATURE:02x?}");
        process::exit(1);
    }

    let mut found_idat = false;
    loop {
        let chunk = match reader.read_chunk() {
            Ok(chunk) => chunk,
            Err(err) => {
                eprintln!("Couldn't read chunk: {err}");
                process::exit(1);
            }
        };

        println!("{chunk:?}");

        if let PngChunk::ImageData(ImageData(data)) = &chunk && !found_idat {
            println!("> {:?}", ZLibHeader::new(data));

            found_idat = true;
        }

        if let PngChunk::ImageTrailer(_) = chunk {
            break;
        }
    }
}
