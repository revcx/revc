use super::Data;
use super::Demuxer;

use std::fs::File;
use std::io;
use std::io::Read;

use revc::api::*;

pub struct NaluDemuxer {
    reader: Box<dyn Read>,
}

impl NaluDemuxer {
    pub fn new(path: &str) -> io::Result<Box<dyn Demuxer>> {
        Ok(Box::new(NaluDemuxer {
            reader: match path {
                "-" => Box::new(io::stdin()),
                f => Box::new(File::open(&f).unwrap()),
            },
        }))
    }
}

impl Demuxer for NaluDemuxer {
    fn read(&mut self) -> io::Result<Data> {
        let mut buf = [0u8; 4];
        self.reader.read_exact(&mut buf)?;
        let nal_unit_length =
            (buf[3] as u32) << 24 | (buf[2] as u32) << 16 | (buf[1] as u32) << 8 | buf[0] as u32;

        let mut data: Vec<u8> = vec![0; nal_unit_length as usize];
        self.reader.read_exact(&mut data)?;

        Ok(Data::Packet(Packet {
            data: Some(data),
            pts: 0,
        }))
    }
}
