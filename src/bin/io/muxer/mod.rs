mod y4m;
mod yuv;

use std::io;
use std::path::Path;

use self::yuv::YuvMuxer;
use super::Data;
use revc::api::Rational;

pub trait Muxer {
    fn write(&mut self, data: Data, bitdepth: u8, frame_rate: Rational) -> io::Result<()>;
}

pub fn new(filename: &str) -> io::Result<Box<dyn Muxer>> {
    if let Some(ext) = Path::new(filename).extension() {
        Ok(YuvMuxer::new(filename))
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Filename doesn't have extension",
        ))
    }
}
