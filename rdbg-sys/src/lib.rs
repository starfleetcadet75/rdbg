//! The `rdbg_sys` crate exposes a common API for performing platform-specific operations.

pub mod errors;

#[cfg(target_family = "unix")]
mod unix;
#[cfg(target_family = "unix")]
pub use unix::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TraceEvent {
    Signal(u8), // TODO: Expand this into each signal?
    Event(u8),
    SyscallEnter,
    //    SyscallExit,
    /// Process resumed execution
    Continued,
    /// A SIGTRAP occurred
    Breakpoint,
    /// Contains the signal number that killed the process and whether a coredump occurred
    Killed(u8, bool),
    /// Contains the error code the process exited with
    Exit(i32),
}
