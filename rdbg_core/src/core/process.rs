//! A `Process` acts as a layer between the `Debugger` and the OS specific stubs.
//! It abstracts away the tracking of the `Pid` and any `ProcessEvent`s.

use nix::sys::signal::Signal;

use {Address, Pid};
use core::profile::Profile;
use stubs::linux;
use util::error::RdbgResult;

/// Process events are used by the `Debugger` for
/// tracking the current state of a traced process.
#[derive(Debug)]
pub enum ProcessEvent {
    Continued,
    Breakpoint,
    Stopped(bool),
    Signaled(Signal),
    Exited(Pid, i8),
}

/// A `Process` traced by the `Debugger`.
#[derive(Debug)]
pub struct Process {
    /// The Pid of the traced process.
    pid: Pid,
    /// The last ProcessEvent received.
    /// This is used by the Debugger for tracking
    /// the current state of the traced process.
    last_event: ProcessEvent,
}

impl Process {
    /// Starts a new `Process` using the given `Profile`.
    ///
    /// # Arguments
    ///
    /// * `profile` - A `Profile` object that provides the path to the program
    ///               to be traced along with other specific environment properties.
    pub fn new(profile: &Profile) -> RdbgResult<Process> {
        debug!("Creating new process");

        let path = profile.program_path.to_str().expect(
            "failed to convert path to string",
        );
        let pid = linux::execute(path)?;

        Ok(Process {
            pid: pid,
            last_event: ProcessEvent::Stopped(false),
        })
    }

    pub fn attach(pid: i32) -> RdbgResult<Process> {
        debug!("Attaching to process");

        let pid = Pid::from_raw(pid);
        linux::attach(pid)?;

        Ok(Process {
            pid: pid,
            last_event: ProcessEvent::Stopped(false),
        })
    }

    pub fn continue_execution(&mut self) -> RdbgResult<()> {
        debug!("Continuing execution of process");

        match linux::continue_execution(self.pid) {
            Ok(event) => {
                // Save the event received by continue so that the Debugger can handle it
                self.last_event = event;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub fn read_memory(&self, address: Address) -> RdbgResult<i64> {
        linux::read_memory(self.pid, address)
    }

    pub fn write_memory(&self, address: Address, data: i64) -> RdbgResult<()> {
        linux::write_memory(self.pid, address, data)
    }
}
