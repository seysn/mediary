use std::fs::File;

use ebml::{element::EbmlHeaderElement, EbmlDocument};

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let doc = EbmlDocument::<EbmlHeaderElement, File>::new(File::open(&path).unwrap()).unwrap();
    dbg!(doc.header);
}
