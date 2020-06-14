#![allow(warnings)]
#![allow(dead_code)]

mod demuxer;
mod muxer;

use clap::{App, AppSettings, Arg};
use revc::api::*;

use std::io;

struct CLISettings {
    pub demuxer: Box<dyn demuxer::Demuxer>,
    pub muxer: Box<dyn muxer::Muxer>,
    pub frames: usize,
    pub verbose: bool,
    pub threads: usize,
}

fn parse_cli() -> CLISettings {
    let mut app = App::new("revcd")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Rust EVC Decoder")
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
                .long("threads")
                .takes_value(true)
                .default_value("1"),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("file name of input bitstream")
                .short("i")
                .long("input")
                .required_unless("FULLHELP")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .help("file name of decoded output")
                .short("o")
                .long("output")
                .required_unless("FULLHELP")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("FRAMES")
                .help("maximum number of frames to be decoded")
                .short("f")
                .long("frames")
                .takes_value(true)
                .default_value("0"),
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
    }
}

#[derive(PartialEq)]
enum EvcdState {
    STATE_DECODING,
    STATE_PULLING,
    STATE_BUMPING,
}

fn print_stat(stat: EvcdStat, bs_cnt: usize) {
    eprint!("[{:4}] NALU --> ", bs_cnt);
    if stat.nalu_type < NaluType::EVC_SPS_NUT {
        eprint!("{}-slice", stat.stype);

        eprint!(" ({} bytes", stat.read);
        eprint!(", poc={}, tid={}, ", stat.poc, stat.tid);

        for i in 0..2 {
            eprint!("[L{} ", i);
            for j in 0..stat.refpic_num[i] as usize {
                eprint!("{} ", stat.refpic[i][j]);
            }
            eprint!("] ");
        }
    } else if stat.nalu_type == NaluType::EVC_SPS_NUT {
        eprint!("Sequence Parameter Set ({} bytes)", stat.read);
    } else if stat.nalu_type == NaluType::EVC_PPS_NUT {
        eprint!("Picture Parameter Set ({} bytes)", stat.read);
    } else if stat.nalu_type == NaluType::EVC_APS_NUT {
        eprint!("Adaptation Parameter Set ({} bytes)", stat.read);
    } else if stat.nalu_type == NaluType::EVC_SEI_NUT {
        eprint!("SEI message: ");
        if stat.ret == EVC_OK {
            eprint!("MD5 check OK");
        } else if stat.ret == EVC_ERR_BAD_CRC {
            eprint!("MD5 check mismatch!");
        } else if stat.ret == EVC_WARN_CRC_IGNORED {
            eprint!("MD5 check ignored!");
        }
    }
    eprint!("\n");
}

fn print_summary(w: usize, h: usize, bs_cnt: usize, pic_cnt: usize, clk_tot: usize) {
    eprint!(
        "=======================================================================================\n"
    );
    eprint!("Resolution                        = {} x {}\n", w, h);
    eprint!("Processed NALUs                   = {}\n", bs_cnt);
    eprint!("Decoded frame count               = {}\n", pic_cnt);
    if pic_cnt > 0 {
        eprint!("total decoding time               = {} msec,", clk_tot);
        eprint!(" {:.3} sec\n", clk_tot as f32 / 1000.0);

        eprint!(
            "Average decoding time for a frame = {} msec\n",
            clk_tot / pic_cnt
        );
        eprint!(
            "Average decoding speed            = {:.3} frames/sec\n",
            pic_cnt as f32 * 1000.0 / (clk_tot as f32)
        );
    }
    eprint!(
        "=======================================================================================\n"
    );
}

fn main() -> io::Result<()> {
    let mut cli = parse_cli();
    let cfg = Config {
        threads: cli.threads,
        ..Default::default()
    };

    let mut ctx: Context<u8> = Context::new(&cfg);

    let mut pic_cnt: usize = 0;
    let mut clk_tot = 1;
    let mut bs_cnt = 0;
    let mut w = 0;
    let mut h = 0;

    let mut state = EvcdState::STATE_DECODING;

    loop {
        if (state == EvcdState::STATE_DECODING) {
            if cli.frames != 0 && pic_cnt == cli.frames {
                if cli.verbose {
                    eprint!("bumping process starting...\n");
                }
                state = EvcdState::STATE_BUMPING;
                continue;
            } else {
                match cli.demuxer.read() {
                    Ok(pkt) => {
                        let ret = ctx.decode(&mut Some(pkt));
                        match ret {
                            Ok(stat) => {
                                if stat.fnum >= 0 {
                                    state = EvcdState::STATE_PULLING;
                                }
                                if (cli.verbose) {
                                    print_stat(stat, bs_cnt);
                                }
                                bs_cnt += 1;
                            }
                            Err(err) => {
                                eprint!("Decoding error = {:?}\n", err);
                                break;
                            }
                        }
                    }
                    _ => {
                        if cli.verbose {
                            eprint!("bumping process starting...\n");
                        }
                        state = EvcdState::STATE_BUMPING;
                        continue;
                    }
                };
            }
        }

        if (state != EvcdState::STATE_DECODING) {
            let ret = ctx.pull();
            match ret {
                Ok(_) => {}
                Err(err) => {
                    if (err == EvcError::EVC_ERR_UNEXPECTED) {
                        if cli.verbose {
                            eprint!("bumping process completed\n");
                        }
                    } else {
                        eprint!("failed to pull the decoded image\n");
                    }
                    break;
                }
            }

            // after pulling, reset state to decoding mode
            if (state == EvcdState::STATE_PULLING) {
                state = EvcdState::STATE_DECODING;
            }
        }
    }

    if cli.verbose {
        print_summary(w, h, bs_cnt, pic_cnt, clk_tot);
    }

    Ok(())
}
