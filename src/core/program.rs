use object::{File, Machine, Object};

use std::fmt::Write;
use std::fs;
use std::io::Read;
use std::path::Path;

use arch::x86_64::X86_64;
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
                Machine::X86 => Box::new(X86_64::new()),
                Machine::X86_64 => Box::new(X86_64::new()),
                _ => bail!("Unsupported Architecture"),
            }
        };

        Ok(Program {
            program_path: program_path,
            architecture: architecture,
            bytes: bytes,
        })
    }

    pub fn get(&self) -> File { File::parse(&self.bytes).expect("Failed to parse file bytes") }

    /// The address of the entry point to the `Program`.
    pub fn entry(&self) -> u64 { self.get().entry() }

    /// Prints segment information parsed from the program.
    pub fn segments(&self) {
        for segment in self.get().segments() {
            println!("{:?}", segment);
        }
    }

    /// Prints section header information parsed from the program.
    pub fn sections(&self) -> String {
        let mut output = String::new();
        for section in self.get().sections() {
            write!(&mut output, "{:#?}", section).unwrap()
        }
        output
    }
}
