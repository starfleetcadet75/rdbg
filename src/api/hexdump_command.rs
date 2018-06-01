use api::Command;
use core::debugger::Debugger;
use util::errors::*;
use util::hexdump;

pub struct HexdumpCommand;

impl Command for HexdumpCommand {
    // TODO: Find a cleaner way of parsing these annoying args
    fn execute(&self, args: &[&str], debugger: &mut Debugger) -> RdbgResult<()> {
        OnlyWhenRunning!(debugger);

        let mut address = debugger.read_register("rsp")?;
        let mut format = "x";
        let mut size = 64;

        if 0 < args.len() {
            if args[0] == "c" || args[0] == "r" || args[0] == "g" {
                if 1 < args.len() {
                    if args[1].starts_with("0x") {
                        address = FromHexString!(args[1]);
                    } else if args[1].starts_with("$") {
                        address =
                            debugger.read_register(args[1].split("$").skip(1).next().unwrap())?;
                    }
                }

                let data = debugger.read_memory(address, size)?;
                hexdump::dump_array(&data, args[0]);
                return Ok(());
            } else if args[0] == "o" || args[0] == "x" || args[0] == "X" || args[0] == "b" {
                format = args[0];
            } else if args[0].starts_with("0x") {
                address = FromHexString!(args[0]);
            } else if args[0].starts_with("$") {
                address = debugger.read_register(args[0].split("$").skip(1).next().unwrap())?;
            }

            if 1 < args.len() {
                size = args[1]
                    .trim()
                    .parse::<usize>()
                    .expect("Failed to parse size argument");
            }

            let data = debugger.read_memory(address, size)?;
            hexdump::dump(&data, format);
        }
        Ok(())
    }

    fn usage(&self) {
        println!("Hexdumps data at the specified address or register. hexdump [location] [size]");
    }
}
