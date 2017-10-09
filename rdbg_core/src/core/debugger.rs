//! The main `Debugger` module.
//! This module contains the main interface for the core functionality.

use fnv::FnvHashMap;

use super::arch::Arch;
use super::debugger_state::DebuggerState;
use super::program::Program;
use super::super::{Address, Pid};
use super::super::breakpoint::breakpoint;
use super::super::stubs::linux;
use super::super::util::error::{RdbgError, RdbgResult};

pub struct Debugger {
    pub pid: Pid,
    state: DebuggerState,
    program: Option<Program>,
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
            state: DebuggerState::Empty,
            program: None,
            breakpoints: FnvHashMap::default(),
        }
    }

    pub fn load_program(&mut self, program: Program) -> RdbgResult<()> {
        info!("Loading program: {:?}", program.path);
        if let DebuggerState::Running = self.state {
            error!(
                "Failed to load new program, tracee must be stopped before a new program can be loaded."
            );
        } else {
            // TODO: Use a loader with ELF data
            self.program = Some(program);
            self.state = DebuggerState::ExecLoaded;
        }
        Ok(())
    }

    /// Starts debugging of the target given by the program path.
    /// Passes any arguments given as parameters as arguments to the new program.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use rdbg_core::core::debugger;
    /// use rdbg_core::core::program;
    ///
    /// let path = "./hello_world.bin";
    /// let program = program::Program::new(&PathBuf::from(path));
    ///
    /// let mut dbg = debugger::Debugger::new();
    /// dbg.load_program(program);
    ///
    /// if let Err(error) = dbg.execute_target() {
    ///    println!("Error: {}", error);
    /// }
    /// ```
    pub fn execute_target(&mut self) -> RdbgResult<()> {
        // Check if a program is loaded before trying to run
        if let Some(ref prog) = self.program {
            let path = prog.path.to_str().expect(
                "failed to convert path to string",
            );

            let (pid, state) = linux::execute_target(path)?;
            self.pid = pid;
            self.state = state;
            Ok(())
        } else {
            Err(RdbgError::NoProgramLoaded)
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
    /// use rdbg_core::Pid;
    ///
    /// let mut dbg = debugger::Debugger::new();
    ///
    /// if let Err(error) = dbg.attach_target(Pid::from_raw(1000)) {
    ///    println!("Error: {}", error);
    /// }
    /// ```
    pub fn attach_target(&mut self, pid: Pid) -> RdbgResult<()> {
        self.pid = pid;
        linux::attach_target(self.pid)
    }

    pub fn get_entrypoint(&mut self) -> RdbgResult<Address> {
        match self.state {
            DebuggerState::ExecLoaded |
            DebuggerState::Running => {
                if let Some(ref program) = self.program {
                    Ok(program.entry)
                } else {
                    Err(RdbgError::NoProgramLoaded)
                }
            }
            _ => Err(RdbgError::NoProgramLoaded),
        }
    }

    pub fn procinfo(&mut self) -> RdbgResult<()> {
        match self.state {
            DebuggerState::ExecLoaded |
            DebuggerState::Running => {
                if let Some(ref prog) = self.program {
                    println!(
                        "status = {:?}\nexe = {:?}\nargs = {:?}\npid = {:?}",
                        self.state,
                        prog.path,
                        prog.args,
                        self.pid
                    );
                }
                Ok(())
            }
            _ => Err(RdbgError::NoProgramLoaded),
        }
    }

    pub fn continue_execution(&mut self) -> RdbgResult<()> {
        if let DebuggerState::Running = self.state {
            let pc = &self.get_pc()?;
            if self.breakpoints.contains_key(pc) {
                self.step_over_breakpoint()?;
            }

            self.state = linux::continue_execution(self.pid)?;
            if let DebuggerState::Breakpoint = self.state {
                self.set_pc(self.get_pc().unwrap() - 1); // move the pc back one instruction
                info!("Hit breakpoint at address {:#x}", self.get_pc().unwrap());
            }

            Ok(())
        } else {
            Err(RdbgError::NoProgramLoaded)
        }
    }

    /// Reads a word from the process memory at the given address.
    pub fn read_memory(&self, address: Address) -> RdbgResult<i64> {
        if let DebuggerState::Running = self.state {
            linux::read_memory(self.pid, address)
        } else {
            Err(RdbgError::NoProgramLoaded)
        }
    }

    /// Writes a word with the given value to the process memory
    /// at the given address.
    pub fn write_memory(&self, address: Address, data: i64) -> RdbgResult<()> {
        if let DebuggerState::Running = self.state {
            linux::write_memory(self.pid, address, data)
        } else {
            Err(RdbgError::NoProgramLoaded)
        }
    }

    pub fn print_breakpoints(&self) {
        let mut count = 1;
        for (address, breakpoint) in &self.breakpoints {
            println!(
                "Breakpoint {} is at {:#x}, enabled = {}",
                count,
                address,
                breakpoint.is_enabled()
            );
            count += 1;
        }
    }

    pub fn set_breakpoint_at(&mut self, address: Address) -> RdbgResult<()> {
        if let DebuggerState::Running = self.state {
            println!(
                "Breakpoint {} at {:#x}",
                self.breakpoints.len() + 1,
                address
            );
            self.breakpoints.insert(
                address,
                breakpoint::Breakpoint::new(
                    self.pid,
                    address,
                ),
            );
            Ok(())
        } else {
            Err(RdbgError::NoProgramLoaded)
        }
    }

    pub fn remove_breakpoint(&mut self, address: Address) -> RdbgResult<()> {
        if let DebuggerState::Running = self.state {
            if self.breakpoints.contains_key(&address) {
                self.breakpoints.remove(&address);
                info!("Removed breakpoint at {:#x}", address);
            } else {
                info!("No breakpoint found at {:#x}", address);
            }
            Ok(())
        } else {
            Err(RdbgError::NoProgramLoaded)
        }
    }

    pub fn enable_breakpoint(&mut self, address: Address) -> RdbgResult<()> {
        if self.breakpoints.contains_key(&address) {
            let mut bp = *self.breakpoints.get_mut(&address).unwrap();
            if !bp.is_enabled() {
                bp.enable()?;
            }
        } else {
            println!("No breakpoint at address {:#x}", address)
        }
        Ok(())
    }

    pub fn disable_breakpoint(&mut self, address: Address) -> RdbgResult<()> {
        if self.breakpoints.contains_key(&address) {
            let mut bp = *self.breakpoints.get_mut(&address).unwrap();
            if bp.is_enabled() {
                bp.disable()?; // TODO: disable does not seem to actually modify the value
            }
        } else {
            println!("No breakpoint at address {:#x}", address)
        }
        Ok(())
    }

    fn single_step_instruction(&mut self) -> RdbgResult<()> {
        self.state = linux::single_step_instruction(self.pid)?;
        if let DebuggerState::Breakpoint = self.state {
            self.set_pc(self.get_pc().unwrap() - 1); // move the pc back one instruction
            info!("Hit breakpoint at address {:#x}", self.get_pc().unwrap());
        }
        Ok(())
    }

    pub fn single_step_instruction_with_breakpoints(&mut self) -> RdbgResult<()> {
        if let DebuggerState::Running = self.state {
            let pc = &self.get_pc().unwrap();
            if self.breakpoints.contains_key(pc) {
                self.step_over_breakpoint()
            } else {
                self.single_step_instruction()
            }
        } else {
            Err(RdbgError::NoProgramLoaded)
        }
    }

    fn step_over_breakpoint(&mut self) -> RdbgResult<()> {
        let pc = &self.get_pc().unwrap();
        // safe unwrap, checks are performed outside fn
        let mut bp = *self.breakpoints.get_mut(pc).unwrap();

        if bp.is_enabled() {
            bp.disable()?; // disable the breakpoint to step over it
            self.single_step_instruction()?;
            bp.enable()?;
        }
        Ok(())
    }
}
