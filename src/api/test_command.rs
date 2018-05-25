use api::Command;
use core::debugger::Debugger;
use util::errors::*;

impl Command for TestCommand {
    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> RdbgResult<()> {
        if args.len() < 2 {
            self.usage();
        } else {
            println!("{:#x}", debugger.program.entry());
        }
        Ok(())
    }

    fn usage(&self) {
        println!("Test command");
    }
}
