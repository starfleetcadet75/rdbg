use nix::sys::ptrace::{self, Request};
use nix::sys::signal::Signal;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{execve, fork, ForkResult, Pid};

use std::ffi::CString;

use core::TraceEvent;
use sys::Word;
use util::errors::*;

pub fn execute(program: &str) -> RdbgResult<Pid> {
    let program_as_cstring = &CString::new(program).expect("Failed to convert path to CString");

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

            ptrace::traceme()?;
            execve(program_as_cstring, &[], &[])
                .ok()
                .expect("execve() operation failed");
            unreachable!();
        }
    }
}

/// Attempts to attach to a running process with the given pid.
pub fn attach(pid: Pid) -> RdbgResult<()> {
    ptrace::attach(pid).chain_err(|| format!("Failed to attach to PID: {}", pid))
}

/// Attempts to detach from the running process with the given pid.
pub fn detach(pid: Pid) -> RdbgResult<()> {
    ptrace::detach(pid).chain_err(|| format!("Failed to detach from PID: {}", pid))
}

/// Restart the stopped tracee process, as with `ptrace(PTRACE_CONT, ...)`.
pub fn continue_execution(pid: Pid) -> RdbgResult<TraceEvent> {
    ptrace::cont(pid, None)?;
    wait_for_signal(pid)
}

/// Move the stopped tracee process forward by a single step as with
/// `ptrace(PTRACE_SINGLESTEP, ...)`
///
/// Advances the execution of the process with PID `pid` by a single step
pub fn single_step(pid: Pid) -> RdbgResult<TraceEvent> {
    ptrace::step(pid, None)?;
    wait_for_signal(pid)
}

/// Ask for next syscall, as with `ptrace(PTRACE_SYSCALL, ...)`
///
/// Arranges for the tracee to be stopped at the next entry to or exit from a system call.
pub fn syscall(pid: Pid) -> RdbgResult<()> {
    ptrace::syscall(pid).chain_err(|| format!("Failed to call `ptrace::syscall`"))
}

/// Sends a SIGKILL to the tracee and waits for it to stop.
pub fn kill(pid: Pid) -> RdbgResult<TraceEvent> {
    ptrace::cont(pid, Signal::SIGKILL)?;
    wait_for_signal(pid)
}

#[allow(deprecated)]
pub fn read_memory(pid: Pid, address: Word) -> RdbgResult<Word> {
    unsafe {
        ptrace::ptrace(
            Request::PTRACE_PEEKDATA,
            pid,
            address as *mut _,
            0 as *mut _,
        ).map(|r| r as Word)
            .and_then(|r| Ok(r))
            .chain_err(|| format!("Failed to read from memory at address {:#x}", address))
    }
}

/// Modifies the memory of a process, as with `ptrace(PTRACE_POKEUSER, ...)`
///
/// A memory chunk of a size of a machine word is overwriten in the requested
/// place in the memory of a process.
///
/// # Safety
///
/// This function allows for accessing arbitrary data in the traced process
/// and may crash the inferior or introduce race conditions if used
/// incorrectly and is thus marked `unsafe`.
#[allow(deprecated)]
pub fn write_memory(pid: Pid, address: Word, data: Word) -> RdbgResult<()> {
    unsafe {
        ptrace::ptrace(
            Request::PTRACE_POKEDATA,
            pid,
            address as *mut _,
            data as *mut _,
        ).map(|_| ())
            .chain_err(|| format!("Failed to write to memory at address {:#x}", address))
    }
}

/// Peeks a user-accessible register, as with `ptrace(PTRACE_PEEKUSER, ...)`.
#[allow(deprecated)]
pub fn read_register(pid: Pid, register: usize) -> RdbgResult<Word> {
    // let reg_arg = (register as i32) as *mut c_void;
    unsafe {
        ptrace::ptrace(
            Request::PTRACE_PEEKUSER,
            pid,
            register as *mut _,
            0 as *mut _,
        ).map(|r| r as Word)
            .and_then(|r| Ok(r))
            .chain_err(|| "Failed to read from register")
    }
}

/// Sets the value of a user-accessible register, as with `ptrace(PTRACE_POKEUSER, ...)`
///
/// # Safety
///
/// When incorrectly used, may change the registers to bad values,
/// causing e.g. memory being corrupted by a syscall, thus is marked unsafe
#[allow(deprecated)]
pub fn write_register(pid: Pid, register: usize, data: Word) -> RdbgResult<()> {
    unsafe {
        ptrace::ptrace(
            Request::PTRACE_POKEUSER,
            pid,
            register as *mut _,
            data as *mut _,
        ).map(|_| ())
            .chain_err(|| "Failed to write to register")
    }
}

fn wait_for_signal(pid: Pid) -> RdbgResult<TraceEvent> {
    // The `waitpid()` function is used to wait on and obtain status information from child processes.
    // Each status (other than `StillAlive`) describes a state transition in a child process `Pid`,
    // such as the process exiting or stopping, plus additional data about the transition if any.
    match waitpid(pid, None) {
        // The process exited normally (as with `exit()` or returning from
        // `main`) with the given exit code. This case matches the C macro
        // `WIFEXITED(status)`; the second field is `WEXITSTATUS(status)`.
        Ok(WaitStatus::Exited(_, status)) => Ok(TraceEvent::Exit(status)),
        // The process was killed by the given signal. The third field
        // indicates whether the signal generated a core dump. This case
        // matches the C macro `WIFSIGNALED(status)`; the last two fields
        // correspond to `WTERMSIG(status)` and `WCOREDUMP(status)`.
        Ok(WaitStatus::Signaled(_, signal, coredump)) => {
            Ok(TraceEvent::Killed(signal as u8, coredump))
        }
        // The process is alive, but was stopped by the given signal. This
        // is only reported if `WaitPidFlag::WUNTRACED` was passed. This
        // case matches the C macro `WIFSTOPPED(status)`; the second field
        // is `WSTOPSIG(status)`.
        // Ok(WaitStatus::Stopped(_, signal)) => Ok(TraceEvent::Signal(signal as u8)),
        Ok(WaitStatus::Stopped(_, signal)) => match signal {
            Signal::SIGTRAP => Ok(TraceEvent::Breakpoint),
            _ => Ok(TraceEvent::Signal(signal as u8)),
        },
        // The traced process was stopped by a `PTRACE_EVENT_*` event. See
        // [`ptrace`(2)] for more information. All currently-defined events
        // use `SIGTRAP` as the signal; the third field is the `PTRACE_EVENT_*`
        // value of the event.
        //
        // [`ptrace`(2)]: http://man7.org/linux/man-pages/man2/ptrace.2.html
        Ok(WaitStatus::PtraceEvent(_, _, code)) => Ok(TraceEvent::Event(code as u8)),
        // The traced process was stopped by execution of a system call,
        // and `PTRACE_O_TRACESYSGOOD` is in effect. See [`ptrace`(2)] for
        // more information.
        //
        // [`ptrace`(2)]: http://man7.org/linux/man-pages/man2/ptrace.2.html
        Ok(WaitStatus::PtraceSyscall(_)) => Ok(TraceEvent::SyscallEnter),
        // The process was previously stopped but has resumed execution
        // after receiving a `SIGCONT` signal. This is only reported if
        // `WaitPidFlag::WCONTINUED` was passed. This case matches the C
        // macro `WIFCONTINUED(status)`.
        Ok(WaitStatus::Continued(_)) => Ok(TraceEvent::Continued),
        Ok(status) => bail!("Unexpected wait status {:?}", status),
        Err(_) => bail!("Unexpected wait status"),
    }
}
