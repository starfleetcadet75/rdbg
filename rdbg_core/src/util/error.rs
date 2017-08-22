use nix;

use std::{fmt, result};
use std::error::Error;
use std::num::ParseIntError;

/// The rdbg error type provides a common way of
/// handling errors that may occur during execution.
#[derive(Debug)]
pub enum RdbgError {
    /// A command encountered an error during execution.
    CommandError(String),
    /// An error was caused by a call into the Nix crate.
    NixError,
    /// An error reading program input.
    ParseError,
}

/// rdbg result type
pub type RdbgResult<T> = result::Result<T, RdbgError>;

impl From<nix::Error> for RdbgError {
    fn from(_: nix::Error) -> RdbgError { RdbgError::NixError }
}

impl From<ParseIntError> for RdbgError {
    fn from(_: ParseIntError) -> RdbgError { RdbgError::ParseError }
}

impl Error for RdbgError {
    fn description(&self) -> &str {
        match self {
            &RdbgError::CommandError(_) => "Error executing command",
            &RdbgError::NixError => "Error calling nix function",
            &RdbgError::ParseError => "Error parsing input value",
        }
    }
}

impl fmt::Display for RdbgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &RdbgError::CommandError(ref cmd) => write!(f, "Error executing command: {}", cmd),
            &RdbgError::NixError => write!(f, "Error calling nix function"),
            &RdbgError::ParseError => write!(f, "Error parsing input value"),
        }
    }
}
