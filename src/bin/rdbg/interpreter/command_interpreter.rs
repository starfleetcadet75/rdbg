use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::error::Error;

use rdbg_core::core::debugger;

static PROMPT: &'static str = "\x1b[1;32mrdbg>\x1b[0m ";

pub struct CommandInterpreter {
    debugger: debugger::Debugger,
}

impl CommandInterpreter {
    pub fn new(debugger: debugger::Debugger) -> CommandInterpreter {
        let interpreter = CommandInterpreter {
            debugger,
        };
        interpreter
    }

    pub fn read_line(&self) -> Result<(), Box<Error>> {
        let history_file = "/tmp/.rdbg_history";
        debug!("Starting debugger...");

        let mut rl = Editor::new().history_ignore_space(true);
        let completer = FilenameCompleter::new();
        rl.set_completer(Some(completer));

        if let Err(_) = rl.load_history(history_file) {
            info!("No previous command history file found at: {}", history_file);
        }

        loop {
            let readline = rl.readline(PROMPT);
            match readline {
                Ok(line) => {
                    rl.add_history_entry(&line);
                    debug!("User Command: {}", line);
                    self.handle_command(&line);
                },
                Err(ReadlineError::Interrupted) => { break },  // Handle Ctrl-C
                Err(ReadlineError::Eof) => { break },  // Handle Ctrl-D
                Err(err) => { error!("Unknown Error (Rustyline): {:?}", err); break },
            }
        }
        rl.save_history(history_file).unwrap();
        Ok(())
    }

    fn handle_command(&self, cmd: &str) {

    }

    fn handle_unknown_command(&self, cmd: &str) {
        println!("Undefined command: \"{}\".  Try \"help\"", cmd);
    }
}

