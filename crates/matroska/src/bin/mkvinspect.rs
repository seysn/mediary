use std::{fs::File, path::PathBuf};

use clap::Parser;
use ebml::element::{EbmlElement, EbmlElementValue, MasterElement};
use matroska::{element::MkvElement, Matroska, MatroskaReader};

#[derive(Debug, Parser)]
#[clap(version)]
struct Args {
    #[arg(short, long, action)]
    parse: bool,

    path: PathBuf,
}

fn main() {
    let args = Args::parse();

    let file = File::open(&args.path).unwrap();
    if args.parse {
        inspect_parsed(file);
    } else {
        inspect(file);
    }
}

fn inspect_parsed(file: File) {
    let mkv = Matroska::read(file).unwrap();
    println!("{mkv:#?}");
}

fn inspect(file: File) {
    let mkv = MatroskaReader::read(file).unwrap();

    let header = &mkv.ebml_header;
    println!("Ebml (Master)");
    println!("  EbmlVersion = UnsignedInteger({})", header.ebml_version);
    println!(
        "  EbmlReadVersion = UnsignedInteger({})",
        header.ebml_read_version
    );
    println!(
        "  EbmlMaxIDLength = UnsignedInteger({})",
        header.max_id_length
    );
    println!(
        "  EbmlMaxSizeLength = UnsignedInteger({})",
        header.max_size_length
    );
    println!("  DocType = String(\"{}\")", header.doc_type);
    println!(
        "  DocTypeVersion = UnsignedInteger({})",
        header.doc_type_version
    );
    println!(
        "  DocTypeReadVersion = UnsignedInteger({})",
        header.doc_type_read_version
    );

    for elem in mkv {
        read_element(elem.unwrap(), 0);
    }
}

fn read_element(element: EbmlElement<MkvElement, File>, depth: usize) {
    match element {
        EbmlElement::Master(element) => read_master(element, depth),
        EbmlElement::Value(element) => {
            let value = element.value().unwrap();
            let value = if let EbmlElementValue::Binary(bin) = value {
                format!("Binary({bin:02x?})")
            } else {
                format!("{value:?}")
            };

            println!("{}{} = {value}", " ".repeat(depth * 2), element.name());
        }
        EbmlElement::LazyValue(element) => {
            println!(
                "{}{} = {:?}([ {} bytes ]) ",
                " ".repeat(depth * 2),
                element.name(),
                element.kind(),
                element.size,
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
