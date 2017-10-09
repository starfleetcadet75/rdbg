/// Used for tracking the current state the debugger is running in.
#[derive(Debug)]
pub enum DebuggerState {
    /// The debugger was started without a program and no program has been loaded.
    Empty,
    /// An exec-file has been loaded by the debugger but is not being run.
    ExecLoaded,
    /// The debugger is actively debugging a program.
    Running,
    /// The debugger hit a breakpoint.
    Breakpoint,
    /// The inferior process has exited.
    Exited,
}