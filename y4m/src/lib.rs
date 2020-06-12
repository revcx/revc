//! # YUV4MPEG2 (.y4m) Encoder/Decoder

use std::fmt;
use std::io;
use std::io::Read;
use std::io::Write;
use std::num;
use std::str;

const MAX_PARAMS_SIZE: usize = 1024;
const FILE_MAGICK: &[u8] = b"YUV4MPEG2 ";
const FRAME_MAGICK: &[u8] = b"FRAME";
const TERMINATOR: u8 = 0x0A;
const FIELD_SEP: u8 = b' ';
const RATIO_SEP: u8 = b':';

/// Both encoding and decoding errors.
#[derive(Debug)]
pub enum Error {
    /// End of the file. Technically not an error, but it's easier to process
    /// that way.
    EOF,
    /// Bad input parameters provided.
    BadInput,
    /// Unknown colorspace (possibly just unimplemented).
    UnknownColorspace,
    /// Error while parsing the file/frame header.
    // TODO(Kagami): Better granularity of parse errors.
    ParseError,
    /// Error while reading/writing the file.
    IoError(io::Error),
}

macro_rules! parse_error {
    () => {
        return Err(Error::ParseError);
    };
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        match err.kind() {
            io::ErrorKind::UnexpectedEof => Error::EOF,
            _ => Error::IoError(err),
        }
    }
}

impl From<num::ParseIntError> for Error {
    fn from(_: num::ParseIntError) -> Error {
        Error::ParseError
    }
}

impl From<str::Utf8Error> for Error {
    fn from(_: str::Utf8Error) -> Error {
        Error::ParseError
    }
}

trait EnhancedRead {
    fn read_until(&mut self, ch: u8, buf: &mut [u8]) -> Result<usize, Error>;
}

impl<R: Read> EnhancedRead for R {
    // Current implementation does one `read` call per byte. This might be a
    // bit slow for long headers but it simplifies things: we don't need to
    // check whether start of the next frame is already read and so on.
    fn read_until(&mut self, ch: u8, buf: &mut [u8]) -> Result<usize, Error> {
        let mut collected = 0;
        while collected < buf.len() {
            let chunk_size = self.read(&mut buf[collected..=collected])?;
            if chunk_size == 0 {
                return Err(Error::EOF);
            }
            if buf[collected] == ch {
                return Ok(collected);
            }
            collected += chunk_size;
        }
        parse_error!()
    }
}

fn parse_bytes(buf: &[u8]) -> Result<usize, Error> {
    // A bit kludgy but seems like there is no other way.
    Ok(str::from_utf8(buf)?.parse()?)
}

/// Simple ratio structure since stdlib lacks one.
#[derive(Debug, Clone, Copy)]
pub struct Ratio {
    /// Numerator.
    pub num: usize,
    /// Denominator.
    pub den: usize,
}

impl Ratio {
    /// Create a new ratio.
    pub fn new(num: usize, den: usize) -> Ratio {
        Ratio { num, den }
    }
}

impl fmt::Display for Ratio {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.num, self.den)
    }
}

/// Colorspace (color model/pixel format). Only subset of them is supported.
///
/// From libavformat/yuv4mpegenc.c:
///
/// > yuv4mpeg can only handle yuv444p, yuv422p, yuv420p, yuv411p and gray8
/// pixel formats. And using 'strict -1' also yuv444p9, yuv422p9, yuv420p9,
/// yuv444p10, yuv422p10, yuv420p10, yuv444p12, yuv422p12, yuv420p12,
/// yuv444p14, yuv422p14, yuv420p14, yuv444p16, yuv422p16, yuv420p16, gray9,
/// gray10, gray12 and gray16 pixel formats.
#[derive(Debug, Clone, Copy)]
pub enum Colorspace {
    /// Grayscale only, 8-bit.
    Cmono,
    /// 4:2:0 with coincident chroma planes, 8-bit.
    C420,
    /// 4:2:0 with coincident chroma planes, 10-bit.
    C420p10,
    /// 4:2:0 with coincident chroma planes, 12-bit.
    C420p12,
    /// 4:2:0 with biaxially-displaced chroma planes, 8-bit.
    C420jpeg,
    /// 4:2:0 with vertically-displaced chroma planes, 8-bit.
    C420paldv,
    /// Found in some files. Same as `C420`.
    C420mpeg2,
    /// 4:2:2, 8-bit.
    C422,
    /// 4:2:2, 10-bit.
    C422p10,
    /// 4:2:2, 12-bit.
    C422p12,
    /// 4:4:4, 8-bit.
    C444,
    /// 4:4:4, 10-bit.
    C444p10,
    /// 4:4:4, 12-bit.
    C444p12,
}

impl Colorspace {
    /// Return the bit depth per sample
    #[inline]
    pub fn get_bit_depth(self) -> usize {
        match self {
            Colorspace::Cmono
            | Colorspace::C420
            | Colorspace::C422
            | Colorspace::C444
            | Colorspace::C420jpeg
            | Colorspace::C420paldv
            | Colorspace::C420mpeg2 => 8,
            Colorspace::C420p10 | Colorspace::C422p10 | Colorspace::C444p10 => 10,
            Colorspace::C420p12 | Colorspace::C422p12 | Colorspace::C444p12 => 12,
        }
    }

    /// Return the number of bytes in a sample
    #[inline]
    pub fn get_bytes_per_sample(self) -> usize {
        if self.get_bit_depth() <= 8 {
            1
        } else {
            2
        }
    }
}

/// get plane sizes
pub fn get_plane_sizes(
    width: usize,
    height: usize,
    colorspace: Colorspace,
) -> (usize, usize, usize) {
    let y_plane_size = width * height * colorspace.get_bytes_per_sample();

    let c420_chroma_size =
        ((width + 1) / 2) * ((height + 1) / 2) * colorspace.get_bytes_per_sample();
    let c422_chroma_size = ((width + 1) / 2) * height * colorspace.get_bytes_per_sample();

    let c420_sizes = (y_plane_size, c420_chroma_size, c420_chroma_size);
    let c422_sizes = (y_plane_size, c422_chroma_size, c422_chroma_size);
    let c444_sizes = (y_plane_size, y_plane_size, y_plane_size);

    match colorspace {
        Colorspace::Cmono => (y_plane_size, 0, 0),
        Colorspace::C420
        | Colorspace::C420p10
        | Colorspace::C420p12
        | Colorspace::C420jpeg
        | Colorspace::C420paldv
        | Colorspace::C420mpeg2 => c420_sizes,
        Colorspace::C422 | Colorspace::C422p10 | Colorspace::C422p12 => c422_sizes,
        Colorspace::C444 | Colorspace::C444p10 | Colorspace::C444p12 => c444_sizes,
    }
}

/// YUV4MPEG2 decoder.
pub struct Decoder<'d, R: Read + 'd> {
    reader: &'d mut R,
    params_buf: Vec<u8>,
    frame_buf: Vec<u8>,
    raw_params: Vec<u8>,
    width: usize,
    height: usize,
    framerate: Ratio,
    colorspace: Colorspace,
    y_len: usize,
    u_len: usize,
}

impl<'d, R: Read> Decoder<'d, R> {
    /// Create a new decoder instance.
    pub fn new(reader: &mut R) -> Result<Decoder<R>, Error> {
        let mut params_buf = vec![0; MAX_PARAMS_SIZE];
        let end_params_pos = reader.read_until(TERMINATOR, &mut params_buf)?;
        if end_params_pos < FILE_MAGICK.len() || !params_buf.starts_with(FILE_MAGICK) {
            parse_error!()
        }
        let raw_params = (&params_buf[FILE_MAGICK.len()..end_params_pos]).to_owned();
        let mut width = 0;
        let mut height = 0;
        // Framerate is actually required per spec, but let's be a bit more
        // permissive as per ffmpeg behavior.
        let mut framerate = Ratio::new(25, 1);
        let mut colorspace = None;
        // We shouldn't convert it to string because encoding is unspecified.
        for param in raw_params.split(|&b| b == FIELD_SEP) {
            if param.is_empty() {
                continue;
            }
            let (name, value) = (param[0], &param[1..]);
            // TODO(Kagami): interlacing, pixel aspect, comment.
            match name {
                b'W' => width = parse_bytes(value)?,
                b'H' => height = parse_bytes(value)?,
                b'F' => {
                    let parts: Vec<_> = value.splitn(2, |&b| b == RATIO_SEP).collect();
                    if parts.len() != 2 {
                        parse_error!()
                    }
                    let num = parse_bytes(parts[0])?;
                    let den = parse_bytes(parts[1])?;
                    framerate = Ratio::new(num, den);
                }
                b'C' => {
                    colorspace = match value {
                        b"mono" => Some(Colorspace::Cmono),
                        b"420" => Some(Colorspace::C420),
                        b"420p10" => Some(Colorspace::C420p10),
                        b"420p12" => Some(Colorspace::C420p12),
                        b"422" => Some(Colorspace::C422),
                        b"422p10" => Some(Colorspace::C422p10),
                        b"422p12" => Some(Colorspace::C422p12),
                        b"444" => Some(Colorspace::C444),
                        b"444p10" => Some(Colorspace::C444p10),
                        b"444p12" => Some(Colorspace::C444p12),
                        b"420jpeg" => Some(Colorspace::C420jpeg),
                        b"420paldv" => Some(Colorspace::C420paldv),
                        b"420mpeg2" => Some(Colorspace::C420mpeg2),
                        _ => return Err(Error::UnknownColorspace),
                    }
                }
                _ => {}
            }
        }
        let colorspace = colorspace.unwrap_or(Colorspace::C420);
        if width == 0 || height == 0 {
            parse_error!()
        }
        let (y_len, u_len, v_len) = get_plane_sizes(width, height, colorspace);
        let frame_size = y_len + u_len + v_len;
        let frame_buf = vec![0; frame_size];
        Ok(Decoder {
            reader,
            params_buf,
            frame_buf,
            raw_params,
            width,
            height,
            framerate,
            colorspace,
            y_len,
            u_len,
        })
    }

    /// Iterate over frames, without extra heap allocations. End of input is
    /// indicated by `Error::EOF`.
    pub fn read_frame(&mut self) -> Result<Frame, Error> {
        let end_params_pos = self.reader.read_until(TERMINATOR, &mut self.params_buf)?;
        if end_params_pos < FRAME_MAGICK.len() || !self.params_buf.starts_with(FRAME_MAGICK) {
            parse_error!()
        }
        // We don't parse frame params currently but user has access to them.
        let start_params_pos = FRAME_MAGICK.len();
        let raw_params = if end_params_pos - start_params_pos > 0 {
            // Check for extra space.
            if self.params_buf[start_params_pos] != FIELD_SEP {
                parse_error!()
            }
            Some((&self.params_buf[start_params_pos + 1..end_params_pos]).to_owned())
        } else {
            None
        };
        self.reader.read_exact(&mut self.frame_buf)?;
        Ok(Frame::new(
            [
                &self.frame_buf[0..self.y_len],
                &self.frame_buf[self.y_len..self.y_len + self.u_len],
                &self.frame_buf[self.y_len + self.u_len..],
            ],
            raw_params,
        ))
    }

    /// Return file width.
    #[inline]
    pub fn get_width(&self) -> usize {
        self.width
    }
    /// Return file height.
    #[inline]
    pub fn get_height(&self) -> usize {
        self.height
    }
    /// Return file framerate.
    #[inline]
    pub fn get_framerate(&self) -> Ratio {
        self.framerate
    }
    /// Return file colorspace.
    ///
    /// **NOTE:** normally all .y4m should have colorspace param, but there are
    /// files encoded without that tag and it's unclear what should we do in
    /// that case. Currently C420 is implied by default as per ffmpeg behavior.
    #[inline]
    pub fn get_colorspace(&self) -> Colorspace {
        self.colorspace
    }
    /// Return file raw parameters.
    #[inline]
    pub fn get_raw_params(&self) -> &[u8] {
        &self.raw_params
    }
    /// Return the bit depth per sample
    #[inline]
    pub fn get_bit_depth(&self) -> usize {
        self.colorspace.get_bit_depth()
    }
    /// Return the number of bytes in a sample
    #[inline]
    pub fn get_bytes_per_sample(&self) -> usize {
        self.colorspace.get_bytes_per_sample()
    }
}

/// A single frame.
#[derive(Debug)]
pub struct Frame<'f> {
    planes: [&'f [u8]; 3],
    raw_params: Option<Vec<u8>>,
}

impl<'f> Frame<'f> {
    /// Create a new frame with optional parameters.
    /// No heap allocations are made.
    pub fn new(planes: [&'f [u8]; 3], raw_params: Option<Vec<u8>>) -> Frame<'f> {
        Frame { planes, raw_params }
    }

    /// Create a new frame from data in 16-bit format.
    pub fn from_u16(planes: [&'f [u16]; 3], raw_params: Option<Vec<u8>>) -> Frame<'f> {
        Frame::new(
            [
                unsafe {
                    std::slice::from_raw_parts::<u8>(
                        planes[0].as_ptr() as *const u8,
                        planes[0].len() * 2,
                    )
                },
                unsafe {
                    std::slice::from_raw_parts::<u8>(
                        planes[1].as_ptr() as *const u8,
                        planes[1].len() * 2,
                    )
                },
                unsafe {
                    std::slice::from_raw_parts::<u8>(
                        planes[2].as_ptr() as *const u8,
                        planes[2].len() * 2,
                    )
                },
            ],
            raw_params,
        )
    }

    /// Return Y (first) plane.
    #[inline]
    pub fn get_y_plane(&self) -> &[u8] {
        self.planes[0]
    }
    /// Return U (second) plane. Empty in case of grayscale.
    #[inline]
    pub fn get_u_plane(&self) -> &[u8] {
        self.planes[1]
    }
    /// Return V (third) plane. Empty in case of grayscale.
    #[inline]
    pub fn get_v_plane(&self) -> &[u8] {
        self.planes[2]
    }
    /// Return frame raw parameters if any.
    #[inline]
    pub fn get_raw_params(&self) -> Option<&[u8]> {
        self.raw_params.as_ref().map(|v| &v[..])
    }
}

/// Encoder builder. Allows to set y4m file parameters using builder pattern.
// TODO(Kagami): Accept all known tags and raw params.
#[derive(Debug)]
pub struct EncoderBuilder {
    width: usize,
    height: usize,
    framerate: Ratio,
    colorspace: Colorspace,
}

impl EncoderBuilder {
    /// Create a new encoder builder.
    pub fn new(width: usize, height: usize, framerate: Ratio) -> EncoderBuilder {
        EncoderBuilder {
            width,
            height,
            framerate,
            colorspace: Colorspace::C420,
        }
    }

    /// Specify file colorspace.
    pub fn with_colorspace(mut self, colorspace: Colorspace) -> Self {
        self.colorspace = colorspace;
        self
    }

    /// Write header to the stream and create encoder instance.
    pub fn write_header<W: Write>(self, writer: &mut W) -> Result<Encoder<W>, Error> {
        // XXX(Kagami): Beware that FILE_MAGICK already contains space.
        writer.write_all(FILE_MAGICK)?;
        write!(
            writer,
            "W{} H{} F{}",
            self.width, self.height, self.framerate
        )?;
        write!(writer, " {:?}", self.colorspace)?;
        writer.write_all(&[TERMINATOR])?;
        let (y_len, u_len, v_len) = get_plane_sizes(self.width, self.height, self.colorspace);
        Ok(Encoder {
            writer,
            y_len,
            u_len,
            v_len,
        })
    }
}

/// YUV4MPEG2 encoder.
pub struct Encoder<'e, W: Write + 'e> {
    pub writer: &'e mut W,
    pub y_len: usize,
    pub u_len: usize,
    pub v_len: usize,
}

impl<'e, W: Write> Encoder<'e, W> {
    /// Write next frame to the stream.
    pub fn write_frame(&mut self, frame: &Frame) -> Result<(), Error> {
        if frame.get_y_plane().len() != self.y_len
            || frame.get_u_plane().len() != self.u_len
            || frame.get_v_plane().len() != self.v_len
        {
            return Err(Error::BadInput);
        }
        self.writer.write_all(FRAME_MAGICK)?;
        if let Some(params) = frame.get_raw_params() {
            self.writer.write_all(&[FIELD_SEP])?;
            self.writer.write_all(params)?;
        }
        self.writer.write_all(&[TERMINATOR])?;
        self.writer.write_all(frame.get_y_plane())?;
        self.writer.write_all(frame.get_u_plane())?;
        self.writer.write_all(frame.get_v_plane())?;
        Ok(())
    }
}

/// Create a new decoder instance. Alias for `Decoder::new`.
pub fn decode<R: Read>(reader: &mut R) -> Result<Decoder<R>, Error> {
    Decoder::new(reader)
}

/// Create a new encoder builder. Alias for `EncoderBuilder::new`.
pub fn encode(width: usize, height: usize, framerate: Ratio) -> EncoderBuilder {
    EncoderBuilder::new(width, height, framerate)
}
