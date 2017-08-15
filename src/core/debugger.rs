//! The main `Debugger` module.
//! This module contains the main interface for the core functionality.
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{execve, fork, ForkResult, Pid};

use std::path::Path;
use std::error::Error;
use std::ffi::CString;
use fnv::FnvHashMap;

use super::ptrace_wrapper;
use super::super::InferiorPid;
use super::super::breakpoint::breakpoint;

pub struct Debugger {
    pid: InferiorPid,
    breakpoints: FnvHashMap<u64, breakpoint::Breakpoint>,
}

impl Debugger {
    /// Creates a new `Debugger`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use rdbg_core::core::debugger;
    /// let mut dbg = debugger::Debugger::new();
    /// ```
    pub fn new() -> Debugger {
        Debugger {
            pid: InferiorPid::from_raw(0),
            breakpoints: FnvHashMap::default(),
        }
    }

    /// Starts debugging of the target given by the program path.
    /// Passes any arguments given as parameters as arguments to the new program.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::path::Path;
    /// use rdbg_core::core::debugger;
    ///
    /// let program = Path::new("./hello_world.bin");
    /// let mut dbg = debugger::Debugger::new();
    ///
    /// if let Err(error) = dbg.execute_target(program, &[]) {
    ///    println!("Error: {}", error);
    /// }
    /// ```
    pub fn execute_target(&mut self, program: &Path, args: &[&str]) -> Result<(), Box<Error>> {
        match fork()? {
            ForkResult::Parent { child } => {
                debug!(
                    "Continuing execution in parent process, new child has pid: {}",
                    child
                );
                self.attach_target(child)
            }
            ForkResult::Child => {
                debug!("Executing new child process");

                let program_as_cstring = &CString::new(program.to_str().unwrap()).unwrap();
                ptrace_wrapper::trace_me();
                execve(program_as_cstring, &[], &[]).ok().expect(
                    "execve() operation failed",
                );
                unreachable!();
            }
        }
    }

    /// Attempts to attach the debugger to the running process with the given pid.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use rdbg_core::core::debugger;
    ///
    /// let mut dbg = debugger::Debugger::new();
    ///
    /// if let Err(error) = dbg.attach_target(1000, &[]) {
    ///    println!("Error: {}", error);
    /// }
    /// ```
    pub fn attach_target(&mut self, pid: InferiorPid) -> Result<(), Box<Error>> {
        self.pid = pid;

        match waitpid(pid, None) {
            Ok(WaitStatus::Stopped(pid, signal::SIGTRAP)) => Ok(()),
            Ok(_) => panic!("Unexpected status in run_debugger"),
            Err(_) => panic!("Unhandled error in run_debugger"),
        }
    }

    pub fn continue_execution(&self) -> i8 {
        debug!("Continuing execution...");
        ptrace_wrapper::continue_execution(self.pid);

        match waitpid(self.pid, None) {
            Ok(WaitStatus::Exited(_, code)) => return code,
            Ok(_) => panic!("Unexpected status in continue_execution"),
            Err(_) => panic!("Unhandled error in continue_execution"),
        }
    }

    fn print_rip(&self) {
        let rip = ptrace_wrapper::get_instruction_pointer(self.pid).unwrap();
        println!("RIP: {:?}", format!("{:#x}", rip));
    }

    pub fn set_breakpoint_at(&mut self, address: u64) {
        self.breakpoints.insert(
            address,
            breakpoint::Breakpoint::new(
                self.pid,
                address,
            ),
        );
    }
}
