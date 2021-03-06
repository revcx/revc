#![allow(warnings)]
#![allow(dead_code)]

mod io;

use clap::{App, AppSettings, Arg};
use io::*;
use revc::api::*;

use std::time::Instant;

struct CLISettings {
    demuxer: Box<dyn demuxer::Demuxer>,
    muxer: Box<dyn muxer::Muxer>,
    frames: usize,
    verbose: bool,
    threads: usize,
    bitdepth: u8,
}

fn parse_cli() -> std::io::Result<CLISettings> {
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
                .short("t")
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
            Arg::with_name("BITDEPTH")
                .help("output bitdepth (8(default), 10)")
                .short("b")
                .long("bitdepth")
                .takes_value(true)
                .default_value("8"),
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

    Ok(CLISettings {
        demuxer: demuxer::new(matches.value_of("INPUT").unwrap(), None)?,
        muxer: muxer::new(matches.value_of("OUTPUT").unwrap())?,
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
    })
}

fn print_stat(stat: &EvcStat, bs_cnt: usize) {
    eprint!("[{:>7}] NALU --> ", bs_cnt);
    if stat.nalu_type < NaluType::EVC_SPS_NUT {
        eprint!("{}-slice", stat.stype);

        eprint!(" ({:>10} bytes", stat.bytes);
        eprint!(", poc={:>7}, tid={:>2}), ", stat.poc, stat.tid);

        for i in 0..2 {
            eprint!("[L{} ", i);
            for j in 0..stat.refpic_num[i] as usize {
                eprint!("{} ", stat.refpic[i][j]);
            }
            eprint!("] ");
        }
    } else if stat.nalu_type == NaluType::EVC_SPS_NUT {
        eprint!("Sequence Parameter Set ({} bytes)", stat.bytes);
    } else if stat.nalu_type == NaluType::EVC_PPS_NUT {
        eprint!("Picture Parameter Set ({} bytes)", stat.bytes);
    } else if stat.nalu_type == NaluType::EVC_APS_NUT {
        eprint!("Adaptation Parameter Set ({} bytes)", stat.bytes);
    } else if stat.nalu_type == NaluType::EVC_SEI_NUT {
        eprint!("SEI message ({} bytes)", stat.bytes);
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
        eprint!("Total decoding time               = {} msec,", clk_tot);
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

#[derive(PartialEq)]
enum EvcdState {
    STATE_DECODING,
    STATE_BUMPING,
}

fn main() -> std::io::Result<()> {
    let mut cli = parse_cli()?;
    let cfg = Config {
        threads: cli.threads,
        enc: None,
    };

    let mut ctx = Context::new(&cfg);

    let mut pic_ocnt: usize = 0;
    let mut clk_tot = 0;
    let mut bs_cnt = 0;
    let mut w = 0;
    let mut h = 0;

    let mut state = EvcdState::STATE_DECODING;

    loop {
        if cli.frames != 0 && pic_ocnt == cli.frames {
            break;
        }

        if state == EvcdState::STATE_DECODING {
            match cli.demuxer.read() {
                Ok(mut data) => {
                    let start = Instant::now();
                    let ret = ctx.push(&mut data);
                    let duration = start.elapsed();
                    clk_tot += duration.as_millis() as usize;

                    match ret {
                        Ok(_) => {}
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

        let mut data = Data::Empty;
        let start = Instant::now();
        let ret = ctx.pull(&mut data);
        let duration = start.elapsed();
        clk_tot += duration.as_millis() as usize;

        match ret {
            Ok(st) => {
                if let Some(stat) = st {
                    if cli.verbose {
                        print_stat(&stat, bs_cnt);
                    }
                    bs_cnt += 1;
                }

                let has_frame = if let Data::RefFrame(frame) = &data {
                    let f = frame.borrow();
                    w = f.planes[0].cfg.width;
                    h = f.planes[0].cfg.height;
                    true
                } else {
                    false
                };

                if has_frame {
                    cli.muxer.write(data, cli.bitdepth, Rational::new(30, 1))?;
                    pic_ocnt += 1;
                }
            }
            Err(err) => {
                if err == EvcError::EVC_OK_NO_MORE_OUTPUT {
                    if cli.verbose {
                        eprint!("bumping process completed\n");
                    }
                } else {
                    eprint!("failed to pull the decoded frame\n");
                }
                break;
            }
        }
    }

    if cli.verbose {
        print_summary(w, h, bs_cnt, pic_ocnt, clk_tot);
    }

    Ok(())
}
