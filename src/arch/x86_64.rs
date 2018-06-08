use capstone::prelude::*;

use std::collections::HashMap;

use arch::Architecture;
use util::errors::*;

/// The 64-bit X86 Architecture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct X86_64 {
    registers: HashMap<&'static str, usize>,
    flags: HashMap<&'static str, usize>,
}

impl X86_64 {
    pub fn new() -> X86_64 {
        // See `/usr/include/sys/user.h` for register offsets.
        let registers: HashMap<&str, usize> = [
            ("r15", 0),
            ("r14", 8),
            ("r13", 16),
            ("r12", 24),
            ("rbp", 32),
            ("rbx", 40),
            ("r11", 48),
            ("r10", 56),
            ("r9", 64),
            ("r8", 72),
            ("rax", 80),
            ("rcx", 88),
            ("rdx", 96),
            ("rsi", 104),
            ("rdi", 112),
            ("orig_rax", 120),
            ("rip", 128),
            ("cs", 136),
            ("eflags", 144),
            ("rsp", 152),
            ("ss", 160),
            ("fs_base", 168),
            ("gs_base", 176),
            ("ds", 184),
            ("es", 192),
            ("fs", 200),
            ("gs", 208),
        ].iter()
            .cloned()
            .collect();

        let flags: HashMap<&str, usize> = [
            ("CF", 0),
            ("PF", 2),
            ("AF", 4),
            ("ZF", 6),
            ("SF", 7),
            ("IF", 9),
            ("DF", 10),
            ("OF", 11),
        ].iter()
            .cloned()
            .collect();

        X86_64 {
            registers: registers,
            flags: flags,
        }
    }
}

impl Architecture for X86_64 {
    fn word_size(&self) -> usize { 64 }

    fn instruction_pointer(&self) -> &str { "rip" }

    fn stack_pointer(&self) -> &str { "rsp" }

    fn frame_pointer(&self) -> Option<&'static str> { Some("rbp") }

    fn return_address_register(&self) -> Option<Vec<&'static str>> { None }

    fn flags_register(&self) -> &str { "eflags" }

    fn general_purpose_registers(&self) -> Vec<&'static str> {
        vec![
            "rax", "rbx", "rcx", "rdx", "rdi", "rsi", "r8", "r9", "r10", "r11", "r12", "r13",
            "r14", "r15",
        ]
    }

    fn args(&self) -> Vec<&'static str> { vec!["rdi", "rsi", "rdx", "rcx", "r8", "r9"] }

    fn return_value_register(&self) -> &str { "rax" }

    fn get_register_offset(&self, name: &str) -> Option<usize> { self.registers.get(name).cloned() }

    fn box_clone(&self) -> Box<Architecture> { Box::new(self.clone()) }

    fn get_disassembler(&self) -> RdbgResult<Capstone> {
        Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Intel)
            .detail(true)
            .build()
            .chain_err(|| "Capstone: Failed to create Capstone engine")
    }
}

// pub mod x86 {
//    pub const EBX: usize = 0;
//    pub const ECX: usize = 4;
//    pub const EDX: usize = 8;
//    pub const ESI: usize = 12;
//    pub const EDI: usize = 16;
//    pub const EBP: usize = 20;
//    pub const EAX: usize = 24;
//    pub const XDS: usize = 28;
//    pub const XES: usize = 32;
//    pub const XFS: usize = 36;
//    pub const XGS: usize = 40;
//    pub const ORIG_EAX: usize = 44;
//    pub const EIP: usize = 48;
//    pub const XCS: usize = 52;
//    pub const EFLAGS: usize = 56;
//    pub const ESP: usize = 60;
//    pub const XSS: usize = 64;
//}
