use std::fs::File;
use std::io;
use std::io::Read;

use super::*;
use revc::api::frame::*;
use revc::api::*;

pub struct YuvDemuxer {
    reader: Box<dyn Read>,
    info: Option<VideoInfo>,
}

impl YuvDemuxer {
    pub fn new(path: &str, info: Option<VideoInfo>) -> io::Result<Box<dyn Demuxer>> {
        Ok(Box::new(YuvDemuxer {
            reader: match path {
                "-" => Box::new(io::stdin()),
                f => Box::new(File::open(&f).unwrap()),
            },
            info,
        }))
    }
}

impl Demuxer for YuvDemuxer {
    fn read(&mut self) -> io::Result<Data> {
        if let Some(info) = &self.info {
            let bytes_per_sample = if info.bit_depth > 8 { 2 } else { 1 };
            let pitch_y = info.width * bytes_per_sample;
            let height = info.height;
            let chroma_sampling_period = info.chroma_sampling.sampling_period();
            let (pitch_uv, height_uv) = (
                (pitch_y * bytes_per_sample) / chroma_sampling_period.0,
                height / chroma_sampling_period.1,
            );

            let (mut rec_y, mut rec_u, mut rec_v) = (
                vec![128u8; pitch_y * height],
                vec![128u8; pitch_uv * height_uv],
                vec![128u8; pitch_uv * height_uv],
            );

            self.reader.read_exact(&mut rec_y)?;
            self.reader.read_exact(&mut rec_u)?;
            self.reader.read_exact(&mut rec_v)?;

            let mut frame: Frame<u16> = Frame::new(info.width, info.height, info.chroma_sampling);

            frame.planes[0].copy_from_raw_u8(&rec_y, pitch_y, bytes_per_sample);
            frame.planes[1].copy_from_raw_u8(&rec_u, pitch_uv, bytes_per_sample);
            frame.planes[2].copy_from_raw_u8(&rec_v, pitch_uv, bytes_per_sample);

            Ok(Data::Frame(Some(frame)))
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid VideoInfo for YuvDemuxer",
            ))
        }
    }

    fn info(&self) -> Option<VideoInfo> {
        self.info
    }
}
