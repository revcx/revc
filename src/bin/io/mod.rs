use std::cmp::*;
use std::io;

use revc::api::{frame::Frame, Packet};

pub mod demuxer;
pub mod muxer;

pub enum Data<'a> {
    RefFrame(&'a Frame<u16>),
    Frame(Frame<u16>),
    Packet(Packet),
}

/* clipping within min and max */
#[inline]
pub fn IFVCA_CLIP<T: Ord>(min_x: T, max_x: T, value: T) -> T {
    max(min_x, min(max_x, value))
}

pub fn map_y4m_error(e: y4m::Error) -> io::Error {
    match e {
        y4m::Error::EOF => io::Error::new(io::ErrorKind::UnexpectedEof, "y4m: End of File"),
        y4m::Error::BadInput => io::Error::new(io::ErrorKind::InvalidInput, "y4m: Bad Input"),
        y4m::Error::UnknownColorspace => {
            io::Error::new(io::ErrorKind::Other, "y4m: Unknown Color Space")
        }
        y4m::Error::ParseError(_) => io::Error::new(io::ErrorKind::Other, "y4m: Parse Error"),
        y4m::Error::IoError(e) => e,
        // Note that this error code has nothing to do with the system running out of memory,
        // it means the y4m decoder has exceeded its memory allocation limit.
        y4m::Error::OutOfMemory => io::Error::new(io::ErrorKind::Other, "y4m: Out of Memory"),
    }
}
