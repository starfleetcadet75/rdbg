use Address;

#[derive(Debug)]
pub struct Program {
    pub entry: Address,
}

impl Program {
    pub fn new() -> Program { Program { entry: 0 } }
}
