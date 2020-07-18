use super::Muxer;
use crate::{map_y4m_error, Data, IFVCA_CLIP};

use std::cmp::*;
use std::fs::File;
use std::io;
use std::io::Write;
use std::slice;

use revc::api::frame::*;
use revc::api::Rational;

pub struct Y4mMuxer {
    writer: Option<Box<dyn Write>>,
    encoder: Option<y4m::Encoder<Box<dyn Write>>>,
}

impl Y4mMuxer {
    pub fn new(path: &str) -> Box<dyn Muxer> {
        Box::new(Y4mMuxer {
            writer: Some(match path {
                "-" => Box::new(io::stdout()),
                f => Box::new(File::create(&f).unwrap()),
            }),
            encoder: None,
        })
    }
}

impl Muxer for Y4mMuxer {
    fn write(&mut self, data: Data, bit_depth: u8, frame_rate: Rational) -> io::Result<()> {
        if self.encoder.is_none() && self.writer.is_some() {
            if let Data::RefFrame(f) = data {
                let width = f.planes[0].cfg.width;
                let height = f.planes[0].cfg.height;
                let writer = self.writer.take().unwrap();
                self.encoder = Some(
                    y4m::EncoderBuilder::new(
                        width,
                        height,
                        y4m::Ratio::new(frame_rate.num as usize, frame_rate.den as usize),
                    )
                    .write_header(writer)
                    .map_err(|e| map_y4m_error(e))?,
                );
            }
        }

        if let (Data::RefFrame(f), Some(encoder)) = (data, &mut self.encoder) {
            let bytes_per_sample = if bit_depth > 8 { 2 } else { 1 };
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
                if bit_depth > 8 {
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
                if bit_depth > 8 {
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
                if bit_depth > 8 {
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

            let rec_frame = y4m::Frame::new([&rec_y, &rec_u, &rec_v], None);
            encoder
                .write_frame(&rec_frame)
                .map_err(|e| map_y4m_error(e))?;

            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Frame Data for YuvMuxer",
            ))
        }
    }
}
