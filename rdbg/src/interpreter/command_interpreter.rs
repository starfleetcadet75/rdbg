use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use fnv::FnvHashMap;

use std::error::Error;

use rdbg_core::core::debugger;
use rdbg_core::commands::Command;
use rdbg_core::util::error::RdbgResult;

static PROMPT: &'static str = "\x1b[1;32mrdbg>\x1b[0m ";

pub struct CommandInterpreter {
    dbg: debugger::Debugger,
    commands: FnvHashMap<&'static str, Command>,
}

impl CommandInterpreter {
    pub fn new() -> CommandInterpreter {
        CommandInterpreter {
            dbg: debugger::Debugger::new(),
            commands: Command::map(),
        }
    }

    pub fn set_program(&mut self, path: &str) -> RdbgResult<()> {
        let load_cmd = self.commands.get("load").unwrap(); // safe unwrap, load is a command
        (load_cmd.execute)(&[path], &mut self.dbg)
    }

    pub fn read_line(&mut self) -> Result<(), Box<Error>> {
        let history_file = "/tmp/.rdbg_history";
        debug!("Starting debugger session");

        let mut rl = Editor::new().history_ignore_space(true);
        let completer = FilenameCompleter::new();
        rl.set_completer(Some(completer));

        if let Err(_) = rl.load_history(history_file) {
            info!(
                "No previous command history file found at: {}",
                history_file
            );
        }

        loop {
            let readline = rl.readline(PROMPT);
            match readline {
                Ok(line) => {
                    debug!("User Command: {}", line);
                    rl.add_history_entry(&line);

                    let v: Vec<&str> = line.split(' ').collect();
                    if v[0] == "quit" || v[0] == "q" {
                        break;
                    }

                    self.handle_command(v);
                }
                Err(ReadlineError::Interrupted) => break,  // Handle Ctrl-C
                Err(ReadlineError::Eof) => break,  // Handle Ctrl-D
                Err(err) => {
                    error!("Unknown Error (Rustyline): {:?}", err);
                    break;
                }
            }
        }
        rl.save_history(history_file).unwrap();
        Ok(())
    }

    fn handle_command(&mut self, input: Vec<&str>) {
        let cmd = input[0];
        match self.commands.get(cmd) {
            Some(cmd) => {
                let mut args = input.as_slice();

                if 0 < args.len() {
                    args = &args[1..];
                }

                if let Err(e) = (cmd.execute)(args, &mut self.dbg) {
                    error!("Error ({}) executing command: {}", e, cmd.name);
                }
            }
            None => self.handle_unknown_command(cmd),
        }
    }

    fn handle_unknown_command(&self, cmd: &str) {
        println!("Undefined command: \"{}\".  Try \"help\"", cmd);
    }
}
