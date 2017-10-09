//! OS specific stub for Linux systems.

use libc;
use libc::c_void;
use nix::sys::ptrace;
use nix::sys::ptrace::ptrace::*;
use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::{ForkResult, execve, fork};

use std::ffi::CString;
use std::ptr;

use super::super::{Address, Pid};
use super::super::core::debugger_state::DebuggerState;
use super::super::util::error::{RdbgError, RdbgResult};

pub fn execute_target(path: &str) -> RdbgResult<(Pid, DebuggerState)> {
    let program_as_cstring = &CString::new(path).expect("failed to convert path to CString");

    match fork()? {
        ForkResult::Parent { child } => {
            debug!(
                "Continuing execution in parent process, new child has pid: {}",
                child
            );
            match wait_for_signal(child) {
                Ok(state) => Ok((child, state)),
                Err(_) => Err(RdbgError::NixError),
            }
        }
        ForkResult::Child => {
            debug!("Executing new child process");

            ptrace::traceme().ok();
            execve(program_as_cstring, &[], &[]).ok().expect(
                "execve() operation failed",
            );
            unreachable!();
        }
    }
}

pub fn attach_target(pid: Pid) -> RdbgResult<()> {
    match ptrace::attach(pid) {
        Ok(_) => Ok(()),
        Err(_) => Err(RdbgError::NixError),
    }
}

#[allow(deprecated)]
pub fn continue_execution(pid: Pid) -> RdbgResult<(DebuggerState)> {
    ptrace::cont(pid, None)?;
    wait_for_signal(pid)
}

#[allow(deprecated)]
pub fn single_step_instruction(pid: Pid) -> RdbgResult<(DebuggerState)> {
    unsafe {
        ptrace::ptrace(PTRACE_SINGLESTEP, pid, ptr::null_mut(), ptr::null_mut())?;
        wait_for_signal(pid)
    }
}

#[allow(deprecated)]
pub fn read_memory(pid: Pid, address: Address) -> RdbgResult<i64> {
    unsafe {
        match ptrace::ptrace(
            PTRACE_PEEKDATA,
            pid,
            address as *mut c_void,
            ptr::null_mut(),
        ) {
            Ok(data) => Ok(data),
            Err(_) => Err(RdbgError::NixError),
        }
    }
}

#[allow(deprecated)]
pub fn write_memory(pid: Pid, address: Address, data: i64) -> RdbgResult<()> {
    unsafe {
        match ptrace::ptrace(
            PTRACE_POKEDATA,
            pid,
            address as *mut c_void,
            data as *mut c_void,
        ) {
            Ok(_) => Ok(()),
            Err(_) => Err(RdbgError::NixError),
        }
    }
}

#[allow(deprecated)]
fn wait_for_signal(pid: Pid) -> RdbgResult<(DebuggerState)> {
    match waitpid(pid, None) {
        Ok(WaitStatus::Exited(_, code)) => {
            info!("WaitStatus: Exited with status: {}", code);
            println!("[Inferior (process {}) exited with status {}]", pid, code);
            Ok((DebuggerState::Exited))
        }
        Ok(WaitStatus::Signaled(_, signal, core_dump)) => {
            info!(
                "WaitStatus: Process killed by signal: {:?}, core dumped?: {}",
                signal,
                core_dump
            );
            Ok((DebuggerState::Exited))
        }
        Ok(WaitStatus::Stopped(_, _)) => {
            match ptrace::getsiginfo(pid) {
                Ok(siginfo) if siginfo.si_signo == libc::SIGTRAP => {
                    debug!("Recieved SIGTRAP");
                    debug!("si_code: {:?}", siginfo.si_code);

                    if siginfo.si_code == 0x80 {
                        Ok((DebuggerState::Breakpoint))
                    } else {
                        Ok((DebuggerState::Running))
                    }
                }
                Ok(siginfo) if siginfo.si_signo == libc::SIGSEGV => {
                    info!("Recieved SIGSEGV, reason: {}", siginfo.si_code);
                    Ok((DebuggerState::Exited))
                }
                Ok(siginfo) => {
                    debug!("Recieved {}", siginfo.si_signo);
                    Ok((DebuggerState::Running))
                }
                Err(_) => Err(RdbgError::NixError),
            }
        }
        Ok(WaitStatus::Continued(_)) => {
            info!("WaitStatus: Continued");
            Ok((DebuggerState::Running))
        }
        Ok(_) => panic!("Unknown waitstatus"),
        Err(_) => Err(RdbgError::NixError),
    }
}
