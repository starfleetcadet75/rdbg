use goblin;
use goblin::Object;
use goblin::elf;
use goblin::error;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use super::super::Address;
use util::error::{RdbgError, RdbgResult};

// TODO: Make different types for each arch (ELF, PE, MACH)
pub struct Program {
    pub path: PathBuf,
    pub args: Vec<String>,
    pub entry: Address,
}

impl Program {
    pub fn new(path: &PathBuf) -> Program {
        Program {
            path: PathBuf::from(path),
            args: Vec::new(),
            entry: 0,
        }
    }

    pub fn args(&mut self, args: Vec<String>) { self.args = args; }

    pub fn load(&mut self) -> RdbgResult<()> {
        let mut buffer = Vec::new();
        let mut fd = File::open(self.path.clone())?;
        fd.read_to_end(&mut buffer)?;

        match goblin::parse(&buffer)? {
            Object::Elf(elf) => {
                println!("elf: {:#?}", &elf);
            }
            Object::PE(pe) => {
                println!("pe: {:#?}", &pe);
            }
            Object::Mach(mach) => {
                println!("mach: {:#?}", &mach);
            }
            Object::Archive(archive) => {
                println!("archive: {:#?}", &archive);
            }
            Object::Unknown(magic) => println!("Unsupported Binary: {:#?}", &magic),
        }
        Ok(())

        // match goblin::elf::Elf::parse(&buffer) {
        //     Ok(binary) => {
        //         self.entry = binary.entry;
        //         Ok(())
        //     }
        //     Err(_) => Err(RdbgError::GoblinError),
        // }
    }
}
