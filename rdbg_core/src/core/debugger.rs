//! The main `Debugger` module.
//! This module contains the main interface for the core functionality.
use nix::sys::ptrace;
use nix::sys::ptrace::ptrace::*;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{execve, fork, ForkResult};
use libc;
use libc::c_void;

use std::ptr;
use std::ffi::CString;
use fnv::FnvHashMap;

use super::arch::Arch;
use super::program::Program;
use super::super::{Pid, Address};
use super::super::breakpoint::breakpoint;
use super::super::util::error::{RdbgResult, RdbgError};

pub struct Debugger {
    pub pid: Pid,
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
        match self.program {
            Some(ref prog) => {
                let path = prog.path.to_str().unwrap();
                let program_as_cstring = &CString::new(path).unwrap();

                match fork()? {
                    ForkResult::Parent { child } => {
                        debug!(
                            "Continuing execution in parent process, new child has pid: {}",
                            child
                        );
                        self.pid = child;
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
            }
            None => panic!("There is no file loaded."),
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
        self.program = Some(program);
        // TODO: load ELF and DWARF info here
        Ok(())
    }

    pub fn procinfo(&mut self) -> RdbgResult<()> {
        match self.program {
            Some(ref program) => println!("{:?}", program),
            None => println!("There is no program loaded."),
        }
        Ok(())
    }

    #[allow(deprecated)]
    pub fn continue_execution(&mut self) -> RdbgResult<()> {
        let pc = &self.get_pc()?;
        if self.breakpoints.contains_key(pc) {
            self.step_over_breakpoint()?;
        }
        ptrace::cont(self.pid, None)?;
        self.wait_for_signal()
    }

    /// Reads a word from the process memory at the given address.
    #[allow(deprecated)]
    pub fn read_memory(&self, address: Address) -> RdbgResult<i64> {
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
    }

    /// Writes a word with the given value to the process memory
    /// at the given address.
    #[allow(deprecated)]
    pub fn write_memory(&self, address: Address, data: i64) -> RdbgResult<()> {
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
    }

    pub fn set_breakpoint_at(&mut self, address: Address) {
        self.breakpoints.insert(
            address,
            breakpoint::Breakpoint::new(
                self.pid,
                address,
            ),
        );
    }

    pub fn remove_breakpoint(&mut self, address: Address) {
        self.breakpoints.remove(&address);
    }

    #[allow(deprecated)]
    fn single_step_instruction(&self) -> RdbgResult<()> {
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
        let pc = &self.get_pc().unwrap();
        if self.breakpoints.contains_key(pc) {
            self.step_over_breakpoint()
        } else {
            self.single_step_instruction()
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
    fn wait_for_signal(&self) -> RdbgResult<()> {
        match waitpid(self.pid, None) {
            Ok(WaitStatus::Exited(_, code)) => {
                info!("WaitStatus: Exited with status: {}", code);
                println!(
                    "[Inferior (process {}) exited with status {}]",
                    self.pid,
                    code
                );
                Ok(())
            }
            Ok(WaitStatus::Signaled(_, signal, core_dump)) => {
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
            info!(
                "Hit breakpoint at address {:?}",
                format!("{:#x}", self.get_pc().unwrap())
            );
        }
    }
}
