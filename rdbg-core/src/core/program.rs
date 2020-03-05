use crate::util::errors::RdbgResult;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct Program {
    pub path: String,
    bytes: Vec<u8>,
}

impl Program {
    pub fn open(path: String) -> RdbgResult<Program> {
        let mut fd = File::open(Path::new(&path))?;
        let bytes = {
            let mut v = Vec::new();
            fd.read_to_end(&mut v)?;
            v
        };

        Ok(Program { path, bytes })
    }
}
