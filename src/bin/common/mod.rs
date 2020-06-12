use revc::api::*;
use revc::com::frame::*;
use revc::com::util::Pixel;

use std::fmt;
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub struct FrameSummary {
    // Frame size in bytes
    pub width: usize,
    pub height: usize,
    pub pts: u64,
    pub frame_type: FrameType,
}

impl<T: Pixel> From<Frame<T>> for FrameSummary {
    fn from(frame: Frame<T>) -> Self {
        Self {
            width: frame.planes[0].cfg.width,
            height: frame.planes[0].cfg.height,
            pts: frame.pts,
            frame_type: frame.frame_type,
        }
    }
}

impl fmt::Display for FrameSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Frame {} - {} - {}x{}",
            self.pts, self.frame_type, self.width, self.height,
        )
    }
}

#[derive(Debug, Clone)]
pub struct ProgressInfo {
    // Frame rate of the video
    frame_rate: Rational,
    // The length of the whole video, in frames, if known
    total_frames: Option<usize>,
    // The time the encode was started
    time_started: Instant,
    // List of frames encoded so far
    frame_info: Vec<FrameSummary>,
}

impl ProgressInfo {
    pub fn new(frame_rate: Rational, total_frames: Option<usize>) -> Self {
        Self {
            frame_rate,
            total_frames,
            time_started: Instant::now(),
            frame_info: Vec::with_capacity(total_frames.unwrap_or_default()),
        }
    }

    pub fn add_frame(&mut self, frame: FrameSummary) {
        //self.encoded_size += frame.size;
        self.frame_info.push(frame);
    }

    pub fn frames_decoded(&self) -> usize {
        self.frame_info.len()
    }

    pub fn decoding_fps(&self) -> f64 {
        let duration = Instant::now().duration_since(self.time_started);
        self.frame_info.len() as f64
            / (duration.as_secs() as f64 + duration.subsec_millis() as f64 / 1000f64)
    }

    /*pub fn video_fps(&self) -> f64 {
        self.frame_rate.num as f64 / self.frame_rate.den as f64
    }*/

    // Returns the bitrate of the frames so far, in bits/second
    /*pub fn bitrate(&self) -> usize {
        let bits = self.encoded_size * 8;
        let seconds = self.frame_info.len() as f64 / self.video_fps();
        (bits as f64 / seconds) as usize
    }*/

    // Estimates the final filesize in bytes, if the number of frames is known
    /*pub fn estimated_size(&self) -> usize {
        self.total_frames
            .map(|frames| self.encoded_size * frames / self.frames_encoded())
            .unwrap_or_default()
    }*/

    // Estimates the remaining encoding time in seconds, if the number of frames is known
    pub fn estimated_time(&self) -> f64 {
        self.total_frames
            .map(|frames| (frames - self.frames_decoded()) as f64 / self.decoding_fps())
            .unwrap_or_default()
    }

    // Number of frames of given type which appear in the video
    pub fn get_frame_type_count(&self, frame_type: FrameType) -> usize {
        self.frame_info
            .iter()
            .filter(|frame| frame.frame_type == frame_type)
            .count()
    }

    // Size in bytes of all frames of given frame type
    /*pub fn get_frame_type_size(&self, frame_type: FrameType) -> usize {
        self.frame_info.iter()
            .filter(|frame| frame.frame_type == frame_type)
            .map(|frame| frame.size)
            .sum()
    }*/

    pub fn print_summary(&self) -> String {
        let (key, inter) = (
            self.get_frame_type_count(FrameType::KEY),
            self.get_frame_type_count(FrameType::INTER),
        );
        format!("Key: {:>6}, Inter: {:>6}", key, inter,)
    }
}

impl fmt::Display for ProgressInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(total_frames) = self.total_frames {
            write!(
                f,
                "decoded {}/{} frames, {:.3} fps, est. time: {:.0} s",
                self.frames_decoded(),
                total_frames,
                self.decoding_fps(),
                self.estimated_time()
            )
        } else {
            write!(
                f,
                "decoded {} frames, {:.3} fps",
                self.frames_decoded(),
                self.decoding_fps()
            )
        }
    }
}
