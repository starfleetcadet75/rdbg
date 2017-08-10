use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{execve, fork, ForkResult, Pid};

use std::ffi::CString;
use std::error::Error;
use std::path::Path;
use std::collections::HashMap;

use super::super::InferiorPid;
use super::super::breakpoint::breakpoint;
use super::ptrace_wrapper;

pub struct Debugger {
    pid: InferiorPid,
    breakpoints: HashMap<u64, breakpoint::Breakpoint>,
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger {
            pid: InferiorPid::from_raw(0),
            breakpoints: HashMap::new(),
        }
    }

    // Starts debugging the target program given by the program path,
    // uses any args specified.
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

    // Attempts to attach the debugger to the running process with the given pid.
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

    fn set_breakpoint_at(&self, address: u64) {}
}
