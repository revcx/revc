use std::io;

mod nalu;
use self::nalu::NaluDemuxer;

use revc::api::Packet;

pub trait Demuxer {
    fn read(&mut self) -> io::Result<Packet>;
}

pub fn new(filename: &str) -> Box<dyn Demuxer> {
    NaluDemuxer::new(filename)
}
