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
