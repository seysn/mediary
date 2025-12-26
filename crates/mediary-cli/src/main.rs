use std::{
    ffi::OsStr,
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
    time::Instant,
};

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
    Pnm(PathBuf),
}

#[derive(Debug, Clone)]
enum ToImage {
    Pnm(PathBuf),
}

impl FromStr for FromImage {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(FromImage::from_extension(s))
    }
}

impl FromStr for ToImage {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ToImage::from_extension(s))
    }
}

impl FromImage {
    fn from_extension<P: Into<PathBuf>>(path: P) -> Self {
        let path = path.into();
        match path.extension().and_then(OsStr::to_str) {
            Some("jpeg") | Some("jpg") => Self::Jpeg(path),
            Some("pnm") | Some("ppm") => Self::Pnm(path),
            ext => unimplemented!("Cannot decode extension {ext:?}"),
        }
    }
}

impl ToImage {
    fn from_extension<P: Into<PathBuf>>(path: P) -> Self {
        let path = path.into();
        match path.extension().and_then(OsStr::to_str) {
            Some("pbm") | Some("pgm") | Some("ppm") | Some("pnm") => Self::Pnm(path),
            ext => unimplemented!("Cannot decode extension {ext:?}"),
        }
    }

    fn path(&self) -> &Path {
        match self {
            ToImage::Pnm(path_buf) => path_buf.as_path(),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let instant = Instant::now();
    let image = match args.from {
        FromImage::Jpeg(path) => {
            let jpeg = RawJpeg::read(path)?;
            jpeg.decode()?
        }
        FromImage::Pnm(path) => {
            let pnm = PnmImage::read(path)?;
            pnm.into_rgb()
        }
    };
    println!("Decoding took {:?}", instant.elapsed());

    let instant = Instant::now();
    match &args.to {
        ToImage::Pnm(path) => {
            PnmImage::new(image).write(path)?;
        }
    }
    println!("Encoding took {:?}", instant.elapsed());
    println!("Saved file {:?}", args.to.path());

    Ok(())
}
