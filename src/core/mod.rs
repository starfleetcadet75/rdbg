mod breakpoint;
pub mod debugger;
mod memory;
mod program;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TraceEvent {
    SyscallEnter,
    SyscallExit,
    Continued,
    Breakpoint,
    Signal(u8),
    Event(u8),
    Killed(u8, bool),
    Exit(i32),
}
