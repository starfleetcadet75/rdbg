use capstone::prelude::Capstone;

use std::fmt::Debug;

use util::errors::RdbgResult;

pub mod x86;

pub trait Architecture: Debug {
    /// Get the size of a natural word for this architecture in bits.
    fn word_size(&self) -> usize;
    /// Get the name of the instruction pointer for this architecture.
    fn instruction_pointer(&self) -> &str;
    /// Get the name of the stack pointer for this architecture.
    fn stack_pointer(&self) -> &str;
    /// Get the name of the frame pointer for this architecture, if it has a frame pointer.
    fn frame_pointer(&self) -> Option<&'static str>;
    /// Get a list of return address register names, if used on this architecture.
    fn return_address_register(&self) -> Option<Vec<&'static str>>;
    /// Get the name of the flags register.
    fn flags_register(&self) -> &str;
    /// List of general purpose registers.
    fn general_purpose_registers(&self) -> Vec<&'static str>;
    /// List of register names used for passing function arguments.
    fn args(&self) -> Vec<&'static str>;
    /// Get the name of the register used for return values.
    fn return_value_register(&self) -> &str;
    /// Given a valid register name, returns the offset for use with `ptrace`.
    fn get_register_offset(&self, &str) -> Option<usize>;
    /// Clone into a boxed `Architecture`.
    fn box_clone(&self) -> Box<Architecture>;
    /// Creates a new `Capstone` engine to be used for disassembly.
    fn get_disassembler(&self) -> RdbgResult<Capstone>;
}
