use std::{fs::File, path::PathBuf};

use clap::Parser;
use ebml::element::{EbmlElement, MasterElement};
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
    println!("<Ebml>");
    println!("  <EbmlVersion>{}</EbmlVersion>", header.ebml_version);
    println!(
        "  <EbmlReadVersion>{}</EbmlReadVersion>",
        header.ebml_read_version
    );
    println!(
        "  <EbmlMaxIDLength>{}</EbmlMaxIDLength>",
        header.max_id_length
    );
    println!(
        "  <EbmlMaxSizeLength>{}</EbmlMaxSizeLength>",
        header.max_size_length
    );
    println!("  <DocType>{}</DocType>", header.doc_type);
    println!(
        "  <DocTypeVersion>{}</DocTypeVersion>",
        header.doc_type_version
    );
    println!(
        "  <DocTypeReadVersion>{}</DocTypeReadVersion>",
        header.doc_type_read_version
    );
    println!("</Ebml>");

    for elem in mkv {
        read_element(elem.unwrap(), 0);
    }
}

fn read_element(element: EbmlElement<MkvElement, File>, depth: usize) {
    match element {
        EbmlElement::Master(element) => read_master(element, depth),
        EbmlElement::Value(element) => {
            let name = element.name();
            let value = element.value().unwrap();
            println!("{}<{name}>{value}</{name}>", " ".repeat(depth * 2),);
        }
        EbmlElement::LazyValue(element) => {
            let name = element.name();
            println!(
                "{}<{name}> [ {} bytes ] </{name}>",
                " ".repeat(depth * 2),
                element.size,
            );
        }
    }
}

fn read_master(element: MasterElement<MkvElement, File>, depth: usize) {
    println!("{}<{}>", " ".repeat(depth * 2), element.name(),);

    for child in element.children() {
        read_element(child.unwrap(), depth + 1)
    }

    println!("{}</{}>", " ".repeat(depth * 2), element.name(),);
}
