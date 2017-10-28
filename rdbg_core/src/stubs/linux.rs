//! OS specific stub for Linux systems using the `ptrace()` syscall.
//!
//! Refer to https://github.com/nix-rust/nix/blob/master/src/sys/ptrace.rs for Nix ptrace usage.
//! For detailed description of the ptrace requests, consult `man ptrace`.

use libc;
use libc::c_void;
use libc::siginfo_t;
use nix::sys::ptrace;
use nix::sys::ptrace::ptrace::*;
use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::{ForkResult, execve, fork};

use std::ffi::CString;
use std::ptr;

use {Address, Pid};
use core::process::ProcessEvent;
use util::error::{RdbgError, RdbgResult};

pub fn execute(path: &str) -> RdbgResult<Pid> {
    let program_as_cstring = &CString::new(path).expect("failed to convert path to CString");

    // To start tracing a new process, fork the debugger and call the `execve()` syscall in
    // the new child. The child is then replaced with the tracee process.
    match fork()? {
        ForkResult::Parent { child } => {
            debug!(
                "Continuing execution in parent process, new child has pid: {}",
                child
            );
            wait_for_signal(child)?;
            Ok(child)
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

pub fn attach(pid: Pid) -> RdbgResult<()> {
    match ptrace::attach(pid) {
        Ok(_) => Ok(()),
        Err(_) => Err(RdbgError::NixError),
    }
}

#[allow(deprecated)]
pub fn continue_execution(pid: Pid) -> RdbgResult<ProcessEvent> {
    ptrace::cont(pid, None)?;
    wait_for_signal(pid)
}

#[allow(deprecated)]
pub fn single_step_instruction(pid: Pid) -> RdbgResult<ProcessEvent> {
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
fn wait_for_signal(pid: Pid) -> RdbgResult<ProcessEvent> {
    // The `waitpid()` function is used to wait on and obtain status information from child processes.
    // Each status (other than `StillAlive`) describes a state transition
    // in a child process `Pid`, such as the process exiting or stopping,
    // plus additional data about the transition if any.
    match waitpid(pid, None) {
        Ok(WaitStatus::Exited(_, code)) => {
            debug!("WaitStatus: Exited with status: {}", code);
            println!("[Inferior (process {}) exited with status {}]", pid, code);

            Ok(ProcessEvent::Exited(pid, code))
        }
        Ok(WaitStatus::Signaled(_, signal, core_dump)) => {
            debug!(
                "WaitStatus: Process killed by signal: {:?}, core dumped?: {}",
                signal,
                core_dump
            );
            Ok(ProcessEvent::Signaled(signal))
        }
        Ok(WaitStatus::Stopped(_, _)) => {
            match ptrace::getsiginfo(pid) {
                Ok(siginfo) => handle_siginfo(siginfo),
                Err(_) => Err(RdbgError::NixError),
            }
        }
        // TODO: Check if there is a WPTRACEEVENT macro to handle
        Ok(WaitStatus::Continued(_)) => {
            debug!("WaitStatus: Continued");
            Ok(ProcessEvent::Continued)
        }
        Ok(_) => panic!("Unknown waitstatus"),
        Err(_) => Err(RdbgError::NixError),
    }
}

fn handle_siginfo(siginfo: siginfo_t) -> RdbgResult<ProcessEvent> {
    match siginfo.si_signo {
        libc::SIGTRAP => {
            info!("Recieved SIGTRAP\nsi_code: {:?}", siginfo.si_code);

            if siginfo.si_code == 0x80 {
                Ok(ProcessEvent::Breakpoint)
            } else {
                Ok(ProcessEvent::Stopped(false))
            }
        }
        libc::SIGSEGV => {
            info!("Recieved SIGSEGV, reason: {}", siginfo.si_code);
            Ok(ProcessEvent::Stopped(true))
        }
        libc::SIGBUS => {
            info!("Recieved SIGBUS, memory fault");
            Ok(ProcessEvent::Stopped(true))
        }
        libc::SIGFPE => {
            info!("Recieved SIGFPE, math error");
            Ok(ProcessEvent::Stopped(true))
        }
        libc::SIGCHLD => {
            info!("Recieved SIGCHLD, child exited");
            Ok(ProcessEvent::Stopped(true))
        }
        libc::SIGABRT => {
            info!("Recieved SIGABRT, aborted");
            Ok(ProcessEvent::Stopped(true))
        }
        _ => {
            debug!("Recieved {}", siginfo.si_signo);
            Ok(ProcessEvent::Continued)
        }
    }
}