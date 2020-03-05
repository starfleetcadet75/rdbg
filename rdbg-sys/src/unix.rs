use std::ffi::CString;

use error_chain::bail;
use nix::sys::ptrace;
use nix::sys::signal::Signal;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{execve, fork, ForkResult, Pid};

use crate::errors::*;
use crate::TraceEvent;

/// Attempts to spawn a new process from the given `filename` and attach to it.
pub fn spawn(filename: &str) -> SysResult<i32> {
    let filename = &CString::new(filename).expect("Failed to convert path to CString");

    // To start tracing a new process, fork the debugger and call the `execve()` syscall in
    // the new child. The child is then replaced with the tracee process.
    match fork()? {
        ForkResult::Parent { child } => {
            wait_for_signal(child)?;
            Ok(child.as_raw())
        }
        ForkResult::Child => {
            ptrace::traceme()?;

            // TODO: Send any local arguments to the inferior
            execve(filename, &[], &[])
                .ok()
                .expect("execve() operation failed");
            unreachable!();
        }
    }
}

/// Attempts to attach to a running process with the given `pid`.
pub fn attach(pid: i32) -> SysResult<()> {
    ptrace::attach(Pid::from_raw(pid)).chain_err(|| format!("Failed to attach PID: {}", pid))
}

/// Attempts to detach from the running process with the given `pid`.
pub fn detach(pid: i32) -> SysResult<()> {
    ptrace::detach(Pid::from_raw(pid), None)
        .chain_err(|| format!("Failed to detach from PID: {}", pid))
}

/// Continue a stopped process, as with `ptrace(PTRACE_CONT, ...)`.
pub fn cont(pid: i32) -> SysResult<TraceEvent> {
    let pid = Pid::from_raw(pid);
    ptrace::cont(pid, None)?;
    wait_for_signal(pid)
}

/// Advances the execution of the process with PID `pid` by a single step.
pub fn step(pid: i32) -> SysResult<TraceEvent> {
    let pid = Pid::from_raw(pid);
    ptrace::step(pid, None)?;
    wait_for_signal(pid)
}

/// Arranges for the tracee to be stopped at the next entry or exit from a system call.
pub fn syscall(pid: i32) -> SysResult<()> {
    let pid = Pid::from_raw(pid);
    ptrace::syscall(pid, None).chain_err(|| format!("Failed to call `ptrace::syscall`"))
}

/// Sends a SIGKILL to the tracee and waits for it to stop.
pub fn kill(pid: i32) -> SysResult<TraceEvent> {
    let pid = Pid::from_raw(pid);
    ptrace::cont(pid, Signal::SIGKILL)?;
    wait_for_signal(pid)
}

pub fn read_register(pid: i32, regnum: usize) -> SysResult<()> {
    let pid = Pid::from_raw(pid);
    let regs = ptrace::getregs(pid)?;
    println!("{:?}", regs);
    Ok(())
}

pub fn write_register(pid: i32, regnum: usize, data: u32) -> SysResult<()> {
    let pid = Pid::from_raw(pid);
    unimplemented!()
}

pub fn read_memory(pid: i32, address: usize) -> SysResult<i64> {
    let pid = Pid::from_raw(pid);
    let data = ptrace::read(pid, address as *mut _)
        .chain_err(|| format!("Failed reading from process memory"))?;
    // TODO: c_long = i64
    Ok(data)
}

/// Modifies the memory of a process, as with `ptrace(PTRACE_POKEUSER, ...)`
pub fn write_memory(pid: i32, address: u32, data: u32) -> SysResult<()> {
    let pid = Pid::from_raw(pid);
    ptrace::write(pid, address as *mut _, data as *mut _)
        .chain_err(|| format!("Failed writing to process memory"))
}

fn wait_for_signal(pid: Pid) -> SysResult<TraceEvent> {
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
