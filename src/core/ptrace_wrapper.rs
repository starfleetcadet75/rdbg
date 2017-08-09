//! Provides wrappers around ptrace functions

use nix::sys::ptrace::*;
use nix::sys::ptrace::ptrace::*;
use libc::{user_regs_struct, c_void};

use std::{ptr, mem};
use std::error::Error;

use super::super::InferiorPid;

pub fn trace_me() {
    ptrace(PTRACE_TRACEME, InferiorPid::from_raw(0), ptr::null_mut(), ptr::null_mut())
        .ok()
        .expect("PTRACE_TRACEME failed");
}

pub fn attach(pid: InferiorPid) {
    ptrace(PTRACE_ATTACH, pid, ptr::null_mut(), ptr::null_mut())
        .ok()
        .expect("PTRACE_ATTACH failed");
}

pub fn continue_execution(pid: InferiorPid) {
    ptrace(PTRACE_CONT, pid, ptr::null_mut(), ptr::null_mut())
        .ok()
        .expect("PTRACE_CONTINUE failed");
}

pub fn get_instruction_pointer(pid: InferiorPid) -> Result<u64, Box<Error>> {
    let mut registers: user_regs_struct = unsafe { mem::zeroed() };
    let register_ptr: *mut c_void = &mut registers as *mut _ as *mut c_void;

    ptrace(PTRACE_GETREGS, pid, ptr::null_mut(), register_ptr)?;
    Ok(registers.rip)
}
