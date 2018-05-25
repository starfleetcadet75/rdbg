use api::Command;
use core::debugger::Debugger;
use util::errors::*;

pub struct RegsCommand;

impl Command for RegsCommand {
    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> RdbgResult<()> {
        if 0 < args.len() {
            // TODO: Only print the registers given as args, in the order given
            self.print_register("rip", debugger)?;
        } else {
            self.print_register("rip", debugger)?;

            // TODO: Need a way to iterate through the register names
            // for register in debugger.program.architecture.registers() {
            //     self.print_register(register, debugger)?;
            // }
        }
        Ok(())
    }

    fn usage(&self) {
        println!("Show registers");
    }
}

impl RegsCommand {
    fn print_register(&self, register: &str, debugger: &mut Debugger) -> RdbgResult<()> {
        println!("{:?}\t{:#x}", register, debugger.read_register(register)?);
        Ok(())
    }
}
