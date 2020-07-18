use std::io;

mod nalu;
use self::nalu::NaluDemuxer;

use super::Data;

pub trait Demuxer {
    fn read(&mut self) -> io::Result<Data>;
}

pub fn new(filename: &str) -> Box<dyn Demuxer> {
    NaluDemuxer::new(filename)
}
