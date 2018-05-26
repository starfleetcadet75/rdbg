use sys::Word;

#[derive(Debug)]
pub struct Breakpoint {
    /// The address of the `Breakpoint`.
    pub address: Word,
    /// Indicates whether the `Breakpoint` is enabled.
    pub enabled: bool,
    /// The instruction that has been replaced by an interrupt.
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
