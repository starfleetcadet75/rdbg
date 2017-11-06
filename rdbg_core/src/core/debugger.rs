use fnv::FnvHashMap;
use nix::sys::ptrace::Register;

use Word;
use core::breakpoint::Breakpoint;
use core::process::{Process, ProcessEvent};
use core::profile::Profile;
use core::project::Project;
use loaders;
use util::error::{RdbgError, RdbgResult};

#[derive(Debug)]
pub struct Debugger {
    project: Option<Project>,
    process: Option<Process>,
    breakpoints: FnvHashMap<Word, Breakpoint>,
}

impl Debugger {
    /// Constructor for a `Debugger` object.
    pub fn new() -> Debugger {
        Debugger {
            project: None,
            process: None,
            breakpoints: FnvHashMap::default(),
        }
    }

    pub fn new_project(&mut self, profile: Profile) -> RdbgResult<()> {
        info!("Creating new project: {:?}", profile);

        let program = loaders::load(&profile.program_path)?;
        self.project = Some(Project::new(profile, program));
        Ok(())
    }

    /// Launchs a new `Process` using the project's profile.
    pub fn execute(&mut self) -> RdbgResult<()> {
        if let Some(ref project) = self.project {
            if self.process.is_none() {
                self.process = Some(Process::new(&project.profile)?);
                Ok(())
            } else {
                Err(RdbgError::ProcessAlreadyRunning)
            }
        } else {
            Err(RdbgError::NoProgramLoaded)
        }
    }

    /// Attempts to attach to a running process with the given pid.
    ///
    /// # Arguments
    ///
    /// * `pid` - The pid of the running process.
    pub fn attach(&mut self, pid: i32) -> RdbgResult<()> {
        if self.process.is_none() {
            self.process = Some(Process::attach(pid)?);
            Ok(())
        } else {
            Err(RdbgError::ProcessAlreadyRunning)
        }
    }

    pub fn continue_execution(&mut self) -> RdbgResult<()> {
        if self.process.is_some() {
            let pc = self.process
                .as_mut()
                .expect("Failed to unwrap process")
                .get_register(Register::RIP)?;

            // Check if process is at a breakpoint and needs to step over it
            if self.breakpoints.contains_key(&pc) {
                let breakpoint = self.breakpoints.get_mut(&pc).expect(
                    "Failed to get mut reference to breakpoint",
                );

                if breakpoint.enabled {
                    // self.disable_breakpoint(breakpoint.address);
                    // self.single_step_instruction();
                    // self.enable_breakpoint(breakpoint.address)?;
                }
            }

            self.process
                .as_mut()
                .expect("Failed to unwrap process")
                .continue_execution()?;

            match self.process
                .as_mut()
                .expect("Failed to unwrap process")
                .last_event {
                ProcessEvent::Breakpoint => {
                    // Move the pc back one instruction
                    self.process
                        .as_mut()
                        .expect("Failed to unwrap process")
                        .set_register(Register::RIP, pc - 1)?;
                    info!("Hit breakpoint at address {:#x}", pc - 1);
                }
                _ => info!("Got ProcessEvent"),
            }
            Ok(())
        } else {
            Err(RdbgError::NoProcessRunning)
        }
    }

    pub fn print_breakpoints(&self) {
        let mut count = 1;
        for (address, breakpoint) in &self.breakpoints {
            println!(
                "Breakpoint {} is at {:#x}, enabled = {}",
                count,
                address,
                breakpoint.enabled
            );
            count += 1;
        }
    }

    pub fn set_breakpoint_at(&mut self, address: Word) -> RdbgResult<()> {
        if self.process.is_some() {
            println!(
                "Breakpoint {} at {:#x}",
                self.breakpoints.len() + 1,
                address
            );

            let breakpoint = Breakpoint::new(address);
            self.breakpoints.insert(address, breakpoint);
            self.enable_breakpoint(address)?;
            Ok(())
        } else {
            Err(RdbgError::NoProcessRunning)
        }
    }

    pub fn remove_breakpoint(&mut self, address: Word) -> RdbgResult<()> {
        if self.process.is_some() {
            if self.breakpoints.contains_key(&address) {
                self.breakpoints.remove(&address);
                info!("Removed breakpoint at {:#x}", address);
            } else {
                info!("No breakpoint found at {:#x}", address);
            }
            Ok(())
        } else {
            Err(RdbgError::NoProcessRunning)
        }
    }

    fn enable_breakpoint(&mut self, address: Word) -> RdbgResult<()> {
        if let Some(ref mut process) = self.process {
            if self.breakpoints.contains_key(&address) {
                let mut breakpoint = self.breakpoints.get_mut(&address).expect(
                    "Failed to get mut reference to breakpoint",
                );

                if !breakpoint.enabled {
                    let mut data = process.read_memory(address)?;
                    breakpoint.stored_word = data.clone(); // save the word being overwritten

                    data &= !0xff; // bitmask out the byte to change
                    data |= 0xcc; // set the int3 instruction
                    process.write_memory(address, data)?;
                    breakpoint.enabled = true;
                }
            } else {
                println!("No breakpoint at address {:#x}", address);
            }
            Ok(())
        } else {
            Err(RdbgError::NoProcessRunning)
        }
    }

    fn disable_breakpoint(&mut self, address: Word) -> RdbgResult<()> {
        if let Some(ref process) = self.process {
            if self.breakpoints.contains_key(&address) {
                let mut breakpoint = self.breakpoints.get_mut(&address).expect(
                    "Failed to get mut reference to breakpoint",
                );

                if breakpoint.enabled {
                    let mut data = process.read_memory(address)?;
                    data &= !0xff;
                    data |= breakpoint.stored_word; // restore the saved word at the breakpoint address

                    process.write_memory(address, data)?;
                    breakpoint.enabled = false;
                }
            } else {
                println!("No breakpoint at address {:#x}", address)
            }
            Ok(())
        } else {
            Err(RdbgError::NoProcessRunning)
        }
    }
}
