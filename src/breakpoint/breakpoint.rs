use super::super::{Pid, Address};
use super::super::core::ptrace_wrapper;

#[derive(Debug, Copy, Clone)]
pub struct Breakpoint {
    pid: Pid,
    address: Address,
    enabled: bool,
    stored_word: i64,
}

impl Breakpoint {
    pub fn new(pid: Pid, address: Address) -> Breakpoint {
        let mut bp = Breakpoint {
            pid: pid,
            address: address,
            enabled: false,
            stored_word: 0,
        };
        bp.enable();
        bp
    }

    pub fn enable(&mut self) {
        self.stored_word = ptrace_wrapper::peek_data(self.pid, self.address);
        let mut data = self.stored_word; // save the current word before setting the int3
        data &= !0xff; // bitmask out the byte to change
        data |= 0xcc; // set the int3 instruction
        ptrace_wrapper::poke_data(self.pid, self.address, data);
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        let mut data = ptrace_wrapper::peek_data(self.pid, self.address);
        data &= !0xff;
        data |= self.stored_word;
        ptrace_wrapper::poke_data(self.pid, self.address, data);
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_address(&self) -> Address {
        self.address
    }
}
