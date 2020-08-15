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
        if let Data::Packet(pkt_data) = data {
            if let Some(pkt) = &pkt_data {
                let mut buf = [0u8; 4];
                let nal_unit_length = pkt.data.len();
                buf[0] = (nal_unit_length & 0xFF) as u8;
                buf[1] = ((nal_unit_length >> 8) & 0xFF) as u8;
                buf[2] = ((nal_unit_length >> 16) & 0xFF) as u8;
                buf[3] = ((nal_unit_length >> 24) & 0xFF) as u8;
                self.writer.write(&buf)?;
                self.writer.write(&pkt.data)?;
            }
            Ok(())
        } else if let Data::RefPacket(pkt_data) = data {
            let pkt = pkt_data.borrow();

            let mut buf = [0u8; 4];
            let nal_unit_length = pkt.data.len();
            buf[0] = (nal_unit_length & 0xFF) as u8;
            buf[1] = ((nal_unit_length >> 8) & 0xFF) as u8;
            buf[2] = ((nal_unit_length >> 16) & 0xFF) as u8;
            buf[3] = ((nal_unit_length >> 24) & 0xFF) as u8;
            self.writer.write(&buf)?;
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
