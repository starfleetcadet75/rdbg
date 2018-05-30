use api::Command;
use core::debugger::Debugger;
use util::errors::*;

pub struct StepiCommand;

impl Command for StepiCommand {
    fn execute(&self, _: &[&str], debugger: &mut Debugger) -> RdbgResult<()> {
        OnlyWhenRunning!(debugger);
        debugger.single_step_with_breakpoint()
    }

    fn usage(&self) {
        println!("Single step the next instruction.");
    }
}
