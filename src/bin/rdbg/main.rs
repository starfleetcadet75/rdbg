#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
extern crate error_chain;
extern crate rdbg_core;
extern crate rustyline;
extern crate simplelog;

use clap::{App, Arg, ArgMatches};
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, WriteLogger};

use std::fs::File;

use rdbg_core::util::errors::*;

mod interpreter;
use interpreter::Interpreter;

/// Inits the logger using any args given.
fn setup_logger(args: &ArgMatches) {
    let log_level = match args.occurrences_of("v") {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 | _ => LevelFilter::Debug,
    };

    // Writes all logging to a file and prints logging
    // at the given level to the terminal output.
    CombinedLogger::init(vec![
        TermLogger::new(log_level, Config::default()).unwrap(),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("/tmp/debugger.log").expect("Failed to create log file"),
        ),
    ]).expect("Failed to initialize logging");
}

fn run() -> RdbgResult<()> {
    let args = App::new("rdbg")
        .version(crate_version!())
        .author(crate_authors!())
        .about("A debugger written in Rust")
        .args(&[
            Arg::from_usage("[exec-file] 'Use EXECFILE as the executable to debug.'"),
            Arg::from_usage("--args [val]... 'Arguments to pass to the inferior.'")  // TODO: get multiple args
                    .min_values(1)
                    .requires("exec-file"),
        ])
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the log level verbosity"),
        )
        .get_matches();

    setup_logger(&args);

    // TODO: Print usage when arg not present
    let path = args
        .value_of("exec-file")
        .expect("No exec-file argument given");
    let mut interpreter = Interpreter::new(path.to_string());

    // Run the interpreter until an error occurs
    interpreter.read_line()
}

fn main() {
    if let Err(ref err) = run() {
        error!("{}", err);

        for err in err.iter().skip(1) {
            error!("caused by: {}", err);
        }

        // This is only logged when `RUST_BACKTRACE=1` is set
        if let Some(backtrace) = err.backtrace() {
            error!("backtrace: {:?}", backtrace);
        }
        std::process::exit(1);
    }
}
