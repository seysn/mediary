use std::fs::File;

use ebml::element::{EbmlElement, MasterElement};
use matroska::{element::MkvElement, Matroska};

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let file = File::open(&path).unwrap();
    let mkv = Matroska::read(file).unwrap();
    dbg!(&mkv.ebml_document.header);

    for elem in mkv.ebml_document.iter() {
        read_element(elem, 0);
    }
}

fn read_element(element: EbmlElement<MkvElement, File>, depth: usize) {
    match element {
        EbmlElement::Master(element) => read_master(element, depth),
        EbmlElement::Value(element) => {
            println!(
                "{}{} = {:?}",
                " ".repeat(depth * 2),
                element.name(),
                element.value(),
            );
        }
        EbmlElement::LazyValue(element) => {
            println!(
                "{}{} ({:?})",
                " ".repeat(depth * 2),
                element.name(),
                element.kind(),
            );
        }
    }
}

fn read_master(element: MasterElement<MkvElement, File>, depth: usize) {
    println!(
        "{}{} ({:?})",
        " ".repeat(depth * 2),
        element.name(),
        element.kind()
    );

    for child in element.children() {
        read_element(child, depth + 1)
    }
}
