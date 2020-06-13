use super::Demuxer;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

use std::fs::File;
use std::io;
use std::io::Read;

use revc::api::*;

pub struct NaluDemuxer {
    reader: Box<dyn Read>,
}

impl NaluDemuxer {
    pub fn new(path: &str) -> Box<dyn Demuxer> {
        Box::new(NaluDemuxer {
            reader: match path {
                "-" => Box::new(io::stdin()),
                f => Box::new(File::open(&f).unwrap()),
            },
        })
    }
}

impl Demuxer for NaluDemuxer {
    fn read(&mut self) -> io::Result<Packet> {
        //TODO: To be confirmed with ETM, is it ok from endianness perspective?
        let nal_unit_length = self.reader.read_u32::<LittleEndian>()?;
        let mut data: Vec<u8> = vec![0; nal_unit_length as usize];
        self.reader.read_exact(&mut data)?;

        Ok(Packet {
            data,
            offset: 0,
            pts: 0,
        })
    }
}
