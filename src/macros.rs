/// Used by API commands to check if the program is being debugged
/// before attempting to perform an operation on the tracee.
macro_rules! OnlyWhenRunning {
    ($dbg:expr) => {
        if !$dbg.is_alive() {
            println!("Program is not being run");
            return Ok(());
        }
    };
}

/// Attempts to parse a hex string to a `usize`.
macro_rules! FromHexString {
    ($address:expr) => {
        usize::from_str_radix($address.split("x").skip(1).next().unwrap(), 16)
            .chain_err(|| format!("Invalid address: {}", $address))?
    };
}
