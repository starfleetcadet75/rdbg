#[allow(dead_code)]


#[macro_use]
extern crate log;
extern crate simplelog;
extern crate nix;
extern crate libc;

pub mod core;
pub mod commands;
mod breakpoint;

pub type InferiorPid = nix::unistd::Pid;
