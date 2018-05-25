use api::Command;
use core::debugger::Debugger;
use util::errors::*;

pub struct EntryCommand;

impl Command for EntryCommand {
    fn execute(&self, _: &[&str], debugger: &mut Debugger) -> RdbgResult<()> {
        println!("{:#x}", debugger.program.entry());
        Ok(())
    }

    fn usage(&self) {
        println!("Prints the entry point to the program");
    }
}
