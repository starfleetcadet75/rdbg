use error_chain::error_chain;
use std::io;

error_chain! {
  types {
    RdbgError, RdbgErrorKind, RdbgResultExt, RdbgResult;
  }
  foreign_links {
    Io(io::Error);
    Sys(rdbg_sys::errors::SysError);
  }
  errors {
    UnknownCommand(input: String) {
      description("Undefined command")
      display("Undefined command: \"{}\". Try \"help\".", input)
    }
    InvalidRegister(register: String) {
      description("Invalid register name")
      display("Invalid register name: {}", register)
    }
//    InvalidMemoryAccess(address: Word) {
//      description("Attempted to access invalid memory")
//      display("Invalid address: {:#08x}", address)
//    }
  }
}
