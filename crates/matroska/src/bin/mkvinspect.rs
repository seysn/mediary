use std::fs::File;

use matroska::Matroska;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let file = File::open(&path).unwrap();
    let mkv = Matroska::read(file).unwrap();
    dbg!(&mkv.ebml_document.header);
}
