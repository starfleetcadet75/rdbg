use api::Command;
use core::debugger::Debugger;
use util::errors::*;

pub struct ProcinfoCommand;

impl Command for ProcinfoCommand {
    fn execute(&self, _: &[&str], debugger: &mut Debugger) -> RdbgResult<()> {
        println!("{}", debugger.procinfo()?);
        Ok(())
    }

    fn usage(&self) {
        println!("Displays information about the process being debugged.");
    }
}
