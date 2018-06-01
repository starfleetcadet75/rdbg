use api::Command;
use core::debugger::Debugger;
use util::errors::*;

pub struct BreakCommand;

impl Command for BreakCommand {
    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> RdbgResult<()> {
        OnlyWhenRunning!(debugger);

        if 0 < args.len() {
            if args[0] == "enable" || args[0] == "e" {
                if args[1].starts_with("0x") {
                    debugger.enable_breakpoint(FromHexString!(args[1]))?;
                } else {
                    println!("Must specify address of the breakpoint to enable");
                }
            } else if args[0] == "disable" || args[0] == "d" {
                if args[1].starts_with("0x") {
                    debugger.disable_breakpoint(FromHexString!(args[1]))?;
                } else {
                    println!("Must specify address of the breakpoint to disable");
                }
            } else if args[0] == "remove" || args[0] == "r" {
                if args[1].starts_with("0x") {
                    debugger.remove_breakpoint(FromHexString!(args[1]))?;
                } else {
                    println!("Must specify address of the breakpoint to remove");
                }
            } else if args[0].starts_with("0x") {
                debugger.set_breakpoint_at(FromHexString!(args[0]))?;
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
