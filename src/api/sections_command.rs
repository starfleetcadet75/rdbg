use api::Command;
use core::debugger::Debugger;
use util::errors::*;

pub struct SectionsCommand;

impl Command for SectionsCommand {
    fn execute(&self, _: &[&str], debugger: &mut Debugger) -> RdbgResult<()> {
        println!("{}", debugger.program.sections());
        Ok(())
    }

    fn usage(&self) {
        println!("Prints the section mappings of the program");
    }
}
