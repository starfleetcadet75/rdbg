use api::Command;
use core::debugger::Debugger;
use util::errors::*;

pub struct ContinueCommand;

impl Command for ContinueCommand {
    fn execute(&self, _: &[&str], debugger: &mut Debugger) -> RdbgResult<()> {
        OnlyWhenRunning!(debugger);
        debugger.continue_execution()
    }

    fn usage(&self) {
        println!("Continue program being debugged, after signal or breakpoint");
    }
}
