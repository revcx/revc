#![allow(warnings)]
#![allow(dead_code)]

#[macro_use]
extern crate log;

mod io;

use clap::{App, AppSettings, Arg, ArgMatches};

use std::fs::File;
use std::io::prelude::*;
use std::time::Instant;

use io::demuxer::VideoInfo;
use io::*;
use revc::api::*;

struct CLISettings {
    demuxer: Box<dyn demuxer::Demuxer>,
    muxer: Box<dyn muxer::Muxer>,
    rec: Option<Box<dyn muxer::Muxer>>,
    enc: EncoderConfig,
    frames: usize,
    skip: usize,
    verbose: bool,
    threads: usize,
    bitdepth: u8,
}

pub trait MatchGet {
    fn value_of_int(&self, name: &str) -> Option<std::io::Result<i32>>;
}

impl MatchGet for ArgMatches<'_> {
    fn value_of_int(&self, name: &str) -> Option<std::io::Result<i32>> {
        self.value_of(name).map(|v| {
            v.parse().map_err(|e: std::num::ParseIntError| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
            })
        })
    }
}

fn parse_config(matches: &ArgMatches<'_>) -> std::io::Result<EncoderConfig> {
    let maybe_quantizer = matches.value_of_int("QP");
    let maybe_bitrate = matches.value_of_int("BITRATE");
    let quantizer = maybe_quantizer.unwrap_or_else(|| {
        if maybe_bitrate.is_some() {
            // If a bitrate is specified, the quantizer is the maximum allowed (e.g.,
            //  the minimum quality allowed), which by default should be
            //  unconstrained.
            Ok(51)
        } else {
            Ok(27)
        }
    })? as usize;
    let bitrate: i32 = maybe_bitrate.unwrap_or(Ok(0))?;

    if quantizer == 0 {
        unimplemented!("Lossless encoding not yet implemented");
    } else if quantizer > 51 {
        panic!("Quantizer must be between 0-51");
    }

    let mut max_interval: usize = matches
        .value_of("KEYFRAME_INTERVAL")
        .unwrap()
        .parse()
        .unwrap();
    max_interval = if max_interval == 0 {
        MAX_MAX_KEY_FRAME_INTERVAL
    } else {
        max_interval
    };
    let mut min_interval: usize = matches
        .value_of("MIN_KEYFRAME_INTERVAL")
        .unwrap()
        .parse()
        .unwrap();

    if matches.occurrences_of("MIN_KEYFRAME_INTERVAL") == 0 {
        min_interval = min_interval.min(max_interval);
    }

    let mut cfg = EncoderConfig::default();

    cfg.width = matches.value_of("WIDTH").unwrap_or("0").parse().unwrap();
    cfg.height = matches.value_of("HEIGHT").unwrap_or("0").parse().unwrap();
    if let Some(frame_rate) = matches.value_of("FRAME_RATE") {
        cfg.time_base = Rational::new(
            matches.value_of("TIME_SCALE").unwrap().parse().unwrap(),
            frame_rate.parse().unwrap(),
        );
    }
    cfg.bit_depth = matches
        .value_of("BIT_DEPTH")
        .unwrap_or("8")
        .parse()
        .unwrap();
    cfg.chroma_sampling = ChromaSampling::Cs420;
    cfg.min_key_frame_interval = min_interval;
    // Map an input value of 0 to an infinite interval
    cfg.max_key_frame_interval = max_interval;

    cfg.qp = quantizer as u8;
    cfg.max_qp = matches.value_of("MAXQP").unwrap_or("0").parse().unwrap();
    cfg.min_qp = matches.value_of("MINQP").unwrap_or("0").parse().unwrap();
    cfg.bitrate = bitrate.checked_mul(1000).expect("Bitrate too high");

    cfg.cb_qp_offset = matches
        .value_of("CB_QP_OFFSET")
        .unwrap_or("0")
        .parse()
        .unwrap();
    cfg.cr_qp_offset = matches
        .value_of("CR_QP_OFFSET")
        .unwrap_or("0")
        .parse()
        .unwrap();
    cfg.cu_qp_delta_area = matches
        .value_of("CU_QP_DELTA_AREA")
        .unwrap_or("6")
        .parse()
        .unwrap();
    cfg.max_b_frames = matches.value_of("USE_DQP").unwrap_or("0").parse().unwrap();
    cfg.ref_pic_gap_length = matches
        .value_of("REF_PIC_GAP_LENGTH")
        .unwrap_or("0")
        .parse()
        .unwrap();
    if cfg.max_b_frames == 0 && cfg.ref_pic_gap_length == 0 {
        cfg.ref_pic_gap_length = 1;
    }
    cfg.level = matches.value_of("LEVEL").unwrap_or("51").parse().unwrap();
    cfg.closed_gop = matches.is_present("CLOSED_GOP");
    cfg.disable_hgop = matches.is_present("DISABLE_HGOP");
    cfg.enable_cip = matches.is_present("ENABLE_CIP");
    cfg.disable_dbf = matches.is_present("DISABLE_DBF");
    cfg.num_slices_in_pic = matches
        .value_of("NUM_SLICES_IN_PIC")
        .unwrap_or("1")
        .parse()
        .unwrap();
    cfg.inter_slice_type = matches
        .value_of("INTER_SLICE_TYPE")
        .unwrap_or("0")
        .parse()
        .unwrap();

    Ok(cfg)
}

fn parse_cli() -> std::io::Result<CLISettings> {
    let mut app = App::new("revce")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Rust EVC Encoder")
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::SubcommandsNegateReqs)
        .arg(
            Arg::with_name("FULLHELP")
                .help("Prints more detailed help information")
                .long("fullhelp"),
        )
        // THREADS
        .arg(
            Arg::with_name("THREADS")
                .help("Set the threadpool size")
                .short("t")
                .long("threads")
                .takes_value(true)
                .default_value("1"),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("file name of input video")
                .short("i")
                .long("input")
                .required_unless("FULLHELP")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .help("file name of encoded output")
                .short("o")
                .long("output")
                .required_unless("FULLHELP")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("RECON")
                .help("file name of reconstructed video")
                .short("r")
                .long("recon")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("WIDTH")
                .help("pixel width of input video")
                .short("w")
                .long("width")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("HEIGHT")
                .help("pixel height of input video")
                .short("h")
                .long("height")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("BIT_DEPTH")
                .help("output bit depth (8, 10)")
                .short("d")
                .long("bit-depth")
                .takes_value(true)
                .default_value("8"),
        )
        .arg(
            Arg::with_name("BITRATE")
                .help("Bitrate (kbps)")
                .short("b")
                .long("bitrate")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("MINQP")
                .help("Minimum quantizer (0-51) to use in bitrate mode")
                .long("minqp")
                .takes_value(true)
                .default_value("4"),
        )
        .arg(
            Arg::with_name("MAXQP")
                .help("Maximum quantizer (0-51) to use in bitrate mode")
                .long("maxqp")
                .takes_value(true)
                .default_value("51"),
        )
        .arg(
            Arg::with_name("QP")
                .help("QP value (0-51)")
                .short("q")
                .long("qp")
                .takes_value(true)
                .default_value("27"),
        )
        .arg(
            Arg::with_name("CB_QP_OFFSET")
                .help("cb qp offset")
                .long("cb_qp_offset")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("CR_QP_OFFSET")
                .help("cr qp offset")
                .long("cr_qp_offset")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("USE_DQP")
                .help("use_dqp ({0,..,25})")
                .long("use_dqp")
                .takes_value(true)
                .default_value("0"),
        )
        .arg(
            Arg::with_name("CU_QP_DELTA_AREA")
                .help("cu_qp_delta_area (>= 6)")
                .long("cu_qp_delta_area")
                .takes_value(true)
                .default_value("6"),
        )
        .arg(
            Arg::with_name("FRAME_RATE")
                .help("frame rate (Hz)")
                .short("z")
                .long("frame-rate")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("TIME_SCALE")
                .help(
                    "The time scale associated with the frame rate if provided (ignored otherwise)",
                )
                .long("time-scale")
                .alias("time_scale")
                .default_value("1")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("MIN_KEYFRAME_INTERVAL")
                .help("Minimum interval between keyframes")
                .long("min-keyint")
                .takes_value(true)
                .default_value("12"),
        )
        .arg(
            Arg::with_name("KEYFRAME_INTERVAL")
                .help("Maximum interval between keyframes")
                .short("p")
                .long("keyint")
                .takes_value(true)
                .default_value("0"),
        )
        .arg(
            Arg::with_name("MAX_B_FRAMES")
                .help("Number of maximum B frames (1,3,7,15)")
                .short("g")
                .long("max_b_frames")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("FRAMES")
                .help("maximum number of frames to be encoded")
                .short("f")
                .long("frames")
                .takes_value(true)
                .default_value("0"),
        )
        .arg(
            Arg::with_name("SIGNATURES")
                .help("embed picture signature (HASH) for conformance checking in decoding")
                .short("s")
                .long("signature"),
        )
        .arg(
            Arg::with_name("REF_PIC_GAP_LENGTH")
                .help("reference picture gap length (1,2,4...) only available when -g is 0")
                .long("ref_pic_gap_length")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("CLOSED_GOP")
                .help("use closed GOP structure. if not set, open GOP is used")
                .long("closed_gop"),
        )
        .arg(
            Arg::with_name("DISABLE_HGOP")
                .help("disable hierarchical GOP. if not set, hierarchical GOP is used")
                .long("disable_hgop"),
        )
        .arg(
            Arg::with_name("SKIP")
                .help("Skip n number of frames and encode")
                .long("skip")
                .takes_value(true)
                .default_value("0"),
        )
        .arg(
            Arg::with_name("LEVEL")
                .help("level setting")
                .long("level")
                .takes_value(true)
                .default_value("51"),
        )
        .arg(
            Arg::with_name("ENABLE_CIP")
                .help("enable constrained intra pred (CIP)")
                .long("enable_cip"),
        )
        .arg(
            Arg::with_name("DISABLE_DBF")
                .help("Disable deblocking filter flag")
                .long("disable_dbf"),
        )
        .arg(
            Arg::with_name("NUM_SLICES_IN_PIC")
                .help("Number of slices in the pic")
                .long("num_slices_in_pic")
                .takes_value(true)
                .default_value("1"),
        )
        .arg(
            Arg::with_name("INTER_SLICE_TYPE")
                .help("Inter slice type (0: SLICE_B 1: SLICE_P)")
                .long("inter_slice_type")
                .takes_value(true),
        )
        // DEBUGGING
        .arg(
            Arg::with_name("VERBOSE")
                .help("Verbose logging; outputs info for every frame")
                .long("verbose")
                .short("v"),
        );

    let matches = app.clone().get_matches();

    if matches.is_present("FULLHELP") {
        app.print_long_help().unwrap();
        std::process::exit(0);
    }

    let rec = match matches.value_of("RECON") {
        Some(recon) => Some(muxer::new(recon)?),
        None => None,
    };

    let enc = parse_config(&matches)?;
    let info = Some(VideoInfo {
        width: enc.width,
        height: enc.height,
        bit_depth: enc.bit_depth,
        chroma_sampling: enc.chroma_sampling,
        time_base: enc.time_base,
    });

    Ok(CLISettings {
        demuxer: demuxer::new(matches.value_of("INPUT").unwrap(), info)?,
        muxer: muxer::new(matches.value_of("OUTPUT").unwrap())?,
        rec,
        enc,
        frames: matches.value_of("FRAMES").unwrap().parse().unwrap(),
        skip: matches.value_of("SKIP").unwrap().parse().unwrap(),
        verbose: matches.is_present("VERBOSE"),
        threads: matches
            .value_of("THREADS")
            .map(|v| v.parse().expect("Threads must be an integer"))
            .unwrap(),
        bitdepth: matches
            .value_of("BIT_DEPTH")
            .map(|v| v.parse().expect("Bitdepth must be an integer"))
            .unwrap(),
    })
}

#[derive(PartialEq)]
enum EvceState {
    STATE_ENCODING,
    STATE_BUMPING,
}

fn main() -> std::io::Result<()> {
    let mut cli = parse_cli()?;

    if let Some(video_info) = cli.demuxer.info() {
        cli.enc.width = video_info.width;
        cli.enc.height = video_info.height;
        cli.enc.bit_depth = video_info.bit_depth;
        cli.enc.chroma_sampling = video_info.chroma_sampling;
        cli.enc.time_base = video_info.time_base;
    }

    eprint!("Encoding settings: {:?}\n", cli.enc);

    let cfg = Config {
        threads: cli.threads,
        enc: Some(cli.enc),
    };

    let mut ctx = Context::new(&cfg);
    if let Context::Invalid(err) = &ctx {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, *err));
    }

    for _ in 0..cli.skip {
        match cli.demuxer.read() {
            Ok(f) => f,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Skipped more frames than in the input",
                ))
            }
        };
    }

    let mut clk_tot = 0;
    let mut pic_cnt = 0;

    let mut state = EvceState::STATE_ENCODING;

    loop {
        if state == EvceState::STATE_ENCODING {
            if cli.frames != 0 && pic_cnt >= cli.frames {
                if cli.verbose {
                    eprint!("bumping process starting...\n");
                }
                state = EvceState::STATE_BUMPING;
                continue;
            } else {
                match cli.demuxer.read() {
                    Ok(mut data) => {
                        let start = Instant::now();
                        let ret = ctx.push(&mut data);
                        let duration = start.elapsed();
                        clk_tot += duration.as_millis() as usize;

                        match ret {
                            Ok(_) => pic_cnt += 1,
                            Err(err) => {
                                eprint!("Encoding error = {:?}\n", err);
                                break;
                            }
                        }
                    }
                    _ => {
                        if cli.verbose {
                            eprint!("bumping process starting...\n");
                        }
                        state = EvceState::STATE_BUMPING;
                        continue;
                    }
                }
            }
        }

        let mut data = Data::Empty;
        let start = Instant::now();
        let ret = ctx.pull(&mut data);
        let duration = start.elapsed();
        clk_tot += duration.as_millis() as usize;

        match ret {
            Ok(_) => cli.muxer.write(
                data,
                cli.bitdepth,
                Rational::new(cli.enc.time_base.den, cli.enc.time_base.num),
            )?,
            Err(err) => {
                if err == EvcError::EVC_OK_NO_MORE_OUTPUT {
                    if cli.verbose {
                        eprint!("bumping process completed\n");
                    }
                } else {
                    eprint!("failed to pull the encoded packet\n");
                }
                break;
            }
        }
    }

    Ok(())
}
