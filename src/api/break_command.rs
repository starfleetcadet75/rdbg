use api::Command;
use core::debugger::Debugger;
use sys::Word;
use util::errors::*;

pub struct BreakCommand;

impl Command for BreakCommand {
    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> RdbgResult<()> {
        OnlyWhenRunning!(debugger);

        if 0 < args.len() {
            if args[0].starts_with("0x") {
                // Remove '0x' from hex address
                let address = Word::from_str_radix(args[0].split("x").skip(1).next().unwrap(), 16)
                    .chain_err(|| format!("Invalid address: {}", args[0]))?;
                debugger.set_breakpoint_at(address)?;
            } else {
                println!("Invalid address: {}", args[0]);
            }
        } else {
            debugger.print_breakpoints();
        }
        Ok(())
    }

    fn usage(&self) {
        println!("Set breakpoint at specified location.");
    }
}
