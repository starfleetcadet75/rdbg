use Address;

#[derive(Debug)]
pub struct ElfProgram {
    pub entry: Address,
    pub is_64: bool,
    pub is_lib: bool,
}

impl ElfProgram {
    pub fn new() -> ElfProgram {
        ElfProgram {
            entry: 0,
            is_64: false,
            is_lib: false,
        }
    }
}
