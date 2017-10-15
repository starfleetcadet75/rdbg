use rustyline::{CompletionType, Config, Editor};
use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;

use std::error::Error;
use std::path::Path;

use rdbg_core::core::profile::Profile;
use rpc::SyncClient;

static PROMPT: &'static str = "\x1b[1;32mrdbg>\x1b[0m ";

pub struct Interpreter {
    client: SyncClient,
}

impl Interpreter {
    pub fn new(client: SyncClient) -> Interpreter { Interpreter { client: client } }

    pub fn read_line(&self) -> Result<(), Box<Error>> {
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
                            break;
                        } else {
                            line = rl.get_history().last().unwrap().clone(); // safe unwrap
                        }
                    } else {
                        rl.add_history_entry(line.as_ref());
                    }

                    // Split the input by spaces
                    let v: Vec<&str> = line.split(' ').collect();
                    if v[0] == "quit" || v[0] == "q" {
                        // Handle quit command in the first word
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

        // Save the history after exiting loop
        rl.save_history(history_file).expect(
            "Unable to write history file",
        );
        Ok(())
    }

    fn handle_command(&self, input: Vec<&str>) {
        let cmd = input[0];
        if cmd == "run" {
            self.run();
        } else if cmd == "load" {
            self.load_profile_command(Profile::new(Path::new(input[1])));
        } else {
            self.handle_unknown_command(cmd);
        }
    }

    pub fn run(&self) {
        match self.client.run() {
            Ok(result) => println!("{}", result),
            Err(error) => println!("error: {}", error),
        }
    }

    pub fn load_profile_command(&self, profile: Profile) {
        match self.client.new_project(profile) {
            Ok(result) => println!("{}", result),
            Err(error) => println!("error: {}", error),
        }
    }

    fn handle_unknown_command(&self, cmd: &str) {
        println!("Undefined command: \"{}\".  Try \"help\"", cmd);
    }
}
