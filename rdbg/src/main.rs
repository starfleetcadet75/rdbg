use std::borrow::Cow::{self, Borrowed, Owned};
use std::fs::File;
use std::{env, fs};

use clap::{clap_app, crate_authors, crate_description, crate_name, crate_version, ArgMatches};
use log::error;
use log::warn;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::config::OutputStreamType;
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::{Cmd, CompletionType, Config, Context, EditMode, Editor, KeyPress};
use rustyline_derive::Helper;
use simplelog::{CombinedLogger, LevelFilter, TermLogger, TerminalMode, WriteLogger};

use rdbg_core::core::debugger::Debugger;
use rdbg_core::util::errors::RdbgResult;

#[derive(Helper)]
struct RdbgHelper {
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    hinter: HistoryHinter,
    colored_prompt: String,
}

impl Completer for RdbgHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for RdbgHelper {
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for RdbgHelper {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

/// Initializes the logger using any args given.
fn setup_logger(args: &ArgMatches) {
    let log_level = match args.occurrences_of("debug") {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 | _ => LevelFilter::Debug,
    };

    // Writes all logging to a file and prints logging at the given level to the terminal output
    let log_file = env::temp_dir().join("rdbg.log");
    CombinedLogger::init(vec![
        TermLogger::new(log_level, simplelog::Config::default(), TerminalMode::Mixed).unwrap(),
        WriteLogger::new(
            LevelFilter::Debug,
            simplelog::Config::default(),
            File::create(log_file).expect("Failed to create log file"),
        ),
    ])
    .expect("Failed to initialize logging");
}

fn run() -> RdbgResult<()> {
    let args = clap_app!(rdbg =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg program: "Executable or core dump file to load.")
        (@arg debug: -d ... "Sets the level of debugging information.")
        (@arg config: -c --config +takes_value "Path to the config file.")
        (@arg pid: -p --pid +takes_value "Attach to running process PID.")
    )
    .get_matches();
    setup_logger(&args);

    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Vi)
        .output_stream(OutputStreamType::Stdout)
        .build();

    let h = RdbgHelper {
        completer: FilenameCompleter::new(),
        highlighter: MatchingBracketHighlighter::new(),
        hinter: HistoryHinter {},
        colored_prompt: "".to_owned(),
    };

    let mut rl = Editor::with_config(config);
    rl.set_helper(Some(h));
    rl.bind_sequence(KeyPress::Meta('N'), Cmd::HistorySearchForward);
    rl.bind_sequence(KeyPress::Meta('P'), Cmd::HistorySearchBackward);

    // Load from the user's rdbg config files
    let config_dir = dirs::config_dir()
        .expect("No config directory found")
        .join("rdbg");
    fs::create_dir(&config_dir).ok();

    let history_file = config_dir.join("history");
    rl.load_history(&history_file).ok();

    println!("{} v{}", crate_name!(), crate_version!());
    println!("Press Ctrl-D or enter \"quit\" to exit.\n");

    let mut debugger = Debugger::new();

    // If the user specified a program load it here
    if let Some(program) = args.value_of("program") {
        debugger.load(program.to_string())?;
    }

    if let Some(pid) = args.value_of("pid") {
        if let Ok(pid) = pid.parse::<i32>() {
            debugger.attach(pid)?;
        } else {
            warn!("Invalid process-id")
        }
    }

    loop {
        let p = format!("rdbg> ");
        rl.helper_mut().expect("No helper found").colored_prompt =
            format!("\x1b[1;32m{}\x1b[0m", p);

        let readline = rl.readline(&p);
        match readline {
            Ok(mut line) => {
                // Mimics GDB behavior by executing the last command
                if line.trim().is_empty() {
                    if rl.history().is_empty() {
                        continue;
                    } else {
                        line = rl.history().last().unwrap().clone(); // safe unwrap
                    }
                } else {
                    rl.add_history_entry::<&str>(line.as_ref());
                }

                let mut tokens = line.split_whitespace();
                match tokens.next().unwrap() {
                    "start" => {
                        debugger.spawn()?;
                    }
                    "quit" => break,
                    _ => println!("Unknown command"),
                };
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                error!("Unknown Rustyline Error: {:?}", err);
                break;
            }
        }
    }

    rl.save_history(&history_file).ok();
    Ok(())
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
