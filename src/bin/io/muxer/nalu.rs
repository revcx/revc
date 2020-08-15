use super::Muxer;
use crate::Data;

use std::cmp::*;
use std::fs::File;
use std::io;
use std::io::Write;
use std::slice;

use revc::api::frame::*;
use revc::api::Rational;

pub struct NaluMuxer {
    writer: Box<dyn Write>,
}

impl NaluMuxer {
    pub fn new(path: &str) -> Box<dyn Muxer> {
        Box::new(NaluMuxer {
            writer: match path {
                "-" => Box::new(io::stdout()),
                f => Box::new(File::create(&f).unwrap()),
            },
        })
    }
}

impl Muxer for NaluMuxer {
    fn write(&mut self, data: Data, bitdepth: u8, frame_rate: Rational) -> io::Result<()> {
        if let Data::RefPacket(pkt_data) = data {
            let pkt = pkt_data.borrow();
            self.writer.write(&pkt.data)?;

            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Packet Data for NaluMuxer",
            ))
        }
    }
}
