use api::Command;
use core::debugger::Debugger;
use util::errors::*;

pub struct StepCommand;

impl Command for StepCommand {
    fn execute(&self, _: &[&str], debugger: &mut Debugger) -> RdbgResult<()> {
        debugger.single_step()
    }

    fn usage(&self) {
        println!("Single step the next instruction");
    }
}
