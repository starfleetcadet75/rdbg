use core::profile::Profile;
use formats::elf_program::ElfProgram;

#[derive(Debug)]
pub struct Project {
    pub profile: Profile,
    pub program: ElfProgram, /* pub comments: HashMap<String>,
                              * pub config: Config, */
}

impl Project {
    pub fn new(profile: Profile, program: ElfProgram) -> Project {
        Project {
            profile: profile,
            program: program,
        }
    }
}
