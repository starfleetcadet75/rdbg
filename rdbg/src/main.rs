#![feature(plugin, use_extern_macros)]
#![plugin(tarpc_plugins)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate tarpc;
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

mod rpc;
mod interpreter;

use interpreter::Interpreter;

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
                    .requires("exec-file"),
                Arg::from_usage("--connect 'Connect to a headless debugger server.'"),
                Arg::from_usage("--headless 'Start a headless debugger.'"),
            ],
        )
        .arg(Arg::with_name("v").short("v").multiple(true).help(
            "Sets the log level verbosity",
        ))
        .get_matches();

    setup_logger(&args);

    // TODO: Accept args to set host:port values
    let address = "127.0.0.1:9000".parse().unwrap();
    let mut client = None;

    if args.is_present("headless") {
        // Start a headless debugger
        rpc::listen(address);
    } else if args.is_present("connect") {
        // Connect to a headless debugger
        client = rpc::connect(address);
    } else {
        client = rpc::run();
    }

    let interpreter = Interpreter::new(client.unwrap());

    let profile = setup_profile(&args);
    if let Some(profile) = profile {
        interpreter.load_profile_command(profile);
    }

    if let Err(error) = interpreter.read_line() {
        error!("Application Error: {}", error);
        process::exit(1);
    }
}
