use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use fnv::FnvHashMap;

use std::error::Error;

use rdbg_core::core::debugger;
use rdbg_core::commands;

static PROMPT: &'static str = "\x1b[1;32mrdbg>\x1b[0m ";

pub struct CommandInterpreter {
    debugger: debugger::Debugger,
    commands: FnvHashMap<&'static str, commands::Command>,
}

impl CommandInterpreter {
    pub fn new(debugger: debugger::Debugger) -> CommandInterpreter {
        CommandInterpreter {
            debugger: debugger,
            commands: commands::Command::map(),
        }
    }

    pub fn read_line(&mut self) -> Result<(), Box<Error>> {
        let history_file = "/tmp/.rdbg_history";
        debug!("Starting debugger session");

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
                    debug!("User Command: {}", line);

                    rl.add_history_entry(&line);
                    if line == "quit" { 
                        break; 
                    }

                    if line == "help" {
                        self.command_help(&[]);
                    }
                    else {
                        self.handle_command(&line);
                    }
                },
                Err(ReadlineError::Interrupted) => { break },  // Handle Ctrl-C
                Err(ReadlineError::Eof) => { break },  // Handle Ctrl-D
                Err(err) => { error!("Unknown Error (Rustyline): {:?}", err); break },
            }
        }
        rl.save_history(history_file).unwrap();
        Ok(())
    }

    fn handle_command(&mut self, cmd: &str) {
        // add check for help along with function to split the line
        if let Some(cmd) = self.commands.get(cmd) {
            let status = (cmd.execute)(&[], &mut self.debugger);
            println!("{}", status);
        }
        else {
            self.handle_unknown_command(cmd);
        }
    }

    fn command_help(&self, args: &[&str]) {
        if args.len() == 0 {
            println!("This is the help message");
        }
        else if let Some(cmd) = self.commands.get(args[0]) {
            println!("{}", cmd.help);
        }
        else {
            self.handle_unknown_command(args[0]);
        }
    }

    fn handle_unknown_command(&self, cmd: &str) {
        println!("Undefined command: \"{}\".  Try \"help\"", cmd);
    }
}

