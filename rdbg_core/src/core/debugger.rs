//! The main `Debugger` module.
//! This module contains the main interface for the core functionality.

use fnv::FnvHashMap;
use goblin;
use goblin::Object;
use goblin::elf;
use goblin::error;
use libc;
use libc::c_void;
use nix::sys::ptrace;
use nix::sys::ptrace::ptrace::*;
use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::{ForkResult, execve, fork};

use std::ffi::CString;
use std::ptr;

use super::arch::Arch;
use super::debugger_state::DebuggerState;
use super::program::Program;
use super::super::{Address, Pid};
use super::super::breakpoint::breakpoint;
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
    #[allow(deprecated)]
    pub fn execute_target(&mut self) -> RdbgResult<()> {
        if let DebuggerState::ExecLoaded = self.state {
            let op_program = self.program.take();
            let program = op_program.unwrap();
            let path = program.path.clone();
            let path = path.to_str().unwrap();
            let program_as_cstring = &CString::new(path).unwrap();
            self.program = Some(program);

            match fork()? {
                ForkResult::Parent { child } => {
                    debug!(
                        "Continuing execution in parent process, new child has pid: {}",
                        child
                    );
                    self.pid = child;
                    self.state = DebuggerState::Running;
                    self.wait_for_signal()
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
    #[allow(deprecated)]
    pub fn attach_target(&mut self, pid: Pid) -> RdbgResult<()> {
        self.pid = pid;
        match ptrace::attach(pid) {
            Ok(_) => Ok(()),
            Err(_) => Err(RdbgError::NixError),
        }
    }

    pub fn load_program(&mut self, program: Program) -> RdbgResult<()> {
        info!("Loading program: {:?}", program.path);
        if let DebuggerState::Running = self.state {
            error!(
                "Failed to load new program, tracee must be stopped before a new program can be loaded."
            );
        } else {
            self.program = Some(program);
            self.state = DebuggerState::ExecLoaded;
        }
        Ok(())
    }

    pub fn get_entrypoint(&mut self) -> RdbgResult<Address> {
        match self.state {
            DebuggerState::ExecLoaded |
            DebuggerState::Running => {
                if let Some(ref program) = self.program {
                    match goblin::elf::Elf::parse(&program.buffer) {
                        Ok(binary) => Ok(binary.entry),
                        Err(_) => Err(RdbgError::GoblinError),
                    }
                } else {
                    Err(RdbgError::GoblinError)
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

    #[allow(deprecated)]
    pub fn continue_execution(&mut self) -> RdbgResult<()> {
        if let DebuggerState::Running = self.state {
            let pc = &self.get_pc()?;
            if self.breakpoints.contains_key(pc) {
                self.step_over_breakpoint()?;
            }
            ptrace::cont(self.pid, None)?;
            self.wait_for_signal()
        } else {
            Err(RdbgError::NoProgramLoaded)
        }
    }

    /// Reads a word from the process memory at the given address.
    #[allow(deprecated)]
    pub fn read_memory(&self, address: Address) -> RdbgResult<i64> {
        if let DebuggerState::Running = self.state {
            unsafe {
                match ptrace::ptrace(
                    PTRACE_PEEKDATA,
                    self.pid,
                    address as *mut c_void,
                    ptr::null_mut(),
                ) {
                    Ok(data) => Ok(data),
                    Err(_) => Err(RdbgError::NixError),
                }
            }
        } else {
            Err(RdbgError::NoProgramLoaded)
        }
    }

    /// Writes a word with the given value to the process memory
    /// at the given address.
    #[allow(deprecated)]
    pub fn write_memory(&self, address: Address, data: i64) -> RdbgResult<()> {
        if let DebuggerState::Running = self.state {
            unsafe {
                match ptrace::ptrace(
                    PTRACE_POKEDATA,
                    self.pid,
                    address as *mut c_void,
                    data as *mut c_void,
                ) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(RdbgError::NixError),
                }
            }
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

    #[allow(deprecated)]
    fn single_step_instruction(&mut self) -> RdbgResult<()> {
        unsafe {
            ptrace::ptrace(
                PTRACE_SINGLESTEP,
                self.pid,
                ptr::null_mut(),
                ptr::null_mut(),
            )?;
            self.wait_for_signal()?;
            Ok(())
        }
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

    #[allow(deprecated)]
    fn wait_for_signal(&mut self) -> RdbgResult<()> {
        match waitpid(self.pid, None) {
            Ok(WaitStatus::Exited(_, code)) => {
                self.state = DebuggerState::Exited;
                info!("WaitStatus: Exited with status: {}", code);
                println!(
                    "[Inferior (process {}) exited with status {}]",
                    self.pid,
                    code
                );
                Ok(())
            }
            Ok(WaitStatus::Signaled(_, signal, core_dump)) => {
                self.state = DebuggerState::Exited;
                info!(
                    "WaitStatus: Process killed by signal: {:?}, core dumped?: {}",
                    signal,
                    core_dump
                );
                Ok(())
            }
            Ok(WaitStatus::Stopped(_, _)) => {
                match ptrace::getsiginfo(self.pid) {
                    Ok(siginfo) if siginfo.si_signo == libc::SIGTRAP => {
                        debug!("Recieved SIGTRAP");
                        self.handle_sigtrap(siginfo);
                        Ok(())
                    }
                    Ok(siginfo) if siginfo.si_signo == libc::SIGSEGV => {
                        debug!("Recieved SIGSEGV, reason: {}", siginfo.si_code);
                        Ok(())
                    }
                    Ok(siginfo) => {
                        debug!("Recieved {}", siginfo.si_signo);
                        Ok(())
                    }
                    Err(_) => Err(RdbgError::NixError),
                }
            }
            Ok(WaitStatus::Continued(_)) => {
                info!("WaitStatus: Continued");
                Ok(())
            }
            Ok(_) => panic!("Unknown waitstatus"),
            Err(_) => Err(RdbgError::NixError),
        }
    }

    // TODO: nix/libc does not currently seem to support the values SI_KERNEL,
    // TRAP_BRKPT, and TRAP_TRACE which are defined here '/usr/include/bits/siginfo.h'
    // in libc for Linux. These are needed in order to handle the codes that come with
    // a SIGTRAP signal. For now, 0x80 seems correct for handling breakpoints and 0x2
    // seems to be the value for TRAP_TRACE, which does not require any handling.
    fn handle_sigtrap(&self, siginfo: libc::siginfo_t) {
        debug!("si_code: {:?}", siginfo.si_code);

        if siginfo.si_code == 0x80 {
            self.set_pc(self.get_pc().unwrap() - 1); // move the pc back one instruction
            info!("Hit breakpoint at address {:#x}", self.get_pc().unwrap());
        }
    }
}
