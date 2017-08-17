//! The main `Debugger` module.
//! This module contains the main interface for the core functionality.
use nix;
use nix::sys::signal;
use nix::sys::ptrace;
use nix::sys::ptrace::ptrace::*;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{execve, fork, ForkResult};
use libc;
use libc::c_void;

use std::ptr;
use std::path::Path;
use std::error::Error;
use std::ffi::CString;
use fnv::FnvHashMap;

use super::arch::Arch;
use super::super::{Pid, Address};
use super::super::breakpoint::breakpoint;

pub struct Debugger {
    pub pid: Pid,
    breakpoints: FnvHashMap<Address, breakpoint::Breakpoint>,
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
            pid: Pid::from_raw(0),
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
                ptrace::traceme().ok();
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
    pub fn attach_target(&mut self, pid: Pid) -> Result<(), Box<Error>> {
        self.pid = pid;

        match waitpid(pid, None) {
            Ok(WaitStatus::Stopped(_, signal::SIGTRAP)) => Ok(()),
            Ok(_) => panic!("Unexpected status in run_debugger"),
            Err(_) => panic!("Unhandled error in run_debugger"),
        }
    }

    pub fn continue_execution(&mut self) {
        debug!("Continuing execution...");
        //self.step_over_breakpoint();
        ptrace::cont(self.pid, None).ok();
        self.wait_for_signal().ok();
    }

    /// Reads a word from the process memory at the given address.
    pub fn read_memory(&self, address: Address) -> nix::Result<i64> {
        unsafe {
            ptrace::ptrace(PTRACE_PEEKDATA, self.pid, address as *mut c_void, ptr::null_mut())
        }
    }

    /// Writes a word with the given value to the process memory
    /// at the given address.
    pub fn write_memory(&self, address: Address, data: i64) {
        unsafe {
            ptrace::ptrace(PTRACE_POKEDATA, self.pid, address as *mut c_void, data as *mut c_void).ok();
        }
    }

    pub fn set_breakpoint_at(&mut self, address: Address) {
        self.breakpoints.insert(
            address,
            breakpoint::Breakpoint::new(
                self.pid,
                address,
            ),
        );
    }

    pub fn remove_breakpoint(&mut self, address: Address) {
        self.breakpoints.remove(&address);
    }

    pub fn single_step_instruction(&self) {
        unsafe {
            ptrace::ptrace(PTRACE_SINGLESTEP, self.pid, ptr::null_mut(), ptr::null_mut()).ok();
        }
        self.wait_for_signal().ok();
    }

    //fn step_over_breakpoint(&mut self) {
    //    let possible_bp_location = self.get_pc() - 1;

    //    // check if a breakpoint exists at the current instruction
    //    if self.breakpoints.contains_key(&possible_bp_location) {
    //        let mut bp = self.breakpoints.entry(possible_bp_location);
    //        if bp.is_enabled() {
    //            let prev_instr_address = possible_bp_location;
    //            self.set_pc(prev_instr_address);

    //            // disable the breakpoint to step over it
    //            bp.disable();
    //            self.single_step_instruction();
    //            bp.enable();
    //        }
    //    }
    //}

    fn wait_for_signal(&self) -> Result<(), Box<Error>> {
        match waitpid(self.pid, None) {
            Ok(WaitStatus::Exited(_, code)) => {
                info!("WaitStatus: Exited with status: {}", code);
                Ok(())
            },
            Ok(WaitStatus::Signaled(_, signal, core_dump)) => {
                info!("WaitStatus: Process killed by signal: {:?}, core dumped?: {}", signal, core_dump);
                Ok(())
            },
            Ok(WaitStatus::Stopped(_, _)) => {
                match ptrace::getsiginfo(self.pid) {
                    Ok(siginfo) if siginfo.si_signo == libc::SIGTRAP => {
                        debug!("Recieved SIGTRAP");
                        Ok(())
                    },
                    Ok(siginfo) if siginfo.si_signo == libc::SIGSEGV => {
                        debug!("Recieved SIGSEGV, reason: {}", siginfo.si_code);
                        Ok(())
                    },
                    Ok(siginfo) => {
                        debug!("Recieved {}", siginfo.si_signo);
                        Ok(())
                    },
                    Err(_) => panic!("Error getting signal"),
                }
            },
            Ok(WaitStatus::Continued(_)) => {
                info!("WaitStatus: Continued");
                Ok(())
            },
            Ok(_) => panic!("Unknown waitstatus"),
            Err(_) => panic!("Unhandled error in wait_for_signal()"),
        }
    }
}
