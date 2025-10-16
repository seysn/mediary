use std::{ffi::OsStr, fs::File, path::PathBuf, str::FromStr};

use clap::Parser;
use mediary_jpeg::RawJpeg;
use mediary_pnm::PnmImage;

#[derive(Debug, Parser)]
#[clap(version)]
struct Args {
    from: FromImage,
    to: ToImage,
}

#[derive(Debug, Clone)]
enum FromImage {
    Jpeg(PathBuf),
}

#[derive(Debug, Clone)]
enum ToImage {
    Pnm(PathBuf),
}

impl FromStr for FromImage {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path: PathBuf = s.parse()?;

        match path.extension().and_then(OsStr::to_str) {
            Some("jpeg") | Some("jpg") => Ok(Self::Jpeg(path)),
            ext => unimplemented!("Cannot decode extension {ext:?}"),
        }
    }
}

impl FromStr for ToImage {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path: PathBuf = s.parse()?;

        match path.extension().and_then(OsStr::to_str) {
            Some("pbm") | Some("pgm") | Some("ppm") => Ok(Self::Pnm(path)),
            ext => unimplemented!("Cannot decode extension {ext:?}"),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let image = match args.from {
        FromImage::Jpeg(path) => {
            let jpeg = RawJpeg::read(path)?;
            jpeg.decode()?
        }
    };

    match args.to {
        ToImage::Pnm(path) => {
            let output = File::create(path)?;
            PnmImage::new(image).write(output)?;
        }
    }

    Ok(())
}
