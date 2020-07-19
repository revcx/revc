mod nalu;
mod y4m;
mod yuv;

use std::io;
use std::path::Path;

use self::y4m::Y4mMuxer;
use self::yuv::YuvMuxer;
use crate::io::muxer::nalu::NaluMuxer;
use revc::api::*;

pub trait Muxer {
    fn write(&mut self, data: Data, bitdepth: u8, frame_rate: Rational) -> io::Result<()>;
}

pub fn new(filename: &str) -> io::Result<Box<dyn Muxer>> {
    if let Some(ext) = Path::new(filename).extension() {
        if ext == "y4m" {
            Ok(Y4mMuxer::new(filename))
        } else if ext == "yuv" {
            Ok(YuvMuxer::new(filename))
        } else {
            // .evc
            Ok(NaluMuxer::new(filename))
        }
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Filename doesn't have extension",
        ))
    }
}
