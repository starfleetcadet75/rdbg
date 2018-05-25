use api::Command;
use core::debugger::Debugger;
use util::errors::*;

pub struct ContinueCommand;

impl Command for ContinueCommand {
    fn execute(&self, _: &[&str], debugger: &mut Debugger) -> RdbgResult<()> {
        debugger.continue_execution()
    }

    fn usage(&self) {
        println!("Continues running the tracee");
    }
}
