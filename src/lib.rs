//! # rdbg_core
//!
//! The `rdbg_core` library contains the core functionality of the debugger.

#[macro_use]
extern crate log;
#[macro_use]
extern crate cfg_if;
#[macro_use]
extern crate error_chain;
extern crate ansi_term;
extern crate capstone;
extern crate nix;
extern crate object;
extern crate simplelog;

#[macro_use]
mod macros;
pub mod api;
mod arch;
mod core;
pub mod util;

/// An integer type, whose size equals a machine word.
///
/// `ptrace` always returns a machine word. This type provides an abstraction
/// of the fact that on *nix systems, `c_long` is always a machine word,
/// so as to prevent the library from leaking C implementation-dependent types.
pub type Word = usize;

// Define common types across different platforms.
cfg_if! {
    if #[cfg(target_os = "windows")] {
        // TODO: What should the Pid type be on Windows?
        // pub type Pid = ;
    } else if #[cfg(target_os = "linux")] {
        pub type Pid = nix::unistd::Pid;
    } else {
        // Unknown or unsupported target_family
    }
}

// The `sys` directory provides wrappers around system specific low-level APIs.
// To add debugging support for a new system, create a new stub file that
// implements the same common API as the others.
cfg_if! {
    if #[cfg(target_os = "windows")] {
        #[path="sys/windows.rs"]
        mod sys;
    } else if #[cfg(target_os = "linux")] {
        #[path="sys/linux.rs"]
        mod sys;
    } else {
        // Unknown or unsupported target_family
    }
}

pub use sys::*;
