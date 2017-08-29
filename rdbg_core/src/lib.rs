//! The `rdbg_core` library contains the core functionality of the debugger.
//! Custom clients can be written around the core.
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate nix;
extern crate libc;
extern crate fnv;
extern crate goblin;

pub mod core;
pub mod commands;
mod breakpoint;
pub mod util;

pub type Pid = nix::unistd::Pid;
pub type Address = u64;