use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Config, Editor};

use rdbg_core::api::RdbgApi;
use rdbg_core::util::errors::*;

static PROMPT: &'static str = "\x1b[1;32mrdbg>\x1b[0m ";

pub struct Interpreter {
    api: RdbgApi,
}

impl Interpreter {
    pub fn new(path: String) -> Interpreter {
        Interpreter {
            api: RdbgApi::new(path),
        }
    }

    pub fn read_line(&mut self) -> RdbgResult<()> {
        let history_file = "/tmp/.rdbg_history";
        debug!("Starting debugger session");

        // Setup the rustyline configuration
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::List)
            .build();
        let mut rl = Editor::with_config(config);

        let completer = FilenameCompleter::new();
        rl.set_completer(Some(completer)); // TODO: rustyline only supports using one completer

        // Attempt to load the history file
        if let Err(_) = rl.load_history(history_file) {
            info!(
                "No previous command history file found at: {}",
                history_file
            );
        }

        // Main interpreter loop
        loop {
            let readline = rl.readline(PROMPT);
            match readline {
                Ok(mut line) => {
                    debug!("User entered command: {}", line);

                    // Mimics GDB behavior by executing the last command
                    if line.is_empty() {
                        if rl.get_history().is_empty() {
                            continue;
                        } else {
                            line = rl.get_history().last().unwrap().clone(); // safe unwrap
                        }
                    } else {
                        rl.add_history_entry(line.as_ref());
                    }

                    self.api.run(&line)?;
                }
                Err(ReadlineError::Interrupted) => break, // Handle Ctrl-C
                Err(ReadlineError::Eof) => break,         // Handle Ctrl-D
                Err(err) => {
                    error!("Unknown Error (Rustyline): {:?}", err);
                    break;
                }
            }
        }

        // Save the history after exiting loop
        rl.save_history(history_file)
            .expect("Unable to write history file");
        Ok(())
    }
}
