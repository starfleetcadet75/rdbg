use Word;

#[derive(Debug)]
pub struct Breakpoint {
    pub address: Word,
    pub enabled: bool,
    pub stored_word: Word,
}

impl Breakpoint {
    pub fn new(address: Word) -> Breakpoint {
        Breakpoint {
            address: address,
            enabled: false,
            stored_word: 0,
        }
    }
}
