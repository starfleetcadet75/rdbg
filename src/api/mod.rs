use core::debugger::Debugger;
use util::errors::*;

pub use self::rdbg_api::RdbgApi;

mod break_command;
mod continue_command;
mod detach_command;
mod entry_command;
mod hexdump_command;
mod kill_command;
mod near_command;
mod procinfo_command;
mod rdbg_api;
mod regs_command;
mod sections_command;
mod start_command;
mod stepi_command;

pub trait Command {
    fn execute(&self, &[&str], debugger: &mut Debugger) -> RdbgResult<()>;
    fn usage(&self);
}
