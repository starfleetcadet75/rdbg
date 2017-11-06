#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate fnv;
extern crate simplelog;
extern crate rustyline;
extern crate rdbg_core;

use clap::{App, Arg, ArgMatches};
use simplelog::{CombinedLogger, Config, LogLevelFilter, TermLogger, WriteLogger};

use std::fs::File;
use std::path::Path;
use std::process;

use rdbg_core::core::profile::Profile;

mod interpreter;

use interpreter::Interpreter;

/// Inits the logger using any args given.
fn setup_logger(args: &ArgMatches) {
    let log_level = match args.occurrences_of("v") {
        0 => LogLevelFilter::Error,
        1 => LogLevelFilter::Warn,
        2 => LogLevelFilter::Info,
        3 | _ => LogLevelFilter::Debug,
    };

    // Writes all logging to a file and prints logging
    // at the given level to the terminal output.
    CombinedLogger::init(vec![
        TermLogger::new(log_level, Config::default()).unwrap(),
        WriteLogger::new(
            LogLevelFilter::Debug,
            Config::default(),
            File::create("/tmp/debugger.log").unwrap()
        ),
    ]).unwrap();
}

/// Returns a `Profile` if the args were given for creating one.
fn setup_profile(args: &ArgMatches) -> Option<Profile> {
    if args.is_present("exec-file") {
        let path = Path::new(args.value_of("exec-file").unwrap());
        Some(Profile::new(&path))
    } else {
        None
    }
}

fn main() {
    let args = App::new("rdbg")
        .version(crate_version!())
        .author(crate_authors!())
        .about("A debugger written in Rust")
        .args(
            &[
                Arg::from_usage("[exec-file] 'Use EXECFILE as the executable to debug.'"),
                Arg::from_usage("--args [val]... 'Arguments to pass to the inferior.'")  // TODO: get multiple args
                    .min_values(1)
                    .requires("exec-file")
            ],
        )
        .arg(Arg::with_name("v").short("v").multiple(true).help(
            "Sets the log level verbosity",
        ))
        .get_matches();

    setup_logger(&args);

    let mut interpreter = Interpreter::new();
    let profile = setup_profile(&args);

    // If a profile was created, load it automatically at startup
    if let Some(profile) = profile {
        interpreter.load_profile_command(profile);
    }

    // Run the interpreter until an error occurs
    if let Err(error) = interpreter.read_line() {
        error!("Application Error: {}", error);
        process::exit(1);
    }
}
