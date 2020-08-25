use std::cell::RefCell;
use std::rc::Rc;
use std::vec::Vec;
use std::{cmp, fmt, io};

use thiserror::Error;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

pub mod frame;

use crate::dec::*;
use crate::def::*;
use crate::enc::*;
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
    EVC_ERR_EMPTY_FRAME = (-204),

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

impl SliceType {
    #[inline]
    pub(crate) fn IS_INTRA_SLICE(&self) -> bool {
        *self == SliceType::EVC_ST_I
    }

    #[inline]
    pub(crate) fn IS_INTER_SLICE(&self) -> bool {
        *self == SliceType::EVC_ST_P || *self == SliceType::EVC_ST_B
    }
}

/*****************************************************************************
 * status after decoder/encoder operation
 *****************************************************************************/
#[derive(Debug, Default)]
pub struct EvcStat {
    /* byte size of decoded/encoded bitstream (read/write size of bitstream) */
    pub bytes: usize,
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

    // encoder only
    /* encoded sei messages byte size */
    pub sei_size: usize,
    /* picture number increased whenever encoding a frame */
    /* quantization parameter used for encoding */
    pub qp: u8,
    pub rec: Option<Rc<RefCell<Frame<pel>>>>,
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
    Frame(Option<Frame<pel>>),
    RefPacket(Rc<RefCell<Packet>>),
    Packet(Option<Packet>),
}

#[derive(Debug, Default)]
pub struct Packet {
    pub data: Vec<u8>,
    pub ts: u64,
}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Packet {} - {} bytes", self.ts, self.data.len())
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

/// Enumeration of possible invalid configuration errors.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Error)]
#[non_exhaustive]
pub enum InvalidConfig {
    /// The width is invalid.
    #[error("invalid width {0} (expected >= 16, <= 32767)")]
    InvalidWidth(usize),
    /// The height is invalid.
    #[error("invalid height {0} (expected >= 16, <= 32767)")]
    InvalidHeight(usize),
    /// RDO lookahead frame count is invalid.
    #[error("invalid rdo lookahead frames {actual} (expected <= {max} and >= {min})")]
    InvalidRdoLookaheadFrames {
        /// The actual value.
        actual: usize,
        /// The maximal supported value.
        max: usize,
        /// The minimal supported value.
        min: usize,
    },
    /// Maximal keyframe interval is invalid.
    #[error("invalid max keyframe interval {actual} (expected <= {max})")]
    InvalidMaxKeyFrameInterval {
        /// The actual value.
        actual: usize,
        /// The maximal supported value.
        max: usize,
    },
    /// Framerate numerator is invalid.
    #[error("invalid framerate numerator {actual} (expected > 0, <= {max})")]
    InvalidFrameRateNum {
        /// The actual value.
        actual: u64,
        /// The maximal supported value.
        max: u64,
    },
    /// Framerate denominator is invalid.
    #[error("invalid framerate denominator {actual} (expected > 0, <= {max})")]
    InvalidFrameRateDen {
        /// The actual value.
        actual: u64,
        /// The maximal supported value.
        max: u64,
    },

    /// The QP is invalid.
    #[error("invalid qp {actual} (expected <= {max} and >= {min})")]
    InvalidQP {
        /// The actual value.
        actual: u8,
        /// The maximal supported value.
        max: u8,
        /// The minimal supported value.
        min: u8,
    },

    #[error("Invalid Max B Frames")]
    InvalidMaxBFrames,
    #[error("Invalid Ref Pic GAP Length")]
    InvalidRefPicGapLength,
    #[error("Invalid Hierarchical GOP")]
    InvalidHierarchicalGOP,

    /// The rate control needs a target bitrate in order to produce results
    #[error("The rate control requires a target bitrate")]
    TargetBitrateNeeded,
}

// We add 1 to rdo_lookahead_frames in a bunch of places.
pub(crate) const MAX_RDO_LOOKAHEAD_FRAMES: usize = usize::max_value() - 1;
// Due to the math in RCState::new() regarding the reservoir frame delay.
pub const MAX_MAX_KEY_FRAME_INTERVAL: usize = i32::max_value() as usize / 3;

#[derive(Clone, Copy, Debug, Default)]
pub struct EncoderConfig {
    // output size
    // Width of the frames in pixels.
    pub width: usize,
    // Height of the frames in pixels.
    pub height: usize,
    // Video time base.
    pub time_base: Rational,
    pub fps: u64,

    // Bit depth.
    pub bit_depth: usize,
    // Chroma subsampling.
    pub chroma_sampling: ChromaSampling,

    // encoder configuration
    // The *minimum* interval between two keyframes
    pub min_key_frame_interval: usize,
    // The *maximum* interval between two keyframes
    pub max_key_frame_interval: usize, //iperiod

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
    pub cu_qp_delta_area: u8,
    pub max_b_frames: u8,
    pub ref_pic_gap_length: u8,
    pub closed_gop: bool,
    pub disable_hgop: bool,
    pub level: u8,
    pub enable_cip: bool,
    pub disable_dbf: bool,
    pub num_slices_in_pic: usize,
    pub inter_slice_type: SliceType,

    // Number of frames to read ahead for the RDO lookahead computation.
    pub rdo_lookahead_frames: usize,
    // Settings which affect the enconding speed vs. quality trade-off.
    //pub speed_settings: SpeedSettings,
    // Rate control configuration
    // rate_control: RateControlConfig,
}

impl EncoderConfig {
    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), InvalidConfig> {
        use InvalidConfig::*;

        let config = self;

        if config.width < 16 || config.width > u16::max_value() as usize || config.width & 7 != 0 {
            return Err(InvalidWidth(config.width));
        }
        if config.height < 16 || config.height > u16::max_value() as usize || config.height & 7 != 0
        {
            return Err(InvalidHeight(config.height));
        }

        if config.qp < MIN_QUANT || config.qp > MAX_QUANT {
            return Err(InvalidQP {
                actual: config.qp,
                max: MAX_QUANT,
                min: MIN_QUANT,
            });
        }

        /*if config.rdo_lookahead_frames > MAX_RDO_LOOKAHEAD_FRAMES || config.rdo_lookahead_frames < 1
        {
            return Err(InvalidRdoLookaheadFrames {
                actual: config.rdo_lookahead_frames,
                max: MAX_RDO_LOOKAHEAD_FRAMES,
                min: 1,
            });
        }*/
        if config.max_key_frame_interval > MAX_MAX_KEY_FRAME_INTERVAL {
            return Err(InvalidMaxKeyFrameInterval {
                actual: config.max_key_frame_interval,
                max: MAX_MAX_KEY_FRAME_INTERVAL,
            });
        }

        if config.time_base.num == 0 || config.time_base.num > u32::max_value() as u64 {
            return Err(InvalidFrameRateNum {
                actual: config.time_base.num,
                max: u32::max_value() as u64,
            });
        }
        if config.time_base.den == 0 || config.time_base.den > u32::max_value() as u64 {
            return Err(InvalidFrameRateDen {
                actual: config.time_base.den,
                max: u32::max_value() as u64,
            });
        }

        if !config.disable_hgop {
            if !(config.max_b_frames == 0
                || config.max_b_frames == 1
                || config.max_b_frames == 3
                || config.max_b_frames == 7
                || config.max_b_frames == 15)
            {
                return Err(InvalidMaxBFrames);
            }

            if config.max_b_frames != 0 {
                if config.max_key_frame_interval % (config.max_b_frames + 1) as usize != 0 {
                    return Err(InvalidHierarchicalGOP);
                }
            }
        }

        if config.ref_pic_gap_length != 0 && config.max_b_frames != 0 {
            return Err(InvalidMaxBFrames);
        }

        if config.max_b_frames == 0 {
            if !(config.ref_pic_gap_length == 1
                || config.ref_pic_gap_length == 2
                || config.ref_pic_gap_length == 4
                || config.ref_pic_gap_length == 8
                || config.ref_pic_gap_length == 16)
            {
                return Err(InvalidRefPicGapLength);
            }
        }

        // TODO: add more validation
        /*let rc = &self.rate_control;

        if (rc.emit_pass_data || rc.summary.is_some()) && config.bitrate == 0 {
            return Err(TargetBitrateNeeded);
        }*/

        Ok(())
    }
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
pub struct EncoderContext(EvceCtx);

pub enum Context {
    Invalid(InvalidConfig),
    Decoder(DecoderContext),
    Encoder(EncoderContext),
}

impl Context {
    pub fn new(cfg: &Config) -> Self {
        if let Some(cfg_enc) = &cfg.enc {
            match cfg_enc.validate() {
                Ok(_) => Context::Encoder(EncoderContext(EvceCtx::new(cfg))),
                Err(err) => return Context::Invalid(err),
            }
        } else {
            Context::Decoder(DecoderContext(EvcdCtx::new(cfg)))
        }
    }

    pub fn push(&mut self, data: &mut Data) -> Result<(), EvcError> {
        match self {
            Context::Decoder(ctx) => {
                if let Data::Packet(pkt) = data {
                    ctx.0.push_pkt(pkt)
                } else {
                    Err(EvcError::EVC_ERR_EMPTY_PACKET)
                }
            }
            Context::Encoder(ctx) => {
                if let Data::Frame(frm) = data {
                    ctx.0.push_frm(frm)
                } else {
                    Err(EvcError::EVC_ERR_EMPTY_FRAME)
                }
            }
            Context::Invalid(_) => Err(EvcError::EVC_ERR_UNSUPPORTED),
        }
    }

    pub fn pull(&mut self, data: &mut Data) -> Result<Option<EvcStat>, EvcError> {
        *data = Data::Empty;

        match self {
            Context::Decoder(ctx) => {
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
                    match ctx.0.pull_frm() {
                        Ok(frame) => *data = Data::RefFrame(frame),
                        Err(err) => {
                            if err != EvcError::EVC_OK_OUTPUT_NOT_AVAILABLE {
                                return Err(err);
                            }
                        }
                    }
                }

                Ok(stat)
            }
            Context::Encoder(ctx) => {
                let mut stat = None;
                let mut pull_pkt = false;
                match ctx.0.encode_frm() {
                    Ok(st) => {
                        pull_pkt = true;
                        stat = Some(st);
                    }
                    Err(err) => {
                        if err != EvcError::EVC_OK_OUTPUT_NOT_AVAILABLE {
                            return Err(err);
                        }
                    }
                }

                if pull_pkt {
                    let packet = ctx.0.pull_pkt()?;
                    *data = Data::RefPacket(packet);
                }

                Ok(stat)
            }
            Context::Invalid(_) => Err(EvcError::EVC_ERR_UNSUPPORTED),
        }
    }
}
