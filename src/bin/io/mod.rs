use revc::api::{frame::Frame, Packet};

pub mod demuxer;
pub mod muxer;

pub enum Data<'a> {
    RefFrame(&'a Frame<u16>),
    Frame(Frame<u16>),
    Packet(Packet),
}
