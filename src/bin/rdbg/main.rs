extern crate fnv;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate rustyline;
extern crate rdbg_core;

mod interpreter;

use clap::{Arg, App};
use simplelog::{Config, TermLogger, WriteLogger, CombinedLogger, LogLevelFilter};
use rdbg_core::core::debugger;
use interpreter::command_interpreter;

use std::process;
use std::path::Path;
use std::fs::File;

fn main() {
    let args = App::new("rdbg")
        .version(crate_version!())
        .author(crate_authors!())
        .about("A debugger written in Rust")
        .arg(Arg::with_name("PROGRAM")
             .help("The program to debug")
             .required(true)
             .index(1))
        .arg(Arg::with_name("v")
             .short("v")
             .multiple(true)
             .help("Sets the level of verbosity"))
        .get_matches();

    let log_level = match args.occurrences_of("v") {
        0 => LogLevelFilter::Error,
        1 => LogLevelFilter::Warn,
        2 => LogLevelFilter::Info,
        3 | _ => LogLevelFilter::Debug,
    };

    CombinedLogger::init(
        vec![
        TermLogger::new(log_level, Config::default()).unwrap(),
        WriteLogger::new(LogLevelFilter::Debug, Config::default(), File::create("/tmp/debugger.log").unwrap()),
        ]
    ).unwrap();

    let program = &String::from(args.value_of("PROGRAM").unwrap());
    let program = Path::new(program);

    let mut dbg = debugger::Debugger::new();
    if let Err(error) = dbg.execute_target(program, &[]) {
        error!("Application Error: {}", error);
        process::exit(1);
    }
    // TODO: Handle attaching to a process

    // start interpreter after starting the trace
    let mut interpreter = command_interpreter::CommandInterpreter::new(dbg);
    if let Err(error) = interpreter.read_line() {
        error!("Application Error: {}", error);
        process::exit(1);
    }
}

