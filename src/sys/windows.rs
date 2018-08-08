// Currently the `nix` crate is used for accessing low level APIs.
// For Windows, a different crate will be needed to perform these functions.
// The API implemented here should ideally follow the same function signatures
// from other platforms as shown below.

// pub fn execute(program: &str) -> RdbgResult<Pid> {}
// pub fn attach(pid: Pid) -> RdbgResult<()> {}
// pub fn detach(pid: Pid) -> RdbgResult<()> {}
// pub fn continue_execution(pid: Pid) -> RdbgResult<TraceEvent> {}
// pub fn single_step(pid: Pid) -> RdbgResult<TraceEvent> {}
// pub fn syscall(pid: Pid) -> RdbgResult<()> {}
// pub fn kill(pid: Pid) -> RdbgResult<TraceEvent> {}
// pub fn read_memory(pid: Pid, address: Word) -> RdbgResult<Word> {}
// pub fn write_memory(pid: Pid, address: Word, data: Word) -> RdbgResult<()> {}
// pub fn read_register(pid: Pid, register: usize) -> RdbgResult<Word> {}
// pub fn write_register(pid: Pid, register: usize, data: Word) -> RdbgResult<()> {}
