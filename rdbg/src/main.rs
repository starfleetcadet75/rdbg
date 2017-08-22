#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate fnv;
extern crate simplelog;
extern crate rustyline;
extern crate rdbg_core;

mod interpreter;

use clap::{App, Arg};
use simplelog::{CombinedLogger, Config, LogLevelFilter, TermLogger, WriteLogger};

use std::fs::File;
use std::process;

use interpreter::command_interpreter;

fn main() {
    let args = App::new("rdbg")
        .version(crate_version!())
        .author(crate_authors!())
        .about("A debugger written in Rust")
        .arg(
            Arg::with_name("PROGRAM")
                .help("The program to debug")
                .index(1),
        )
        .arg(Arg::with_name("v").short("v").multiple(true).help(
            "Sets the level of verbosity",
        ))
        .get_matches();

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

    let mut interpreter = command_interpreter::CommandInterpreter::new();
    match args.value_of("PROGRAM") {
        Some(path) => {
            interpreter.set_program(path).expect(
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
