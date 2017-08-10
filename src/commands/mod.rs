use fnv::FnvHashMap;
use super::core::debugger;

pub struct Command {
    pub name: &'static str,  // name of the command
    pub help: &'static str,  // help message
    pub execute: fn(&[&str], &mut debugger::Debugger) -> i32,  // execute fn with given arguments to the cmd
}

impl Command {
    pub fn map() -> FnvHashMap<&'static str, Self> {
        let mut commands: FnvHashMap<&str, Self> = FnvHashMap::with_capacity_and_hasher(32, Default::default());

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

        //insert_command!("help", command_help, "Print list of commands.");
        insert_command!("continue", "Continue program being debugged, after signal or breakpoint.", command_continue);

        // break [address]
        // run [arglist]
        // bt
        // step
        // help [name]
        // quit

        commands
    }
}

fn command_continue(args: &[&str], dbg: &mut debugger::Debugger) -> i32 {
    info!("Calling continue command");
    dbg.continue_execution();
    0
}

