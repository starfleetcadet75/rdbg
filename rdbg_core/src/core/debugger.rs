// use fnv::FnvHashMap;

use {Address, Pid};
use core::process::{Process, ProcessEvent};
use core::profile::Profile;
use core::project::Project;
use loaders;
use util::error::{RdbgError, RdbgResult};

#[derive(Debug)]
pub struct Debugger {
    project: Option<Project>,
    process: Option<Process>, // breakpoints: FnvHashMap<Address, Breakpoint>,
}

impl Debugger {
    /// Constructor for a `Debugger` object.
    pub fn new() -> Debugger {
        Debugger {
            project: None,
            process: None, // breakpoints: FnvHashMap::default(),
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
                return Ok(());
            }
        }
        Err(RdbgError::NoProgramLoaded)
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
            Err(RdbgError::NoProgramLoaded)
        }
    }
}
