use fnv::FnvHashMap;

use std::path::PathBuf;
use std::str::FromStr;

use super::Address;
use super::core::{program, debugger};
use super::core::arch::{Arch, Register};
use super::util::error::RdbgResult;

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

        insert_command!("print", "Print value of expression EXP.", command_print);

        insert_command!("memory", "Read or write to process memory.", command_memory);

        insert_command!("stepi", "Step one instruction exactly.", command_stepi);

        commands
    }
}

fn command_load(args: &[&str], dbg: &mut debugger::Debugger) -> RdbgResult<()> {
    debug!("Calling load command");
    let path = &PathBuf::from(args[0]);
    let program = program::Program::new(path);
    dbg.load_program(program)
}

fn command_start(args: &[&str], dbg: &mut debugger::Debugger) -> RdbgResult<()> {
    debug!("Calling start command");
    dbg.execute_target()
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
    let address = Address::from_str_radix(args[0], 16).unwrap();
    dbg.set_breakpoint_at(address);
    Ok(())
}

fn command_print(args: &[&str], dbg: &mut debugger::Debugger) -> RdbgResult<()> {
    println!(
        "RIP: {:?}",
        format!("{:#x}", dbg.get_register_value(Register::Rip)?)
    );
    Ok(())
}

fn command_memory(args: &[&str], dbg: &mut debugger::Debugger) -> RdbgResult<()> {
    if args[0] == "read" {
        let address = Address::from_str(args[1]).unwrap();
        println!("{:?}", format!("{:#x}", dbg.read_memory(address)?));
    }
    if args[0] == "write" {
        dbg.write_memory(
            Address::from_str(args[1]).unwrap(),
            i64::from_str(args[2]).unwrap(),
        )?;
    }
    Ok(())
}

fn command_stepi(args: &[&str], dbg: &mut debugger::Debugger) -> RdbgResult<()> {
    dbg.single_step_instruction_with_breakpoints()?;
    Ok(())
}
