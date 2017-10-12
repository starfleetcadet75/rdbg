use Address;

#[derive(Debug)]
pub struct Program {
    pub entry: Address,
    pub is_64: bool,
    pub is_lib: bool,
}

impl Program {
    pub fn new() -> Program {
        Program {
            entry: 0,
            is_64: false,
            is_lib: false,
        }
    }
}
