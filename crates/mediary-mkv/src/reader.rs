use std::{
    fmt::Debug,
    io::{ErrorKind, Read, Seek},
    time::Duration,
};

use crate::{
    element::{MkvCluster, MkvElement, MkvInfo, MkvSeekHead, MkvTags, MkvTracks},
    error::{MkvError, MkvResult},
};
use mediary_ebml::{
    element::{EbmlElement, MasterElement},
    error::{EbmlError, EbmlResult},
    EbmlHeader, EbmlReader,
};

pub struct Matroska<R: Read + Seek> {
    reader: MatroskaReader<R>,
    pub seek_head: MkvSeekHead,
    pub info: MkvInfo,
    pub tracks: MkvTracks,
    pub tags: MkvTags,
    pub clusters: Vec<MkvCluster>,
}

pub struct MatroskaReader<R: Read + Seek> {
    ebml_reader: EbmlReader<MkvElement, R>,
    pub ebml_header: EbmlHeader,
}

impl<R: Read + Seek> Matroska<R> {
    pub fn read(reader: R) -> MkvResult<Self> {
        let mut reader = MatroskaReader::read(reader)?;
        let Some(segment) = reader.next() else {
            return Err(MkvError::Ebml(EbmlError::Io(std::io::Error::from(
                ErrorKind::UnexpectedEof,
            ))));
        };

        let segment: MasterElement<MkvElement, R> = segment?.try_into()?;
        if !matches!(segment.element, MkvElement::Segment) {
            return Err(MkvError::Ebml(EbmlError::UnexpectedElement {
                expected: "Segment",
                found: segment.kind().name(),
            }));
        }

        let mut seek_head: Option<MkvSeekHead> = None;
        let mut info: Option<MkvInfo> = None;
        let mut tracks: Option<MkvTracks> = None;
        let mut tags: Option<MkvTags> = None;
        let mut clusters = Vec::new();
        for element in segment.children() {
            let element = element?;

            let EbmlElement::Master(element) = element else {
                continue;
            };

            match element.element {
                MkvElement::SeekHead => seek_head = Some(MkvSeekHead::read(element)?),
                MkvElement::Info => info = Some(MkvInfo::read(element)?),
                MkvElement::Tracks => tracks = Some(MkvTracks::read(element)?),
                MkvElement::Tags => tags = Some(MkvTags::read(element)?),
                MkvElement::Cluster => clusters.push(MkvCluster::read(element)?),
                _ => (),
            }
        }

        Ok(Self {
            reader,
            seek_head: seek_head.unwrap_or_default(),
            info: info.unwrap_or_default(),
            tracks: tracks.unwrap_or_default(),
            tags: tags.unwrap_or_default(),
            clusters,
        })
    }

    pub fn ebml_header(&self) -> &EbmlHeader {
        &self.reader.ebml_header
    }

    pub fn duration(&self) -> Duration {
        self.info.real_duration()
    }

    pub fn framerate(&self) -> f64 {
        let frames: u64 = self.clusters.iter().map(|cluster| cluster.blocks).sum();
        let seconds = self.duration().as_secs_f64();

        frames as f64 / seconds
    }
}

impl<R: Read + Seek> MatroskaReader<R> {
    pub fn read(reader: R) -> MkvResult<Self> {
        let mut ebml_reader = EbmlReader::new(reader)?;
        let ebml_header = ebml_reader.read_ebml_header()?;

        Ok(Self {
            ebml_reader,
            ebml_header,
        })
    }
}

impl<R: Read + Seek> Iterator for MatroskaReader<R> {
    type Item = EbmlResult<EbmlElement<MkvElement, R>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.ebml_reader.next()
    }
}

impl<R: Read + Seek> Debug for Matroska<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Matroska")
            .field("ebml_header", &self.reader.ebml_header)
            .field("seek_head", &self.seek_head)
            .field("info", &self.info)
            .field("tracks", &self.tracks)
            .field("tags", &self.tags)
            .field("clusters", &self.clusters)
            .field("duration", &self.duration())
            .field("framerate", &self.framerate())
            .finish()
    }
}
