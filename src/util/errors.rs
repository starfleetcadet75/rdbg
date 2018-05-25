use nix;
use std::io;

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
  }
}
