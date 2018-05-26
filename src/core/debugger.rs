use capstone::prelude::*;
use nix::unistd::Pid;

use std::collections::HashMap;

use core::breakpoint::Breakpoint;
use core::program::Program;
use core::TraceEvent;
use sys::{unix, Word};
use util::errors::*;

pub struct Debugger {
    /// `Program` contains most of the static information for a loaded program.
    pub program: Program,
    /// The `Capstone` engine configured to handle the architecture of the loaded program.
    pub disassembler: Capstone,
    /// The `Pid` of the currently traced process.
    pid: Pid,
    /// Collection of all breakpoints currently set.
    breakpoints: HashMap<Word, Breakpoint>,
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
            breakpoints: HashMap::new(),
        }
    }

    pub fn execute(&mut self) -> RdbgResult<()> {
        self.pid = unix::execute(&self.program.program_path)?;
        Ok(())
    }

    pub fn attach(pid: i32) -> RdbgResult<()> {
        debug!("Attempting to attach to process with Pid: {}", pid);
        let pid = Pid::from_raw(pid);
        unix::attach(pid)
    }

    pub fn continue_execution(&mut self) -> RdbgResult<()> {
        debug!("Continuing execution of process");
        self.step_over_breakpoint()?;

        if unix::continue_execution(self.pid)? == TraceEvent::Breakpoint {
            let pc = self.program.architecture.instruction_pointer();
            let value = self.read_register(pc)? - 1;
            self.write_register(pc, value)?;
            println!("Hit breakpoint at {:#x}", value);
        }
        Ok(())
    }

    pub fn single_step(&self) -> RdbgResult<()> {
        debug!("Single stepping the process");
        let event = unix::single_step(self.pid)?;
        warn!("Got event: {:?}", event);
        Ok(())
    }

    pub fn single_step_with_breakpoint(&mut self) -> RdbgResult<()> {
        let pc = self.read_register(self.program.architecture.instruction_pointer())?;

        if self.breakpoints.contains_key(&pc) {
            self.step_over_breakpoint()
        } else {
            self.single_step()
        }
    }

    pub fn syscall(&self) -> RdbgResult<()> { unix::syscall(self.pid) }

    pub fn read_register(&self, register: &str) -> RdbgResult<Word> {
        debug!("Reading from register: {:?}", register);

        match self.program.architecture.get_register_offset(register) {
            Some(offset) => unix::read_register(self.pid, offset),
            None => Err(RdbgErrorKind::InvalidRegister(register.into()).into()),
        }
    }

    pub fn write_register(&self, register: &str, data: Word) -> RdbgResult<()> {
        debug!("Writing to register: {:?}", register);

        match self.program.architecture.get_register_offset(register) {
            Some(offset) => unix::write_register(self.pid, offset, data),
            None => Err(RdbgErrorKind::InvalidRegister(register.into()).into()),
        }
    }

    pub fn procinfo(&self) -> RdbgResult<String> {
        Ok(format!(
            "exe = {:?}\npid = {}",
            self.program.program_path, self.pid
        ))
    }

    // TODO: Move the println to the command
    pub fn print_breakpoints(&self) {
        let mut count = 1;
        for (address, breakpoint) in &self.breakpoints {
            println!(
                "Breakpoint {} is at {:#x}, enabled = {}",
                count, address, breakpoint.enabled
            );
            count += 1;
        }
    }

    pub fn set_breakpoint_at(&mut self, address: Word) -> RdbgResult<()> {
        if self.breakpoints.contains_key(&address) {
            println!("Breakpoint already set at {:#x}", address)
        } else {
            let breakpoint = Breakpoint::new(address);
            self.breakpoints.insert(address, breakpoint);
            self.enable_breakpoint(address)?;

            println!("Breakpoint {} at {:#x}", self.breakpoints.len(), address);
        }
        Ok(())
    }

    pub fn remove_breakpoint(&mut self, address: Word) -> RdbgResult<String> {
        match self.breakpoints.remove(&address) {
            Some(_) => Ok(format!("Removed breakpoint at {:#x}", address)),
            None => Ok(format!("No breakpoint found at {:#x}", address)),
        }
    }

    pub fn enable_breakpoint(&mut self, address: Word) -> RdbgResult<()> {
        if let Some(breakpoint) = self.breakpoints.get_mut(&address) {
            if !breakpoint.enabled {
                let mut data = unix::read_memory(self.pid, address)?;
                breakpoint.stored_word = data.clone(); // Save the word being overwritten

                data &= !0xff; // Bitmask out the byte to change
                data |= 0xcc; // Set the `int3` instruction (opcode 0xcc)
                unix::write_memory(self.pid, address, data)?;
                breakpoint.enabled = true;
            }
        } else {
            println!("No breakpoint at {:#x}", address);
        }
        Ok(())
    }

    pub fn disable_breakpoint(&mut self, address: Word) -> RdbgResult<()> {
        if let Some(breakpoint) = self.breakpoints.get_mut(&address) {
            if breakpoint.enabled {
                let mut data = unix::read_memory(self.pid, address)?;
                data &= !0xff;
                data |= breakpoint.stored_word; // Restore the saved word at the breakpoint address

                unix::write_memory(self.pid, address, data)?;
                breakpoint.enabled = false;
            }
        } else {
            println!("No breakpoint at address {:#x}", address);
        }
        Ok(())
    }

    /// Steps over a breakpoint by disabling and then reenabling it.
    fn step_over_breakpoint(&mut self) -> RdbgResult<()> {
        let pc = self.read_register(self.program.architecture.instruction_pointer())?;

        // Steps over a breakpoint
        if self.breakpoints.contains_key(&pc) {
            if self.breakpoints.get(&pc).unwrap().enabled {
                self.disable_breakpoint(pc)?;
                self.single_step()?;
                self.enable_breakpoint(pc)?;
            }
        }
        Ok(())
    }
}
