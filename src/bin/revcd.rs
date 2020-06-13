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

fn main() -> io::Result<()> {
    let mut cli = parse_cli();
    let cfg = Config {
        threads: cli.threads,
        ..Default::default()
    };

    let mut ctx: Context<u8> = Context::new(&cfg);
    let mut pic_cnt: usize = 0;
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
                        if let Ok(stat) = ret {
                            if stat.fnum >= 0 {
                                state = EvcdState::STATE_PULLING;
                            }
                        //print_stat(stat);
                        } else {
                            break;
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
                    if (err == EvcStatus::EVC_ERR_UNEXPECTED) {
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

    Ok(())
}
