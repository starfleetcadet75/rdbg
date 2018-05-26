use api::Command;
use core::debugger::Debugger;
use util::errors::*;

pub struct StartCommand;

impl Command for StartCommand {
    fn execute(&self, _: &[&str], debugger: &mut Debugger) -> RdbgResult<()> { debugger.execute() }

    fn usage(&self) {
        println!("Starts debugging the loaded program");
    }
}
