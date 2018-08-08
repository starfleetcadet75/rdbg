use nix;
use std::io;
use Word;

error_chain! {
  types {
    RdbgError, RdbgErrorKind, RdbgResultExt, RdbgResult;
  }
  foreign_links {
    Io(io::Error);
    Nix(nix::Error);
  }
  errors {
    InvalidRegister(register: String) {
      description("Invalid register name")
      display("Invalid register name: {}", register)
    }
    InvalidMemoryAccess(address: Word) {
      description("Attempted to access invalid memory")
      display("Invalid address: {:#08x}", address)
    }
  }
}
