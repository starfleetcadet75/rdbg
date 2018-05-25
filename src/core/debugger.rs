use capstone::prelude::*;
use nix::unistd::Pid;

use core::program::Program;
use sys::{unix, Word};
use util::errors::*;

pub enum TraceEvent {
    SyscallEnter,
    SyscallExit,
    Continued,
    Signal(u8),
    Event(u8),
    Killed(u8, bool),
    Exit(i32),
}

pub struct Debugger {
    pub program: Program,
    pub disassembler: Capstone,
    pid: Pid,
}

impl Debugger {
    /// Constructor for a `Debugger` object.
    pub fn new(program_path: String) -> Debugger {
        let program = Program::new(program_path).expect("Failed to open file for reading");
        let disassembler = program
            .architecture
            .get_disassembler()
            .expect("Problem with Capstone");

        Debugger {
            program: program,
            disassembler: disassembler,
            pid: Pid::from_raw(0),
        }
    }

    pub fn execute(&mut self) -> RdbgResult<()> {
        self.pid = unix::execute(&self.program.program_path)?;
        Ok(())
    }

    pub fn continue_execution(&self) -> RdbgResult<()> {
        let event = unix::continue_execution(self.pid);
        Ok(())
    }

    pub fn single_step(&self) -> RdbgResult<()> {
        let event = unix::single_step(self.pid)?;
        // if let TraceEvent::Breakpoint = event {}
        Ok(())
    }

    pub fn syscall(&self) -> RdbgResult<()> { unix::syscall(self.pid) }

    pub fn read_register(&self, register: &str) -> RdbgResult<Word> {
        match self.program.architecture.get_register_offset(register) {
            Some(offset) => unix::read_register(self.pid, offset),
            None => Err(RdbgErrorKind::InvalidRegister(register.into()).into()),
        }
    }
}
