use super::super::InferiorPid;

use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{execve, fork, ForkResult, Pid};

use std::ffi::CString;
use std::error::Error;
use std::path::Path;

use super::ptrace_wrapper;

pub struct Debugger {
    pid: InferiorPid,
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger {
            pid: InferiorPid::from_raw(0),
        }
    }

    // Starts debugging the target program given by the program path,
    // uses any args specified.
    pub fn execute_target(&mut self, program: &Path, args: &[&str]) -> Result<(), Box<Error>> {
        match fork()? {
            ForkResult::Parent { child } => {
                debug!("Continuing execution in parent process, new child has pid: {}", child);
                self.attach_target(child)
            }
            ForkResult::Child => {
                debug!("Executing new child process");

                let program_as_cstring = &CString::new(program.to_str().unwrap()).unwrap();
                ptrace_wrapper::trace_me();
                execve(program_as_cstring, &[], &[])
                    .ok()
                    .expect("execve() operation failed");
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
}

