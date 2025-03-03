use std::fs::File;

use ebml::EbmlDocument;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let _ = EbmlDocument::from_reader(&mut File::open(&path).unwrap()).unwrap();
}
