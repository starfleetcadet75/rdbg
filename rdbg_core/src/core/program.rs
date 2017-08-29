use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use util::error::RdbgResult;

pub struct Program {
    pub path: PathBuf,
    pub args: Vec<String>,
    pub buffer: Vec<u8>,
}

impl Program {
    pub fn new(path: &PathBuf) -> Program {
        Program {
            path: PathBuf::from(path),
            args: Vec::new(),
            buffer: Vec::new(),
        }
    }

    pub fn args(&mut self, args: Vec<String>) { self.args = args; }

    pub fn load(&mut self) -> RdbgResult<()> {
        let mut fd = File::open(self.path.clone())?;
        fd.read_to_end(&mut self.buffer)?;
        Ok(())
    }
}
