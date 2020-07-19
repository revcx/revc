mod nalu;
mod y4m;

use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

use self::nalu::NaluDemuxer;
use self::y4m::Y4mDemuxer;
use super::Data;
use revc::api::*;

#[derive(Debug, Clone, Copy)]
pub struct VideoInfo {
    pub width: usize,
    pub height: usize,
    pub bit_depth: usize,
    pub chroma_sampling: ChromaSampling,
    pub time_base: Rational,
}

impl Default for VideoInfo {
    fn default() -> Self {
        VideoInfo {
            width: 640,
            height: 480,
            bit_depth: 8,
            chroma_sampling: ChromaSampling::Cs420,
            time_base: Rational { num: 30, den: 1 },
        }
    }
}
pub trait Demuxer {
    fn read(&mut self) -> io::Result<Data>;
    fn info(&self) -> Option<VideoInfo>;
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
