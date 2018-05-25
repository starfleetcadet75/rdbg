use object::{File, Machine, Object};

use std::fs;
use std::io::Read;
use std::path::Path;

use arch::x86::X86;
use arch::Architecture;
use util::errors::*;

pub struct Program {
    pub program_path: String,
    pub architecture: Box<Architecture>,
    bytes: Vec<u8>,
}

impl Program {
    pub fn new(program_path: String) -> RdbgResult<Program> {
        let mut fd = fs::File::open(Path::new(&program_path))?;
        let bytes = {
            let mut v = Vec::new();
            fd.read_to_end(&mut v)?;
            v
        };

        let architecture = {
            let binary = File::parse(&bytes)?;
            match binary.machine() {
                Machine::X86 => Box::new(X86::new()),
                Machine::X86_64 => Box::new(X86::new()),
                _ => bail!("Unsupported Architecture"),
            }
        };

        Ok(Program {
            program_path: program_path,
            architecture: architecture,
            bytes: bytes,
        })
    }

    pub(crate) fn get(&self) -> File {
        File::parse(&self.bytes).expect("Failed to parse file bytes")
    }

    pub fn entry(&self) -> u64 { self.get().entry() }

    pub fn segments(&self) {
        for segment in self.get().segments() {
            println!("{:?}", segment);
        }
    }

    pub fn sections(&self) {
        for section in self.get().sections() {
            println!("{:?}", section);
        }
    }
}
