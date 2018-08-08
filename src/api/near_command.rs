use api::Command;
use core::debugger::Debugger;
use util::errors::*;

pub struct NearCommand;

impl Command for NearCommand {
    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> RdbgResult<()> {
        OnlyWhenRunning!(debugger);

        let pc = debugger.program.architecture.instruction_pointer();
        let mut address = debugger.read_register(pc)?;

        if 0 < args.len() {
            if args[0].starts_with("0x") {
                address = FromHexString!(args[0]);
            } else if args[0].starts_with("$") {
                address = debugger.read_register(args[0].split("$").skip(1).next().unwrap())?;
            }
        }

        let instrs = debugger.disassemble(address, 12)?;
        for i in instrs.iter() {
            println!("{}", i);
        }
        Ok(())
    }

    fn usage(&self) {
        println!("Disassemble a specified section of memory.");
    }
}
