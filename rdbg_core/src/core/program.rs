use std::path::PathBuf;

#[derive(Debug)]
pub struct Program {
    pub path: PathBuf,
    pub args: Vec<String>,
}

impl Program {
    pub fn new(path: &PathBuf) -> Program {
        Program {
            path: PathBuf::from(path),
            args: Vec::new(),
        }
    }
}
