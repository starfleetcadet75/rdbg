use super::super::InferiorPid;

//#[derive(Debug, Copy, Clone)]
pub struct Breakpoint {
    pid: InferiorPid,
    address: u64,
    enabled: bool,
    stored_byte: u8,
}

impl Breakpoint {
    pub fn new(pid: InferiorPid, address: u64) -> Breakpoint {
        Breakpoint {
            pid: pid,
            address: address,
            enabled: true,
            stored_byte: 0,
        }
    }

    pub fn enable(&self) {}

    pub fn disable(&self) {}

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_address(&self) -> u64 {
        self.address
    }
}
