use error_chain::error_chain;
use nix;

error_chain! {
  types {
    SysError, SysErrorKind, SysResultExt, SysResult;
  }
  foreign_links {
    Nix(nix::Error);
  }
  errors {
  }
}
