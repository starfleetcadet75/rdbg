use goblin;

use core::program::Program;
use loaders::ProgramLoader;
use util::error::RdbgResult;

pub struct ElfLoader;

impl ProgramLoader for ElfLoader {
    fn load(buffer: &[u8]) -> RdbgResult<(Program)> {
        let mut program = Program::new();
        let binary = goblin::elf::Elf::parse(&buffer)?;

        program.entry = binary.entry;

        Ok(program)
    }
}
