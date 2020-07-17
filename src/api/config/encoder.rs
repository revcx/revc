use crate::api::*;

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

    // data format and ancillary color information
    // Bit depth.
    pub bit_depth: usize,
    // Chroma subsampling.
    pub chroma_sampling: ChromaSampling,
    // Enable signaling timing info in the bitstream.
    pub enable_timing_info: bool,

    // Still picture mode flag.
    pub still_picture: bool,

    // Flag to force all frames to be error resilient.
    pub error_resilient: bool,

    // Interval between switch frames (0 to disable)
    pub switch_frame_interval: u64,

    // encoder configuration
    // The *minimum* interval between two keyframes
    pub min_key_frame_interval: u64,
    // The *maximum* interval between two keyframes
    pub max_key_frame_interval: u64,
    // The number of temporal units over which to distribute the reservoir
    // usage.
    pub reservoir_frame_delay: Option<i32>,
    // Flag to enable low latency mode.
    //
    // In this mode the frame reordering is disabled.
    pub low_latency: bool,
    // The base quantizer to use.
    pub quantizer: usize,
    // The minimum allowed base quantizer to use in bitrate mode.
    pub min_quantizer: u8,
    // The target bitrate for the bitrate mode.
    pub bitrate: i32,
    // Metric to tune the quality for.
    //pub tune: Tune,

    // Number of frames to read ahead for the RDO lookahead computation.
    pub rdo_lookahead_frames: usize,
    // Settings which affect the enconding speed vs. quality trade-off.
    //pub speed_settings: SpeedSettings,
}
