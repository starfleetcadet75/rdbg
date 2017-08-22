
use libc::c_void;
use nix::sys::ptrace;
use nix::sys::ptrace::ptrace::*;

use std::ptr;

use super::super::{Address, Pid};
use super::super::util::error::{RdbgError, RdbgResult};

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
        bp.enable().expect("problem setting breakpoint");
        bp
    }

    pub fn enable(&mut self) -> RdbgResult<()> {
        self.stored_word = self.read_memory(self.address)?;
        let mut data = self.stored_word; // save the current word before setting the int3
        data &= !0xff; // bitmask out the byte to change
        data |= 0xcc; // set the int3 instruction
        self.write_memory(self.address, data)?;
        self.enabled = true;
        Ok(())
    }

    pub fn disable(&mut self) -> RdbgResult<()> {
        let mut data = self.read_memory(self.address)?;
        data &= !0xff;
        data |= self.stored_word;
        self.write_memory(self.address, data)?;
        self.enabled = false;
        Ok(())
    }

    pub fn is_enabled(&self) -> bool { self.enabled }

    pub fn get_address(&self) -> Address { self.address }

    #[allow(deprecated)]
    fn read_memory(&self, address: Address) -> RdbgResult<i64> {
        unsafe {
            match ptrace::ptrace(
                PTRACE_PEEKDATA,
                self.pid,
                address as *mut c_void,
                ptr::null_mut(),
            ) {
                Ok(data) => Ok(data),
                Err(_) => Err(RdbgError::NixError),
            }
        }
    }

    #[allow(deprecated)]
    fn write_memory(&self, address: Address, data: i64) -> RdbgResult<()> {
        unsafe {
            match ptrace::ptrace(
                PTRACE_POKEDATA,
                self.pid,
                address as *mut c_void,
                data as *mut c_void,
            ) {
                Ok(_) => Ok(()),
                Err(_) => Err(RdbgError::NixError),
            }
        }
    }
}
