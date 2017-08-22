use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::{Config, CompletionType, Editor};
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


        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::List)
            .build();
        let mut rl = Editor::with_config(config);
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
                Ok(mut line) => {
                    debug!("User Command: {}", line);

                    if line.is_empty() {
                        if rl.get_history().is_empty() {
                            break;
                        } else {
                            line = rl.get_history().last().unwrap().clone(); // safe unwrap
                        }
                    } else {
                        rl.add_history_entry(line.as_ref());
                    }

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
        rl.save_history(history_file).expect(
            "Unable to write history file",
        );
        Ok(())
    }

    fn handle_command(&mut self, input: Vec<&str>) {
        let cmd = input[0];

        if cmd == "help" {
            // handle the help command
            if 1 < input.len() {
                match self.commands.get(input[1]) {
                    // print the help msg for the given cmd
                    Some(cmd) => println!("{}", cmd.help),
                    None => self.handle_unknown_command(cmd),
                }
            } else {
                println!("This is the help message.");
            }
        } else {
            match self.commands.get(cmd) {
                Some(cmd) => {
                    let mut args = input.as_slice();

                    // handle other args to the command that need to be forwarded along
                    if 0 < args.len() {
                        args = &args[1..];
                    }

                    // try to execute the command with the given args
                    if let Err(e) = (cmd.execute)(args, &mut self.dbg) {
                        error!("Error ({}) executing command: {}", e, cmd.name);
                    }
                }
                None => self.handle_unknown_command(cmd),
            }
        }
    }

    fn handle_unknown_command(&self, cmd: &str) {
        println!("Undefined command: \"{}\".  Try \"help\"", cmd);
    }
}
