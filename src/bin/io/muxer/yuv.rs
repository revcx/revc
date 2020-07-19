use super::Muxer;
use crate::{Data, IFVCA_CLIP};

use std::fs::File;
use std::io;
use std::io::Write;
use std::slice;

use revc::api::frame::*;
use revc::api::Rational;

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
    fn write(&mut self, data: Data, bitdepth: u8, frame_rate: Rational) -> io::Result<()> {
        if let Data::RefFrame(frame) = &data {
            let f = frame.borrow();
            let bytes_per_sample = if bitdepth > 8 { 2 } else { 1 };
            let pitch_y = f.planes[0].cfg.width * bytes_per_sample;
            let height = f.planes[0].cfg.height;
            let chroma_sampling_period = f.chroma_sampling.sampling_period();
            let (pitch_uv, height_uv) = (
                (pitch_y * bytes_per_sample) / chroma_sampling_period.0,
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
                if bitdepth > 8 {
                    unsafe {
                        line_out.copy_from_slice(slice::from_raw_parts::<u8>(
                            line.as_ptr() as *const u8,
                            pitch_y,
                        ));
                    }
                } else {
                    line_out.copy_from_slice(
                        &line
                            .iter()
                            .map(|&v| u8::cast_from(IFVCA_CLIP(0, 255, (v + 2) >> 2)))
                            .collect::<Vec<u8>>()[..pitch_y],
                    );
                }
            }
            for (line, line_out) in f.planes[1]
                .data_origin()
                .chunks(stride_u)
                .zip(rec_u.chunks_mut(pitch_uv))
            {
                if bitdepth > 8 {
                    unsafe {
                        line_out.copy_from_slice(slice::from_raw_parts::<u8>(
                            line.as_ptr() as *const u8,
                            pitch_uv,
                        ));
                    }
                } else {
                    line_out.copy_from_slice(
                        &line
                            .iter()
                            .map(|&v| u8::cast_from(IFVCA_CLIP(0, 255, (v + 2) >> 2)))
                            .collect::<Vec<u8>>()[..pitch_uv],
                    );
                }
            }
            for (line, line_out) in f.planes[2]
                .data_origin()
                .chunks(stride_v)
                .zip(rec_v.chunks_mut(pitch_uv))
            {
                if bitdepth > 8 {
                    unsafe {
                        line_out.copy_from_slice(slice::from_raw_parts::<u8>(
                            line.as_ptr() as *const u8,
                            pitch_uv,
                        ));
                    }
                } else {
                    line_out.copy_from_slice(
                        &line
                            .iter()
                            .map(|&v| u8::cast_from(IFVCA_CLIP(0, 255, (v + 2) >> 2)))
                            .collect::<Vec<u8>>()[..pitch_uv],
                    );
                }
            }

            self.writer.write_all(&rec_y)?;
            self.writer.write_all(&rec_u)?;
            self.writer.write_all(&rec_v)?;

            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Frame Data for YuvMuxer",
            ))
        }
    }
}
