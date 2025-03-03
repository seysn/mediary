use std::io::{Read, Seek};

mod element;
mod error;
mod vint;

pub struct EbmlDocument;

impl EbmlDocument {
    pub fn from_reader<R: Read + Seek>(reader: &mut R) -> error::EbmlResult<Self> {
        while let Ok(element) = element::EbmlElement::from_reader(reader) {
            let size = vint::Vint::from_reader(reader)?;
            println!("{element:?}: size={} type={:?}", size.value, element.kind());

            if matches!(element.kind(), element::ElementType::Master) {
                read_master(reader, size.value, 1)?;
            }
        }

        Ok(Self)
    }
}

fn read_master<R: Read + Seek>(reader: &mut R, size: u64, depth: usize) -> error::EbmlResult<()> {
    let start = reader.stream_position()?;
    while reader.stream_position()? < start + size {
        let element = element::EbmlElement::from_reader(reader)?;
        let size = vint::Vint::from_reader(reader)?;

        if matches!(element.kind(), element::ElementType::Master) {
            println!(
                "{}{element:?}: size={} type=Master",
                "  ".repeat(depth),
                size.value,
            );

            read_master(reader, size.value, depth + 1)?;
        } else {
            let mut buf = vec![0; size.value as usize];
            reader.read_exact(&mut buf)?;

            let value = match element.kind() {
                element::ElementType::String | element::ElementType::Utf8 => {
                    String::from_utf8(buf).unwrap()
                }
                _ => {
                    if buf.len() < 10 {
                        format!("{buf:?}")
                    } else {
                        format!("[size {}]", buf.len())
                    }
                }
            };

            println!(
                "{}{element:?}: size={} type={:?} value={value}",
                "  ".repeat(depth),
                size.value,
                element.kind()
            );
        }
    }

    Ok(())
}
