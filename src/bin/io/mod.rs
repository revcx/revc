use revc::api::{frame::Frame, Packet};

pub mod demuxer;
pub mod muxer;

pub enum Data<'a> {
    Frame(&'a Frame<u16>),
    Packet(Packet),
}
