use nix;
use nix::sys::ptrace;
use nix::sys::ptrace::ptrace::*;
use libc::c_void;

use std::ptr;

use super::super::{Pid, Address};

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
        self.stored_word = self.read_memory(self.address).unwrap();
        let mut data = self.stored_word; // save the current word before setting the int3
        data &= !0xff; // bitmask out the byte to change
        data |= 0xcc; // set the int3 instruction
        self.write_memory(self.address, data);
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        let mut data = self.read_memory(self.address).unwrap();
        data &= !0xff;
        data |= self.stored_word;
        self.write_memory(self.address, data);
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_address(&self) -> Address {
        self.address
    }

    fn read_memory(&self, address: Address) -> nix::Result<i64> {
        unsafe {
            ptrace::ptrace(PTRACE_PEEKDATA, self.pid, address as *mut c_void, ptr::null_mut())
        }
    }

    fn write_memory(&self, address: Address, data: i64) {
        unsafe {
            ptrace::ptrace(PTRACE_POKEDATA, self.pid, address as *mut c_void, data as *mut c_void).ok();
        }
    }
}
