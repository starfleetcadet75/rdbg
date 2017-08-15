//! The `rdbg_core` library contains the core functionality of the debugger.
//! Custom clients can be written around the core.
//!
//! # Examples
//!
//! ```
//! use std::path::Path;
//! use debugger::Debugger;
//!
//! let program = Path::new("./hello_world.bin");
//! let mut dbg = debugger::Debugger::new();
//!
//! dbg.execute_target(program, &[]) {
//! dbg.continue_execution();
//! ```
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate nix;
extern crate libc;
extern crate fnv;

pub mod core;
pub mod commands;
mod breakpoint;

use libc::c_void;

pub type Pid = nix::unistd::Pid;

#[derive(Debug, Copy, Clone)]
pub struct Address(pub u64);

impl Address {
    pub fn as_void_ptr(&self) -> * mut c_void {
        let &Address(addr) = self;
        addr as * mut c_void
    }
}
