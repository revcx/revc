use crate::api::*;

// Due to the math in RCState::new() regarding the reservoir frame delay.
pub const MAX_MAX_KEY_FRAME_INTERVAL: u64 = i32::max_value() as u64 / 3;

// Encoder settings which impact the produced bitstream.
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
}
