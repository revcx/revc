use std::io;

mod yuv;
use self::yuv::YuvMuxer;

use super::Data;

pub trait Muxer {
    fn write(&mut self, data: Data, bitdepth: u8) -> io::Result<()>;
}

pub fn new(filename: &str) -> Box<dyn Muxer> {
    YuvMuxer::new(filename)
}
