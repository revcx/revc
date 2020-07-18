mod nalu;
mod y4m;

use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

use self::nalu::NaluDemuxer;
use self::y4m::Y4mDemuxer;
use super::Data;

pub trait Demuxer {
    fn read(&mut self) -> io::Result<Data>;
}

pub fn new(filename: &str) -> io::Result<Box<dyn Demuxer>> {
    if let Some(ext) = Path::new(filename).extension() {
        if ext == "y4m" {
            Ok(Y4mDemuxer::new(filename)?)
        } else {
            // .evc
            Ok(NaluDemuxer::new(filename)?)
        }
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Filename doesn't have extension",
        ))
    }
}
