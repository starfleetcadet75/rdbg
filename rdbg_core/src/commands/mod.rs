use fnv::FnvHashMap;

use std::path::Path;
use std::str::FromStr;

use Address;
use core::arch::Arch;
use core::debugger;
use util::error::{RdbgError, RdbgResult};

pub struct Command {
    /// The name of the command.
    pub name: &'static str,
    // The help message to be printed by the help command.
    pub help: &'static str,
    // The function to be called when the command is entered.
    pub execute: fn(&[&str], &mut debugger::Debugger) -> RdbgResult<()>,
}

impl Command {
    /// Creates and returns a `FnvHashMap` containing the builtin commands.
    pub fn map() -> FnvHashMap<&'static str, Self> {
        let mut commands: FnvHashMap<&str, Self> =
            FnvHashMap::with_capacity_and_hasher(32, Default::default());

        // Macro for inserting commands into the command map
        // Credit: https://github.com/redox-os/ion/blob/master/src/builtins/mod.rs
        macro_rules! insert_command {
            ($name:expr, $help:expr, $func:ident) => {
                commands.insert(
                    $name,
                    Command {
                        name: $name,
                        help: $help,
                        execute: $func,
                    }
                );
            }
        }

        insert_command!(
            "load",
            "Load the FILE as the program to be debugged.",
            command_load
        );

        insert_command!(
            "start",
            "Starts executing the loaded program.",
            command_start
        );

        insert_command!("entry", "Prints the entry point address.", command_entry);

        insert_command!(
            "procinfo",
            "Displays information about the process being debugged.",
            command_procinfo
        );

        insert_command!(
            "continue",
            "Continue program being debugged, after signal or breakpoint.",
            command_continue
        );

        insert_command!(
            "break",
            "Set breakpoint at specified location.",
            command_break
        );

        insert_command!(
            "clear",
            "Clears a breakpoint at the specified location.",
            command_clear
        );

        insert_command!(
            "enable",
            "Enables the breakpoint at the given address.",
            command_enable
        );

        insert_command!(
            "disable",
            "Disables the breakpoint at the given address.",
            command_disable
        );

        insert_command!("regs", "Print register values.", command_regs);

        insert_command!("mem", "Read or write to process memory.", command_memory);

        insert_command!("stepi", "Step one instruction exactly.", command_stepi);

        commands
    }
}

fn command_load(args: &[&str], dbg: &mut debugger::Debugger) -> RdbgResult<()> {
    debug!("Calling load command");
    check_args_len(args.len(), 1)?;

    let path = Path::new(args[0]);
    // if 1 < args.len() {
    //     args = &args[1..];
    // }

    // program.args(args);
    dbg.load_program(path)
}

fn command_start(args: &[&str], dbg: &mut debugger::Debugger) -> RdbgResult<()> {
    debug!("Calling start command");
    dbg.execute_target()
}

fn command_entry(args: &[&str], dbg: &mut debugger::Debugger) -> RdbgResult<()> {
    debug!("Calling entry command");
    println!("Entry: {:#x}", dbg.get_entrypoint()?);
    Ok(())
}

fn command_procinfo(args: &[&str], dbg: &mut debugger::Debugger) -> RdbgResult<()> {
    debug!("Calling procinfo command");
    dbg.procinfo()
}

fn command_continue(args: &[&str], dbg: &mut debugger::Debugger) -> RdbgResult<()> {
    debug!("Calling continue command");
    dbg.continue_execution()
}

fn command_break(args: &[&str], dbg: &mut debugger::Debugger) -> RdbgResult<()> {
    debug!("Calling break command");

    if 0 < args.len() {
        let mut address = 0;
        if args[0].starts_with("0x") {
            address = Address::from_str_radix(args[0].split("x").skip(1).next().unwrap(), 16)?;
        }
        dbg.set_breakpoint_at(address)?;
    } else {
        dbg.print_breakpoints();
    }
    Ok(())
}

fn command_clear(args: &[&str], dbg: &mut debugger::Debugger) -> RdbgResult<()> {
    debug!("Calling clear command");

    check_args_len(args.len(), 1)?;
    if args[0].starts_with("0x") {
        let address = Address::from_str_radix(args[0].split("x").skip(1).next().unwrap(), 16)?;
        dbg.remove_breakpoint(address)?;
    }
    Ok(())
}

fn command_enable(args: &[&str], dbg: &mut debugger::Debugger) -> RdbgResult<()> {
    debug!("Calling enable command");

    check_args_len(args.len(), 1)?;
    if args[0].starts_with("0x") {
        let address = Address::from_str_radix(args[0].split("x").skip(1).next().unwrap(), 16)?;
        dbg.enable_breakpoint(address)?;
    }
    Ok(())
}

fn command_disable(args: &[&str], dbg: &mut debugger::Debugger) -> RdbgResult<()> {
    debug!("Calling disable command");

    check_args_len(args.len(), 1)?;
    if args[0].starts_with("0x") {
        let address = Address::from_str_radix(args[0].split("x").skip(1).next().unwrap(), 16)?;
        dbg.disable_breakpoint(address)?;
    }
    Ok(())
}

fn command_regs(args: &[&str], dbg: &mut debugger::Debugger) -> RdbgResult<()> { dbg.print_regs() }

fn command_memory(args: &[&str], dbg: &mut debugger::Debugger) -> RdbgResult<()> {
    check_args_len(args.len(), 2)?;

    let mut address = 0;
    if args[1].starts_with("0x") {
        address = Address::from_str_radix(args[1].split("x").skip(1).next().unwrap(), 16)?;
    }

    debug!("Input Address: {:#x}", address);
    if args[0] == "read" || args[0] == "r" {
        println!("{:#x}", dbg.read_memory(address)?);
    }
    if args[0] == "write" || args[0] == "w" {
        dbg.write_memory(address, i64::from_str(args[2]).unwrap())?;
    }
    Ok(())
}

fn command_stepi(args: &[&str], dbg: &mut debugger::Debugger) -> RdbgResult<()> {
    dbg.single_step_instruction_with_breakpoints()?;
    Ok(())
}

fn check_args_len(len: usize, needed: usize) -> RdbgResult<()> {
    if len < needed {
        Err(RdbgError::NotEnoughArgs)
    } else {
        Ok(())
    }
}
