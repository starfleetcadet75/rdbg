use goblin;
use nix;

use std::{fmt, io, result};
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
    /// An error occurred in goblin.
    GoblinError,
    /// An IO error occurred.
    IoError(io::Error),
    /// No program has been loaded by the debugger.
    NoProgramLoaded,
    /// Not enough arguments were given for the command.
    NotEnoughArgs,
    /// Attempted to load a program which was not recognized or is currently unsupported.
    UnsupportedProgram,
}

/// rdbg result type
pub type RdbgResult<T> = result::Result<T, RdbgError>;

impl From<nix::Error> for RdbgError {
    fn from(_: nix::Error) -> RdbgError { RdbgError::NixError }
}

impl From<ParseIntError> for RdbgError {
    fn from(_: ParseIntError) -> RdbgError { RdbgError::ParseError }
}

impl From<goblin::error::Error> for RdbgError {
    fn from(_: goblin::error::Error) -> RdbgError { RdbgError::GoblinError }
}

impl From<io::Error> for RdbgError {
    fn from(err: io::Error) -> RdbgError { RdbgError::IoError(err) }
}

impl Error for RdbgError {
    fn description(&self) -> &str {
        match self {
            &RdbgError::CommandError(_) => "Error executing command",
            &RdbgError::NixError => "Error calling nix function",
            &RdbgError::ParseError => "Error parsing input value",
            &RdbgError::GoblinError => "Error in goblin",
            &RdbgError::IoError(_) => "An IO Error occurred",
            &RdbgError::NoProgramLoaded => "There is no program loaded",
            &RdbgError::NotEnoughArgs => "Not enough arguments were given for the command",
            &RdbgError::UnsupportedProgram => {
                "Attempted to load a program which was not recognized or is currently unsupported"
            }
        }
    }
}

impl fmt::Display for RdbgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &RdbgError::CommandError(ref cmd) => write!(f, "Error executing command: {}", cmd),
            &RdbgError::NixError => write!(f, "Error calling nix function"),
            &RdbgError::ParseError => write!(f, "Error parsing input value"),
            &RdbgError::GoblinError => write!(f, "Error in goblin"),
            &RdbgError::IoError(_) => write!(f, "An IO Error occurred"),
            &RdbgError::NoProgramLoaded => write!(f, "There is no program loaded"),
            &RdbgError::NotEnoughArgs => {
                write!(f, "Not enough arguments were given for the command")
            }
            &RdbgError::UnsupportedProgram => {
                write!(
                    f,
                    "Attempted to load a program which was not recognized or is currently unsupported"
                )
            }
        }
    }
}
