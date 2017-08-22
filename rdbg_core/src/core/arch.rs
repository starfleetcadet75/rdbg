use libc::{c_void, user_regs_struct};
use nix::sys::ptrace;
use nix::sys::ptrace::ptrace::*;

use std::{mem, ptr};
use std::fmt;
use std::slice::Iter;

use super::debugger::Debugger;
use super::super::util::error::RdbgResult;

#[derive(Debug, Copy, Clone)]
pub enum Register {
    R15,
    R14,
    R13,
    R12,
    Rbp,
    Rbx,
    R11,
    R10,
    R9,
    R8,
    Rax,
    Rcx,
    Rdx,
    Rsi,
    Rdi,
    OrigRax,
    Rip,
    Cs,
    Eflags,
    Rsp,
    Ss,
    FsBase,
    GsBase,
    Ds,
    Es,
    Fs,
    Gs,
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Debug::fmt(self, f) }
}

impl Register {
    pub fn iterator() -> Iter<'static, Register> {
        use self::Register::*;

        static REGISTERS: [Register; 27] = [
            R15,
            R14,
            R13,
            R12,
            Rbp,
            Rbx,
            R11,
            R10,
            R9,
            R8,
            Rax,
            Rcx,
            Rdx,
            Rsi,
            Rdi,
            OrigRax,
            Rip,
            Cs,
            Eflags,
            Rsp,
            Ss,
            FsBase,
            GsBase,
            Ds,
            Es,
            Fs,
            Gs,
        ];
        REGISTERS.into_iter()
    }
}

pub trait Arch {
    fn get_register_value(&self, register: Register) -> RdbgResult<u64>;
    fn set_register_value(&self, register: Register, data: u64);
    fn get_pc(&self) -> RdbgResult<u64>;
    fn set_pc(&self, data: u64);
    fn print_regs(&self) -> RdbgResult<()>;
    fn print_reg(&self, register: Register) -> RdbgResult<()>;
}

impl Arch for Debugger {
    #[allow(deprecated)]
    fn get_register_value(&self, register: Register) -> RdbgResult<u64> {
        // TODO: check if this fn is ever made public
        // let regs = ptrace::ptrace_get_data::<user_regs_struct>(PTRACE_GETREGS, pid);
        // println!("regs: {:#?}", regs);
        let mut registers: user_regs_struct = unsafe { mem::zeroed() };
        let register_ptr: *mut c_void = &mut registers as *mut _ as *mut c_void;

        unsafe {
            ptrace::ptrace(PTRACE_GETREGS, self.pid, ptr::null_mut(), register_ptr)?;
        }

        let reg = match register {
            Register::R15 => registers.r15,
            Register::R14 => registers.r14,
            Register::R13 => registers.r13,
            Register::R12 => registers.r12,
            Register::Rbp => registers.rbp,
            Register::Rbx => registers.rbx,
            Register::R11 => registers.r11,
            Register::R10 => registers.r10,
            Register::R9 => registers.r9,
            Register::R8 => registers.r8,
            Register::Rax => registers.rax,
            Register::Rcx => registers.rcx,
            Register::Rdx => registers.rdx,
            Register::Rsi => registers.rsi,
            Register::Rdi => registers.rdi,
            Register::OrigRax => registers.orig_rax,
            Register::Rip => registers.rip,
            Register::Cs => registers.cs,
            Register::Eflags => registers.eflags,
            Register::Rsp => registers.rsp,
            Register::Ss => registers.ss,
            Register::FsBase => registers.fs_base,
            Register::GsBase => registers.gs_base,
            Register::Ds => registers.ds,
            Register::Es => registers.es,
            Register::Fs => registers.fs,
            Register::Gs => registers.gs,
        };
        Ok(reg)
    }

    #[allow(deprecated)]
    fn set_register_value(&self, register: Register, data: u64) {
        unsafe {
            let mut registers: user_regs_struct = mem::zeroed();
            let register_ptr: *mut c_void = &mut registers as *mut _ as *mut c_void;
            ptrace::ptrace(PTRACE_GETREGS, self.pid, ptr::null_mut(), register_ptr).ok();

            match register {
                Register::R15 => registers.r15 = data,
                Register::R14 => registers.r14 = data,
                Register::R13 => registers.r13 = data,
                Register::R12 => registers.r12 = data,
                Register::Rbp => registers.rbp = data,
                Register::Rbx => registers.rbx = data,
                Register::R11 => registers.r11 = data,
                Register::R10 => registers.r10 = data,
                Register::R9 => registers.r9 = data,
                Register::R8 => registers.r8 = data,
                Register::Rax => registers.rax = data,
                Register::Rcx => registers.rcx = data,
                Register::Rdx => registers.rdx = data,
                Register::Rsi => registers.rsi = data,
                Register::Rdi => registers.rdi = data,
                Register::OrigRax => registers.orig_rax = data,
                Register::Rip => registers.rip = data,
                Register::Cs => registers.cs = data,
                Register::Eflags => registers.eflags = data,
                Register::Rsp => registers.rsp = data,
                Register::Ss => registers.ss = data,
                Register::FsBase => registers.fs_base = data,
                Register::GsBase => registers.gs_base = data,
                Register::Ds => registers.ds = data,
                Register::Es => registers.es = data,
                Register::Fs => registers.fs = data,
                Register::Gs => registers.gs = data,
            }
            ptrace::ptrace(PTRACE_SETREGS, self.pid, ptr::null_mut(), register_ptr).ok();
        }
    }

    fn get_pc(&self) -> RdbgResult<u64> { self.get_register_value(Register::Rip) }

    fn set_pc(&self, data: u64) { self.set_register_value(Register::Rip, data); }

    fn print_regs(&self) -> RdbgResult<()> {
        for &reg in Register::iterator() {
            self.print_reg(reg)?;
        }
        Ok(())
    }

    fn print_reg(&self, register: Register) -> RdbgResult<()> {
        println!(
            "{:?}\t{}",
            register,
            format!("{:#x}", self.get_register_value(register)?)
        );
        Ok(())
    }
}
