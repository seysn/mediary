use std::fs::File;

use ebml::element::{EbmlElement, MasterElement};
use matroska::{element::MkvElement, MatroskaReader};

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let file = File::open(&path).unwrap();
    let mkv = MatroskaReader::read(file).unwrap();

    let header = &mkv.ebml_header;
    println!("Header:");
    println!("- ebml_version = {}", header.ebml_version);
    println!("- ebml_read_version = {}", header.ebml_read_version);
    println!("- max_id_length = {}", header.max_id_length);
    println!("- max_size_length = {}", header.max_size_length);
    println!("- doc_type = {}", header.doc_type);
    println!("- doc_type_version = {}", header.doc_type_version);
    println!("- doc_type_read_version = {}", header.doc_type_read_version);
    println!();

    for elem in mkv {
        read_element(elem.unwrap(), 0);
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
        read_element(child.unwrap(), depth + 1)
    }
}
