#![allow(warnings)]
#![allow(dead_code)]

use clap::{App, AppSettings, Arg};

use std::fs::File;
use std::io;
use std::io::prelude::*;

struct Options {
    input: Box<dyn Read>,
    output: Box<dyn Write>,
    frames: usize,
    signature: bool,
    verbose: u8,
    output_bit_depth: usize,
}

fn parse_cli() -> Options {
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
                .help("Maximum number of frames to decode")
                .short("f")
                .long("frames")
                .takes_value(true)
                .default_value("0"),
        )
        .arg(
            Arg::with_name("SIGNATURE")
                .help("conformance check using picture signature (HASH)")
                .short("s")
                .long("signature")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("VERBOSE")
                .help("verbose level\n\t 0: no message\n\t 1: frame-level messages (default)\n\t 2: all messages\n")
                .short("v")
                .long("verbose")
                .takes_value(true)
                .default_value("1"),
        )
        .arg(
            Arg::with_name("OUTPUT_BIT_DEPTH")
                .help("output bitdepth (8, 10)")
                .long("output_bit_depth")
                .takes_value(true)
                .default_value("8")
        );

    let matches = app.clone().get_matches();

    if matches.is_present("FULLHELP") {
        app.print_long_help().unwrap();
        std::process::exit(0);
    }

    Options {
        input: match matches.value_of("INPUT").unwrap() {
            "-" => Box::new(io::stdin()) as Box<dyn Read>,
            f => Box::new(File::open(&f).unwrap()) as Box<dyn Read>,
        },
        output: match matches.value_of("INPUT").unwrap() {
            "-" => Box::new(io::stdout()) as Box<dyn Write>,
            f => Box::new(File::create(&f).unwrap()) as Box<dyn Write>,
        },
        frames: matches.value_of("FRAMES").unwrap().parse().unwrap(),
        signature: matches.is_present("SIGNATURE"),
        verbose: matches.value_of("VERBOSE").unwrap().parse().unwrap(),
        output_bit_depth: matches
            .value_of("OUTPUT_BIT_DEPTH")
            .unwrap()
            .parse()
            .unwrap(),
    }
}

fn main() {
    let mut cli = parse_cli();

    println!("Hello, revcd!");
}
