use api::Command;
use core::debugger::Debugger;
use util::errors::*;

pub struct DetachCommand;

impl Command for DetachCommand {
    fn execute(&self, _: &[&str], debugger: &mut Debugger) -> RdbgResult<()> {
        OnlyWhenRunning!(debugger);
        debugger.detach()
    }

    fn usage(&self) {
        println!("Detach from a process previously attached.");
    }
}
