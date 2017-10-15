use goblin;
use goblin::Hint;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use formats::elf_program::ElfProgram;
use util::error::{RdbgError, RdbgResult};

mod elf_loader;

pub trait ProgramLoader {
    fn load(buffer: &[u8]) -> RdbgResult<ElfProgram>;
}

// TODO: Make this a macro maybe?
pub fn load(path: &Path) -> RdbgResult<ElfProgram> {
    let mut fd = File::open(path)?;
    let peek = goblin::peek(&mut fd)?;

    if let Hint::Unknown(magic) = peek {
        error!("Unkown file type: {:#?}", &magic);
        Err(RdbgError::UnsupportedProgram)
    } else {
        let mut buffer = Vec::new();
        fd.read_to_end(&mut buffer)?;

        match peek {
            Hint::Elf(_) => elf_loader::ElfLoader::load(&buffer),
            Hint::PE => {
                error!("PE programs are not yet supported by rdbg");
                Err(RdbgError::UnsupportedProgram)
            }
            Hint::Mach(_) => {
                error!("MACH programs are not yet supported by rdbg");
                Err(RdbgError::UnsupportedProgram)
            }
            _ => {
                error!("Unknown or unsupported program");
                Err(RdbgError::UnsupportedProgram)
            }
        }
    }
}
