//! # rdbg_core
//!
//! The `rdbg_core` library contains the core functionality of the debugger.

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate capstone;
extern crate nix;
extern crate object;
extern crate simplelog;

#[macro_use]
mod macros;
pub mod api;
mod arch;
mod core;
mod sys;
pub mod util;
