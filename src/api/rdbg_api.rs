use std::collections::HashMap;

use api::break_command::BreakCommand;
use api::continue_command::ContinueCommand;
use api::detach_command::DetachCommand;
use api::entry_command::EntryCommand;
use api::hexdump_command::HexdumpCommand;
use api::kill_command::KillCommand;
use api::near_command::NearCommand;
use api::procinfo_command::ProcinfoCommand;
use api::regs_command::RegsCommand;
use api::sections_command::SectionsCommand;
use api::start_command::StartCommand;
use api::stepi_command::StepiCommand;
use api::Command;
use core::debugger::Debugger;
use util::errors::*;

pub struct RdbgApi {
    commands: HashMap<&'static str, Box<Command>>,
    debugger: Debugger,
}

impl RdbgApi {
    pub fn new(program_path: String) -> RdbgApi {
        let mut commands: HashMap<&str, Box<Command>> = HashMap::new();

        commands.insert("entry", Box::new(EntryCommand));
        commands.insert("start", Box::new(StartCommand));
        commands.insert("continue", Box::new(ContinueCommand));
        commands.insert("break", Box::new(BreakCommand));
        commands.insert("regs", Box::new(RegsCommand));
        commands.insert("near", Box::new(NearCommand));
        commands.insert("stepi", Box::new(StepiCommand));
        commands.insert("procinfo", Box::new(ProcinfoCommand));
        commands.insert("hexdump", Box::new(HexdumpCommand));
        commands.insert("kill", Box::new(KillCommand));
        commands.insert("detach", Box::new(DetachCommand));
        commands.insert("sections", Box::new(SectionsCommand));

        RdbgApi {
            commands: commands,
            debugger: Debugger::new(program_path),
        }
    }

    pub fn run(&mut self, command: &str) -> RdbgResult<()> {
        // Split the input by spaces
        let v: Vec<&str> = command.split(' ').collect();

        match self.commands.get(v[0]) {
            Some(cmd) => {
                let mut args = v.as_slice();

                if 0 < args.len() {
                    args = &args[1..];
                }

                debug!("Calling \'{}\' command", v[0]);
                cmd.execute(args, &mut self.debugger)
            }
            None => RdbgApi::handle_unknown_command(v[0]),
        }
    }

    fn handle_unknown_command(cmd: &str) -> RdbgResult<()> {
        println!("Undefined command: \"{}\".  Try \"help\"", cmd);
        Ok(())
    }
}
