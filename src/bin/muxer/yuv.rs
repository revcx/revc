use super::Muxer;

use std::fs::File;
use std::io;
use std::io::Write;
use std::slice;

use revc::api::frame::*;
use revc::api::util::*;

pub struct YuvMuxer {
    writer: Box<dyn Write>,
}

impl YuvMuxer {
    pub fn new(path: &str) -> Box<dyn Muxer> {
        Box::new(YuvMuxer {
            writer: match path {
                "-" => Box::new(io::stdout()),
                f => Box::new(File::create(&f).unwrap()),
            },
        })
    }
}

impl Muxer for YuvMuxer {
    fn write(&mut self, f: &Frame<u8>) -> io::Result<()> {
        let pitch_y = f.planes[0].cfg.width;
        let height = f.planes[0].cfg.height;
        let chroma_sampling_period = f.chroma_sampling.sampling_period();
        let (pitch_uv, height_uv) = (
            pitch_y / chroma_sampling_period.0,
            height / chroma_sampling_period.1,
        );

        let (mut rec_y, mut rec_u, mut rec_v) = (
            vec![128u8; pitch_y * height],
            vec![128u8; pitch_uv * height_uv],
            vec![128u8; pitch_uv * height_uv],
        );

        let (stride_y, stride_u, stride_v) = (
            f.planes[0].cfg.stride,
            f.planes[1].cfg.stride,
            f.planes[2].cfg.stride,
        );

        for (line, line_out) in f.planes[0]
            .data_origin()
            .chunks(stride_y)
            .zip(rec_y.chunks_mut(pitch_y))
        {
            line_out.copy_from_slice(
                &line.iter().map(|&v| u8::cast_from(v)).collect::<Vec<u8>>()[..pitch_y],
            );
        }
        for (line, line_out) in f.planes[1]
            .data_origin()
            .chunks(stride_u)
            .zip(rec_u.chunks_mut(pitch_uv))
        {
            line_out.copy_from_slice(
                &line.iter().map(|&v| u8::cast_from(v)).collect::<Vec<u8>>()[..pitch_uv],
            );
        }
        for (line, line_out) in f.planes[2]
            .data_origin()
            .chunks(stride_v)
            .zip(rec_v.chunks_mut(pitch_uv))
        {
            line_out.copy_from_slice(
                &line.iter().map(|&v| u8::cast_from(v)).collect::<Vec<u8>>()[..pitch_uv],
            );
        }

        self.writer.write_all(&rec_y)?;
        self.writer.write_all(&rec_u)?;
        self.writer.write_all(&rec_v)?;

        Ok(())
    }
}
