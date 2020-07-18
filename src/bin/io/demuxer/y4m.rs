use super::Demuxer;
use crate::{map_y4m_error, Data};

use std::fs::File;
use std::io;
use std::io::Read;

use revc::api::frame::*;
use revc::api::*;

pub struct Y4mDemuxer {
    reader: y4m::Decoder<Box<dyn Read>>,
}

impl Y4mDemuxer {
    pub fn new(path: &str) -> io::Result<Box<dyn Demuxer>> {
        let reader: Box<dyn Read> = match path {
            "-" => Box::new(io::stdin()),
            f => Box::new(File::open(&f).unwrap()),
        };

        Ok(Box::new(Y4mDemuxer {
            reader: y4m::Decoder::new(reader).map_err(|e| map_y4m_error(e))?,
        }))
    }
}

impl Demuxer for Y4mDemuxer {
    fn read(&mut self) -> io::Result<Data> {
        let width = self.reader.get_width();
        let height = self.reader.get_height();
        let bytes = self.reader.get_bytes_per_sample();
        let color_space = self.reader.get_colorspace();
        let chroma_sampling = map_y4m_color_space(color_space);
        let (xdec, _) = chroma_sampling.sampling_period();
        let chroma_width = (width + 1) >> xdec;
        let frame = self
            .reader
            .read_frame()
            .map(|frame| {
                let mut f: Frame<u16> = Frame::new(width, height, chroma_sampling);

                f.planes[0].copy_from_raw_u8(frame.get_y_plane(), width * bytes, bytes);
                f.planes[1].copy_from_raw_u8(frame.get_u_plane(), chroma_width * bytes, bytes);
                f.planes[2].copy_from_raw_u8(frame.get_v_plane(), chroma_width * bytes, bytes);
                f
            })
            .map_err(|e| map_y4m_error(e))?;

        Ok(Data::Frame(frame))
    }
}

fn map_y4m_color_space(color_space: y4m::Colorspace) -> ChromaSampling {
    use crate::ChromaSampling::*;
    use y4m::Colorspace::*;
    match color_space {
        Cmono => Cs400,
        C420jpeg | C420paldv => Cs420,
        C420mpeg2 => Cs420,
        C420 | C420p10 | C420p12 => Cs420,
        C422 | C422p10 | C422p12 => Cs422,
        C444 | C444p10 | C444p12 => Cs444,
    }
}
