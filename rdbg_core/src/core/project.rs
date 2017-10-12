use std::path::Path;
use std::path::PathBuf;

use core::program::Program;

#[derive(Debug)]
pub struct Profile {
    pub args: Vec<String>,
    pub env: Vec<String>,
}

#[derive(Debug)]
pub struct Project {
    pub program_path: PathBuf,
    pub profile: Profile,
    pub program: Program, /* pub program_hash: Hash,
                           * pub comments: HashMap<String>,
                           * pub config: Config, */
}

impl Project {
    pub fn new(path: &Path, program: Program) -> Project {
        Project {
            program_path: path.to_path_buf(),
            profile: Profile {
                args: Vec::new(),
                env: Vec::new(),
            },
            program: program,
        }
    }
}
