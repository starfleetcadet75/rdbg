//! # rdbg_core
//!
//! The `rdbg_core` library contains the core functionality of the debugger.

#[macro_use]
extern crate log;
extern crate simplelog;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate nix;
extern crate libc;
extern crate fnv;
extern crate goblin;

pub mod core;
pub mod util;
mod loaders;
mod formats;
mod analysis;
mod stubs;

// TODO: Better way to generalize these for different platforms
pub type Pid = nix::unistd::Pid;
pub type Word = usize;