use log::{debug, info};

use crate::core::program::Program;
use crate::util::errors::{RdbgResult, RdbgResultExt};
use rdbg_sys as sys;

pub struct Debugger {
    pub pid: i32,
    program: Option<Program>,
}

impl Debugger {
    pub fn new() -> Self {
        Debugger {
            pid: 0,
            program: None,
        }
    }

    pub fn load(&mut self, program: String) -> RdbgResult<()> {
        self.program = Program::open(program).ok();
        Ok(())
    }

    /// Spawns a new process from the given program's `filename`
    /// with the debugger attached.
    pub fn spawn(&mut self) -> RdbgResult<()> {
        if let Some(ref program) = self.program {
            debug!("Spawning new process: {}", program.path);
            self.pid = sys::spawn(&program.path)?;
            debug!("New process has PID: {}", self.pid);
        } else {
            info!("No program currently loaded");
        }
        Ok(())
    }

    /// Attempts to attach the debugger to the process with process identifier `pid`.
    pub fn attach(&mut self, pid: i32) -> RdbgResult<()> {
        debug!("Attaching to process with PID: {}", pid);
        sys::attach(pid)?;
        self.pid = pid;
        Ok(())
    }

    /// Detaches the debugger from the process it is tracing.
    pub fn detach(&self) -> RdbgResult<()> {
        debug!("Detaching from process");
        sys::detach(self.pid).chain_err(|| format!("Failed to detach from PID: {}", self.pid))
    }

    /// Kills the currently traced process.
    pub fn kill(&self) -> RdbgResult<()> {
        match sys::kill(self.pid)? {
            sys::TraceEvent::Killed(signum, _) => {
                debug!("Inferior killed by signal {}", signum);
            }
            _ => debug!("Received unexpected event"),
        }
        Ok(())
    }

    pub fn cont() {
        unimplemented!()
    }

    pub fn step() {
        unimplemented!()
    }

    // step into
    // step over

    pub fn set_breakpoint() {
        unimplemented!()
    }
    // delete breakpoint
    // enable breakpoint
    // disable breakpoint

    pub fn read_register(&self, register: &str) -> RdbgResult<()> {
        debug!("Reading from register: {:?}", register);
        Ok(())
    }

    pub fn write_register(&self, register: &str, data: u32) -> RdbgResult<()> {
        debug!("Writing {} into register: {:?}", data, register);
        Ok(())
    }

    pub fn read_memory() {
        unimplemented!()
    }

    pub fn write_memory() {
        unimplemented!()
    }
}
