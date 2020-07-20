use std::cell::RefCell;
use std::rc::Rc;
use std::vec::Vec;
use std::{cmp, fmt, io};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

pub mod frame;

use crate::dec::*;
use crate::def::*;
use frame::*;

/*****************************************************************************
 * return values and error code
 *****************************************************************************/
/* not matched CRC value */
pub const EVC_ERR_BAD_CRC: usize = (201);
/* CRC value presented but ignored at decoder*/
pub const EVC_WARN_CRC_IGNORED: usize = (200);
pub const EVC_OK: usize = 0;

#[derive(Debug, FromPrimitive, ToPrimitive, PartialOrd, Ord, PartialEq, Eq)]
pub enum EvcError {
    /* no more output, but it is OK */
    EVC_OK_NO_MORE_OUTPUT = 205,
    /* progress success, but output is not available temporarily */
    EVC_OK_OUTPUT_NOT_AVAILABLE = 204,
    /* frame dimension (width or height) has been changed */
    EVC_OK_DIM_CHANGED = 203,
    /* flush decoding process */
    EVC_OK_FLUSH = 202,

    EVC_ERR = (-1), /* generic error */
    EVC_ERR_INVALID_ARGUMENT = (-101),
    EVC_ERR_OUT_OF_MEMORY = (-102),
    EVC_ERR_REACHED_MAX = (-103),
    EVC_ERR_UNSUPPORTED = (-104),
    EVC_ERR_UNEXPECTED = (-105),
    EVC_ERR_UNSUPPORTED_COLORSPACE = (-201),
    EVC_ERR_MALFORMED_BITSTREAM = (-202),
    EVC_ERR_EMPTY_PACKET = (-203),

    EVC_ERR_UNKNOWN = (-32767), /* unknown error */
}

impl Default for EvcError {
    fn default() -> Self {
        EvcError::EVC_ERR
    }
}

pub const NALU_SIZE_FIELD_IN_BYTES: usize = 4;

#[allow(dead_code, non_camel_case_types)]
#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq, PartialOrd, Clone, Copy)]
pub enum NaluType {
    EVC_NONIDR_NUT = 0,
    EVC_IDR_NUT = 1,
    EVC_SPS_NUT = 24,
    EVC_PPS_NUT = 25,
    EVC_APS_NUT = 26,
    EVC_FD_NUT = 27,
    EVC_SEI_NUT = 28,
    EVC_UNKNOWN_NUT,
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
            EVC_FD_NUT => write!(f, "Filler Data"),
            EVC_SEI_NUT => write!(f, "Supplemental Enhancement Information"),
            EVC_UNKNOWN_NUT => write!(f, "Unknown"),
        }
    }
}

impl From<u8> for NaluType {
    fn from(val: u8) -> Self {
        use self::NaluType::*;
        match val {
            0 => EVC_NONIDR_NUT,
            1 => EVC_IDR_NUT,
            24 => EVC_SPS_NUT,
            25 => EVC_PPS_NUT,
            26 => EVC_APS_NUT,
            27 => EVC_FD_NUT,
            28 => EVC_SEI_NUT,
            _ => EVC_UNKNOWN_NUT,
        }
    }
}

impl Default for NaluType {
    fn default() -> Self {
        NaluType::EVC_NONIDR_NUT
    }
}

#[allow(dead_code, non_camel_case_types)]
#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq, PartialOrd, Clone, Copy)]
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
            EVC_ST_UNKNOWN => write!(f, "Unknown"),
            EVC_ST_I => write!(f, "I"),
            EVC_ST_P => write!(f, "P"),
            EVC_ST_B => write!(f, "B"),
        }
    }
}

impl From<u8> for SliceType {
    fn from(val: u8) -> Self {
        use self::SliceType::*;
        match val {
            1 => EVC_ST_I,
            2 => EVC_ST_P,
            3 => EVC_ST_B,
            _ => EVC_ST_UNKNOWN,
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
    pub refpic: [[isize; 16]; 2], //[2][16]

    pub ret: usize,
}

pub const MAX_NUM_REF_PICS: usize = 21;
pub const MAX_NUM_ACTIVE_REF_FRAME: usize = 5;
pub const MAX_NUM_RPLS: usize = 32;

/* rpl structure */
#[derive(Default)]
pub struct EvcRpl {
    pub poc: usize,
    pub tid: usize,
    pub ref_pic_num: u8,
    pub ref_pic_active_num: u8,
    pub ref_pics: [u8; MAX_NUM_REF_PICS],
    pub pic_type: u8,
}

pub const MAX_QP_TABLE_SIZE: usize = 58;
pub const MAX_QP_TABLE_SIZE_EXT: usize = 70;

/* chromaQP table structure to be signalled in SPS*/
pub struct EvcChromaTable {
    pub chroma_qp_table_present_flag: bool,
    pub same_qp_table_for_chroma: bool,
    pub global_offset_flag: bool,
    pub num_points_in_qp_table_minus1: [usize; 2],
    pub delta_qp_in_val_minus1: [[i8; MAX_QP_TABLE_SIZE]; 2],
    pub delta_qp_out_val: [[i8; MAX_QP_TABLE_SIZE]; 2],
}

static default_qp_talbe: [[i8; MAX_QP_TABLE_SIZE]; 2] = [[0; MAX_QP_TABLE_SIZE]; 2];
impl Default for EvcChromaTable {
    fn default() -> Self {
        EvcChromaTable {
            chroma_qp_table_present_flag: false,
            same_qp_table_for_chroma: false,
            global_offset_flag: false,
            num_points_in_qp_table_minus1: [0; 2],
            delta_qp_in_val_minus1: default_qp_talbe,
            delta_qp_out_val: default_qp_talbe,
        }
    }
}

pub enum Data {
    Empty,
    RefFrame(Rc<RefCell<Frame<pel>>>),
    Frame(Frame<pel>),
    RefPacket(Rc<RefCell<Packet>>),
    Packet(Packet),
}

#[derive(Debug, Default)]
pub struct Packet {
    pub data: Option<Vec<u8>>,
    pub pts: u64,
}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = if let Some(data) = &self.data {
            data.len()
        } else {
            0
        };
        write!(f, "Packet {} - {} bytes", self.pts, len)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, FromPrimitive)]
#[repr(C)]
pub enum ChromaSampling {
    Cs400,
    Cs420,
    Cs422,
    Cs444,
}

impl Default for ChromaSampling {
    fn default() -> Self {
        ChromaSampling::Cs420
    }
}

impl From<u8> for ChromaSampling {
    fn from(val: u8) -> Self {
        use self::ChromaSampling::*;
        match val {
            0 => Cs400,
            1 => Cs420,
            2 => Cs422,
            _ => Cs444,
        }
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

#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive)]
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

#[derive(Clone, Copy, Debug, Default)]
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

// Due to the math in RCState::new() regarding the reservoir frame delay.
pub const MAX_MAX_KEY_FRAME_INTERVAL: u64 = i32::max_value() as u64 / 3;

#[derive(Clone, Copy, Debug, Default)]
pub struct EncoderConfig {
    // output size
    // Width of the frames in pixels.
    pub width: usize,
    // Height of the frames in pixels.
    pub height: usize,
    // Video time base.
    pub time_base: Rational,

    // Bit depth.
    pub bit_depth: usize,
    // Chroma subsampling.
    pub chroma_sampling: ChromaSampling,

    // encoder configuration
    // The *minimum* interval between two keyframes
    pub min_key_frame_interval: u64,
    // The *maximum* interval between two keyframes
    pub max_key_frame_interval: u64, //iperiod

    // The base quantizer to use.
    pub qp: u8,
    // The minimum allowed base quantizer to use in bitrate mode.
    pub min_qp: u8,
    // The maximum allowed base quantizer to use in bitrate mode.
    pub max_qp: u8,
    // The target bitrate for the bitrate mode.
    pub bitrate: i32,

    pub cb_qp_offset: i8,
    pub cr_qp_offset: i8,
    pub use_dqp: u8,
    pub cu_qp_delta_area: u8,
    pub max_b_frames: u8,
    pub ref_pic_gap_length: u8,
    pub closed_gop: bool,
    pub level: u8,
    pub enable_cip: bool,
    pub disable_dbf: bool,
    pub num_slices_in_pic: usize,
    pub inter_slice_type: u8,
    // Settings which affect the enconding speed vs. quality trade-off.
    //pub speed_settings: SpeedSettings,
    // Rate control configuration
    // rate_control: RateControlConfig,
}

/// Contains the encoder configuration.
#[derive(Clone, Copy, Debug, Default)]
pub struct Config {
    /// The number of threads in the threadpool.
    pub threads: usize,

    /// Encoder configuration (optional)
    pub enc: Option<EncoderConfig>,
}

pub struct DecoderContext(EvcdCtx);
pub struct EncoderContext;

pub enum Context {
    Decoder(DecoderContext),
    Encoder(EncoderContext),
}

impl Context {
    pub fn new(cfg: &Config) -> Self {
        if cfg.enc.is_none() {
            Context::Decoder(DecoderContext(EvcdCtx::new()))
        } else {
            Context::Encoder(EncoderContext)
        }
    }

    pub fn push(&mut self, data: &mut Data) -> Result<(), EvcError> {
        if let Context::Decoder(ctx) = self {
            if let Data::Packet(pkt) = data {
                ctx.0.push_pkt(pkt)
            } else {
                Err(EvcError::EVC_ERR_EMPTY_PACKET)
            }
        } else {
            if let Data::Frame(frm) = data {
                Err(EvcError::EVC_ERR_UNSUPPORTED)
            } else {
                Err(EvcError::EVC_ERR_EMPTY_PACKET)
            }
        }
    }

    pub fn pull(&mut self, data: &mut Data) -> Result<Option<EvcdStat>, EvcError> {
        if let Context::Decoder(ctx) = self {
            *data = Data::Empty;

            let mut stat = None;
            let mut pull_frm = false;
            match ctx.0.decode_nalu() {
                Ok(st) => {
                    pull_frm = st.fnum >= 0;
                    stat = Some(st);
                }
                Err(err) => {
                    if err == EvcError::EVC_OK_FLUSH {
                        pull_frm = true;
                    }
                }
            }

            if pull_frm {
                let ret = ctx.0.pull_frm();
                match ret {
                    Ok(frame) => *data = Data::RefFrame(frame),
                    Err(err) => {
                        if err != EvcError::EVC_OK_OUTPUT_NOT_AVAILABLE {
                            return Err(err);
                        }
                    }
                }
            }

            Ok(stat)
        } else {
            Err(EvcError::EVC_ERR_UNSUPPORTED)
        }
    }
}
