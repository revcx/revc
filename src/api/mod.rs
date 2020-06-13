use crate::com::frame::Frame;
use crate::com::util::Pixel;
//use crate::headers::*;
//use crate::internal::*;
//use crate::obu::*;

use std::rc::Rc;
use std::vec::Vec;
use std::{cmp, fmt, io};

use arg_enum_proc_macro::ArgEnum;
use num_derive::*;

/*****************************************************************************
 * return values and error code
 *****************************************************************************/
#[derive(FromPrimitive, ToPrimitive, PartialOrd, Ord, PartialEq, Eq)]
pub enum EvcStatus {
    /* no more frames, but it is OK */
    EVC_OK_NO_MORE_FRM = 205,
    /* progress success, but output is not available temporarily */
    EVC_OK_OUT_NOT_AVAILABLE = 204,
    /* frame dimension (width or height) has been changed */
    EVC_OK_DIM_CHANGED = (203),
    /* decoding success, but output frame has been delayed */
    EVC_OK_FRM_DELAYED = (202),
    /* not matched CRC value */
    EVC_ERR_BAD_CRC = (201),
    /* CRC value presented but ignored at decoder*/
    EVC_WARN_CRC_IGNORED = (200),

    EVC_OK = 0,

    EVC_ERR = (-1), /* generic error */
    EVC_ERR_INVALID_ARGUMENT = (-101),
    EVC_ERR_OUT_OF_MEMORY = (-102),
    EVC_ERR_REACHED_MAX = (-103),
    EVC_ERR_UNSUPPORTED = (-104),
    EVC_ERR_UNEXPECTED = (-105),
    EVC_ERR_UNSUPPORTED_COLORSPACE = (-201),
    EVC_ERR_MALFORMED_BITSTREAM = (-202),

    EVC_ERR_UNKNOWN = (-32767), /* unknown error */
}

impl Default for EvcStatus {
    fn default() -> Self {
        EvcStatus::EVC_OK
    }
}

#[allow(dead_code, non_camel_case_types)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(C)]
pub enum NaluType {
    EVC_NONIDR_NUT = 0,
    EVC_IDR_NUT = 1,
    EVC_SPS_NUT = 24,
    EVC_PPS_NUT = 25,
    EVC_APS_NUT = 26,
    EVC_SEI_NUT = 27,
}

impl fmt::Display for NaluType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::NaluType::*;
        match self {
            EVC_NONIDR_NUT => write!(f, "Non-IDR"),
            EVC_IDR_NUT => write!(f, "Instantaneous Decoder Refresh"),
            EVC_SPS_NUT => write!(f, "Sequence Parameter Se"),
            EVC_PPS_NUT => write!(f, "Picture Parameter Set"),
            EVC_APS_NUT => write!(f, "Adaptation Parameter Set"),
            EVC_SEI_NUT => write!(f, "Supplemental Enhancement Information"),
        }
    }
}

impl Default for NaluType {
    fn default() -> Self {
        NaluType::EVC_NONIDR_NUT
    }
}

#[allow(dead_code, non_camel_case_types)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(C)]
pub enum SliceType {
    EVC_ST_UNKNOWN = 0,
    EVC_ST_I = 1,
    EVC_ST_P = 2,
    EVC_ST_B = 3,
}

impl fmt::Display for SliceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::SliceType::*;
        match self {
            EVC_ST_UNKNOWN => write!(f, "Unknown Slice Type"),
            EVC_ST_I => write!(f, "I Slice"),
            EVC_ST_P => write!(f, "P Slice"),
            EVC_ST_B => write!(f, "B Slice"),
        }
    }
}

impl Default for SliceType {
    fn default() -> Self {
        SliceType::EVC_ST_UNKNOWN
    }
}

/*****************************************************************************
 * status after decoder operation
 *****************************************************************************/
#[derive(Debug, Default)]
pub struct EvcdStat {
    /* byte size of decoded bitstream (read size of bitstream) */
    pub read: usize,
    /* nalu type */
    pub nalu_type: NaluType,
    /* slice type */
    pub stype: SliceType,
    /* frame number monotonically increased whenever decoding a frame.
    note that it has negative value if the decoded data is not frame */
    pub fnum: isize,
    /* picture order count */
    pub poc: isize,
    /* layer id */
    pub tid: isize,

    /* number of reference pictures */
    pub refpic_num: [u8; 2],
    /* list of reference pictures */
    pub refpic: [[isize; 2]; 16],
}

pub struct Packet {
    pub data: Vec<u8>,
    pub offset: usize,
    pub pts: u64,
}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Packet {} - {} bytes", self.pts, self.data.len())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, FromPrimitive)]
#[repr(C)]
pub enum ChromaSampling {
    Cs420,
    Cs422,
    Cs444,
    Cs400,
}

impl Default for ChromaSampling {
    fn default() -> Self {
        ChromaSampling::Cs420
    }
}

impl ChromaSampling {
    // Provides the sampling period in the horizontal and vertical axes.
    pub fn sampling_period(self) -> (usize, usize) {
        use self::ChromaSampling::*;
        match self {
            Cs420 => (2, 2),
            Cs422 => (2, 1),
            Cs444 => (1, 1),
            Cs400 => (2, 2),
        }
    }
}

#[derive(ArgEnum, Debug, Clone, Copy, PartialEq, FromPrimitive)]
#[repr(C)]
pub enum PixelRange {
    Unspecified = 0,
    Limited,
    Full,
}

impl Default for PixelRange {
    fn default() -> Self {
        PixelRange::Unspecified
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Rational {
    pub num: u64,
    pub den: u64,
}

impl Rational {
    pub fn new(num: u64, den: u64) -> Self {
        Rational { num, den }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub threads: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config { threads: 0 }
    }
}

pub struct Context<T: Pixel> {
    //pub(crate) seq_hdr: Option<Rc<SequenceHeader>>,
    //pub(crate) frame_hdr: Option<Rc<FrameHeader>>,
    pub(crate) drain: bool,
    pub(crate) frame: Option<Frame<T>>,
    pub(crate) packet: Option<Packet>,
}

impl<T: Pixel> Context<T> {
    pub fn new(cfg: &Config) -> Self {
        Context {
            //seq_hdr: None,
            //frame_hdr: None,
            drain: false,
            frame: None,
            packet: None,
        }
    }

    pub fn decode(&mut self, pkt: &mut Option<Packet>) -> Result<EvcdStat, EvcStatus> {
        /*if pkt.is_none() {
            return Err(CodecStatus::NeedMoreData);
        }

        self.drain = false;

        if self.packet.is_some() {
            return Err(CodecStatus::EnoughData);
        }

        self.packet = pkt.take();
        */
        Ok(EvcdStat::default())
    }

    pub fn pull(&mut self) -> Result<Frame<T>, EvcStatus> {
        /*if self.drain {
            return self.drain_frame();
        }

        if self.packet.is_none() {
            return Err(CodecStatus::NeedMoreData);
        }

        let pkt = self.packet.as_ref().unwrap();
        let (mut offset, size) = (pkt.offset, pkt.data.len());

        while offset < size {
            /*
            let res = self.parse_obus(offset, false);
            let err = res.is_err();
            if err {
                self.packet.take(); // all packet data are consumed, then release it
            } else {
                offset += res.unwrap();
                if offset >= size {
                    self.packet.take();
                }
            }
            if self.frame.is_some() {
                break;
            } else if err {
                return Err(CodecStatus::Failure);
            }
             */
        }

        if self.packet.is_some() {
            self.packet.as_mut().unwrap().offset = offset;
        }

        let frame = self.frame.take();
        match frame {
            Some(f) => Ok(f),
            None => Err(CodecStatus::NeedMoreData),
        }*/
        Err(EvcStatus::default())
    }

    pub fn flush(&mut self) {
        self.drain = true;
    }
}
