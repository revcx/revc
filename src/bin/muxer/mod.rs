use std::io;

mod yuv;
use self::yuv::YuvMuxer;

use revc::api::frame::Frame;

pub trait Muxer {
    fn write(&mut self, f: &Frame<u16>, bitdepth: u8) -> io::Result<()>;
}

pub fn new(filename: &str) -> Box<dyn Muxer> {
    YuvMuxer::new(filename)
}
