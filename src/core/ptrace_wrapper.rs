//! Provides wrappers around ptrace functions
use nix::sys::ptrace::*;
use nix::sys::ptrace::ptrace::*;
use libc::{user_regs_struct, c_void};

use std::{ptr, mem};
use std::error::Error;

use super::super::{Pid, Address};

// TODO: better error handling
// pub fn trace_me() -> std::io::Result<()> {
pub fn trace_me() {
    ptrace(
        PTRACE_TRACEME,
        Pid::from_raw(0),
        ptr::null_mut(),
        ptr::null_mut(),
    ).ok()
        .expect("PTRACE_TRACEME failed");
}

pub fn attach(pid: Pid) {
    ptrace(PTRACE_ATTACH, pid, ptr::null_mut(), ptr::null_mut())
        .ok()
        .expect("PTRACE_ATTACH failed");
}

pub fn continue_execution(pid: Pid) {
    ptrace(PTRACE_CONT, pid, ptr::null_mut(), ptr::null_mut())
        .ok()
        .expect("PTRACE_CONTINUE failed");
}

pub fn peek_data(pid: Pid, address: Address) -> i64 {
    ptrace(
        PTRACE_PEEKDATA,
        pid,
        address.as_void_ptr(),
        ptr::null_mut(),
    ).ok()
        .expect("PTRACE_PEEKDATA failed")
}

pub fn poke_data(pid: Pid, address: Address, data: i64) {
    ptrace(
        PTRACE_POKEDATA,
        pid,
        address.as_void_ptr(),
        data as *mut c_void,
    ).ok()
        .expect("PTRACE_POKEDATA failed");
}

// let regs = ptrace_get_data::<user_regs_struct>(PTRACE_GETREGS, child);
// println!("regs: {:#?}", r);
pub fn get_instruction_pointer(pid: Pid) -> Result<u64, Box<Error>> {
    let mut registers: user_regs_struct = unsafe { mem::zeroed() };
    let register_ptr: *mut c_void = &mut registers as *mut _ as *mut c_void;

    ptrace(PTRACE_GETREGS, pid, ptr::null_mut(), register_ptr)?;
    Ok(registers.rip)
}
