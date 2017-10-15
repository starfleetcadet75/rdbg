use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub program_path: PathBuf,
    pub args: Vec<String>,
    pub env: Vec<String>,
}

impl Profile {
    pub fn new(path: &Path) -> Profile {
        Profile {
            program_path: path.to_path_buf(),
            args: Vec::new(),
            env: Vec::new(),
        }
    }
}
