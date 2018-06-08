use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;

use nix::unistd::Pid;
use sys::Word;
use util::errors::*;

/// Represents a parsed memory segment from reading '/proc/PID/maps'.
pub struct MemorySegment {
    /// The start address of the memory segment.
    start: i64,
    /// The end address of the memory segment.
    end: i64,
    /// The total size of the memory segment.
    size: i64,
    /// The permissions of the memory segment as a bitmask.
    permissions: u8,
    /// Offset that the memory segment begins in the mapped file.
    offset: i64,
    /// The name of the file that has been mapped.
    name: Option<String>,
}

impl MemorySegment {
    /// Checks if the memory segment contains the given address.
    pub fn contains(&self, address: Word) -> bool {
        let address = address as i64;
        self.start <= address && address < self.end
    }

    /// Checks whether this `MemorySegment` is marked as the stack.
    pub fn is_stack(&self) -> bool {
        if let Some(ref name) = self.name {
            return name == "[stack]";
        }
        false
    }

    /// Checks if this `MemorySegment` is from a memory mapped file.
    pub fn is_memory_mapped_file(&self) -> bool {
        if let Some(ref name) = self.name {
            return name.starts_with('[');
        }
        false
    }

    /// Checks whether the `MemorySegment` has read permissions.
    pub fn read(&self) -> bool { (self.permissions & 4) == 1 }

    /// Checks whether the `MemorySegment` has write permissions.
    pub fn write(&self) -> bool { (self.permissions & 2) == 1 }

    /// Checks whether the `MemorySegment` has execute permissions.
    pub fn execute(&self) -> bool { (self.permissions & 1) == 1 }

    /// Create a String that shows the `MemorySegment` permissions as they appear in `/proc/PID/maps`.
    pub fn permission_string(&self) -> String {
        let mut s = String::new();

        if self.read() {
            s.push('r');
        } else {
            s.push('-');
        }

        if self.write() {
            s.push('w');
        } else {
            s.push('-');
        }

        if self.execute() {
            s.push('x');
        } else {
            s.push('-');
        }

        s.push('p');
        s
    }
}

pub struct Memory {
    /// Copy of the `Pid` field of `Debugger`. Will always be the same value.
    pid: Pid,
    /// Path to the memory file provided by procfs.
    memory_file: PathBuf,
    /// Whether the debugger is able to access the memory file provided by procfs.
    /// Reading from procfs is faster than using `ptrace` requests.
    accessible: bool,
    /// Vector of `MemorySegment`s which are parsed from reading '/proc/PID/maps'.
    segments: Vec<MemorySegment>,
}

impl Memory {
    pub fn new(pid: Pid) -> RdbgResult<Memory> {
        // Test if the debugger is able to read directly from procfs or if it
        // should fallback to using `ptrace` requests to operate on memory.
        let memory_file = PathBuf::from(format!("/proc/{}/mem", pid));
        let accessible = File::open(&memory_file).is_ok();

        Ok(Memory {
            pid: pid,
            memory_file: memory_file,
            accessible: accessible,
            segments: Memory::parse_memory_segments(pid)?,
        })
    }

    pub fn read(&self, address: Word, size: usize) -> RdbgResult<Vec<u8>> {
        let _ = self.validate_address(address)?;
        let mut result = vec![0u8; size];

        if self.accessible {
            // Read from mem file
            let mut memory = File::open(&self.memory_file)?;
            memory.seek(SeekFrom::Start(address as u64))?;
            memory.read_exact(&mut result)?;
        } else {
            error!("Memory file access failed");
            unimplemented!();
            // Fallback to using `ptrace` requests to read memory
            // let offset = address % wordsize;
        }
        Ok(result)
    }

    /// Read one byte from the specified address.
    pub fn peek(&self, address: Word) -> RdbgResult<u8> {
        self.read(address, 1).map(|mut x| x.pop().unwrap())
    }

    /// Validates whether a given address is a valid location in memory.
    fn validate_address(&self, address: Word) -> RdbgResult<String> {
        // TODO: Do something with the name of the section and its permissions
        for segment in self.segments.iter() {
            if segment.contains(address) {
                if let Some(ref name) = segment.name {
                    return Ok(name.to_string());
                }
            }
        }
        Err(RdbgErrorKind::InvalidMemoryAccess(address).into())
    }

    fn parse_memory_segments(pid: Pid) -> RdbgResult<Vec<MemorySegment>> {
        let map_file = format!("/proc/{}/maps", pid);
        let fd = File::open(PathBuf::from(&map_file))?;

        let mut segments = vec![];
        for line in BufReader::new(fd).lines() {
            let temp = line.unwrap();
            let mut tokens = temp.split_whitespace();

            // Parse start and end addresses
            let mut addresses = tokens.next().unwrap().split('-');
            let start = i64::from_str_radix(addresses.next().unwrap(), 16)
                .chain_err(|| "Error parsing hex from /proc map file")?;
            let end = i64::from_str_radix(addresses.next().unwrap(), 16)
                .chain_err(|| "Error parsing hex from /proc map file")?;
            let size = end - start;

            // Parse permission flags
            let perm = tokens.next().unwrap();
            let mut flags = 0;

            if perm.contains('r') {
                flags |= 4;
            }

            if perm.contains('w') {
                flags |= 2;
            }

            if perm.contains('x') {
                flags |= 1;
            }

            // Parse file offset
            let offset = i64::from_str_radix(tokens.next().unwrap(), 16)
                .chain_err(|| "Error parsing hex from /proc map file")?;

            // Skip major/minor devices and inode
            let _ = tokens.next();
            let _ = tokens.next();

            // Parse the name
            let name = tokens.next().map(|s| s.to_string());

            segments.push(MemorySegment {
                start: start,
                end: end,
                size: size,
                permissions: flags,
                offset: offset,
                name: name,
            });
        }
        Ok(segments)
    }
}
