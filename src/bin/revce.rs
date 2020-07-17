#![allow(warnings)]
#![allow(dead_code)]

mod demuxer;
mod muxer;

use clap::{App, AppSettings, Arg};

use std::io;
use std::time::Instant;

struct CLISettings {
    pub demuxer: Box<dyn demuxer::Demuxer>,
    pub muxer: Box<dyn muxer::Muxer>,
    pub frames: usize,
    pub verbose: bool,
    pub threads: usize,
    pub bitdepth: u8,
}

fn parse_cli() -> CLISettings {
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
            Arg::with_name("BITDEPTH")
                .help("output bitdepth (8, 10)")
                .short("b")
                .long("bitdepth")
                .takes_value(true)
                .default_value("8"),
        )
        .arg(
            Arg::with_name("QP")
                .help("QP value (0~51)")
                .short("q")
                .long("qp")
                .takes_value(true),
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
            Arg::with_name("HZ")
                .help("frame rate (Hz)")
                .short("z")
                .long("hz")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("IPERIOD")
                .help("I-picture period")
                .short("p")
                .long("iperiod")
                .takes_value(true),
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
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ENABLE_CIP")
                .help("enable constrained intra pred (CIP)")
                .long("enable_cip"),
        )
        .arg(
            Arg::with_name("DBF")
                .help("Deblocking filter on/off flag")
                .long("dbf")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("NUM_SLICES_IN_PIC")
                .help("Number of slices in the pic")
                .long("num_slices_in_pic")
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

    CLISettings {
        demuxer: demuxer::new(matches.value_of("INPUT").unwrap()),
        muxer: muxer::new(matches.value_of("OUTPUT").unwrap()),
        frames: matches.value_of("FRAMES").unwrap().parse().unwrap(),
        verbose: matches.is_present("VERBOSE"),
        threads: matches
            .value_of("THREADS")
            .map(|v| v.parse().expect("Threads must be an integer"))
            .unwrap(),
        bitdepth: matches
            .value_of("BITDEPTH")
            .map(|v| v.parse().expect("Bitdepth must be an integer"))
            .unwrap(),
    }
}

#[derive(PartialEq)]
enum EvceState {
    STATE_ENCODING,
    STATE_BUMPING,
    STATE_SKIPPING,
}

fn main() {
    let mut cli = parse_cli();
}
