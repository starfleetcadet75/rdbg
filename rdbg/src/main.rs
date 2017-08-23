#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate fnv;
extern crate simplelog;
extern crate rustyline;
extern crate rdbg_core;

mod interpreter;

use clap::{App, Arg, ArgMatches};
use simplelog::{CombinedLogger, Config, LogLevelFilter, TermLogger, WriteLogger};

use std::fs::File;
use std::process;

use interpreter::command_interpreter;

fn setup_logger(args: &ArgMatches) {
    let log_level = match args.occurrences_of("v") {
        0 => LogLevelFilter::Error,
        1 => LogLevelFilter::Warn,
        2 => LogLevelFilter::Info,
        3 | _ => LogLevelFilter::Debug,
    };

    CombinedLogger::init(vec![
        TermLogger::new(log_level, Config::default()).unwrap(),
        WriteLogger::new(
            LogLevelFilter::Debug,
            Config::default(),
            File::create("/tmp/debugger.log").unwrap()
        ),
    ]).unwrap();
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
            ],
        )
        .arg(Arg::with_name("v").short("v").multiple(true).help(
            "Sets the level of verbosity",
        ))
        .get_matches();

    setup_logger(&args);
    let mut interpreter = command_interpreter::CommandInterpreter::new();
    match args.value_of("exec-file") {
        Some(path) => {
            let mut cmd_args = Vec::new();
            cmd_args.push(path);

            if args.value_of("args").is_some() {
                cmd_args.push(args.value_of("args").unwrap());
            }

            interpreter.set_program(cmd_args.as_slice()).expect(
                "Problem loading program",
            )
        }
        None => {}
    }

    if let Err(error) = interpreter.read_line() {
        error!("Application Error: {}", error);
        process::exit(1);
    }
}
