use api::Command;
use core::debugger::Debugger;
use util::errors::*;

pub struct KillCommand;

impl Command for KillCommand {
    fn execute(&self, _: &[&str], debugger: &mut Debugger) -> RdbgResult<()> {
        OnlyWhenRunning!(debugger);
        debugger.kill()
    }

    fn usage(&self) {
        println!("Kill execution of program being debugged.");
    }
}
