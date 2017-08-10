#[macro_use]
extern crate log;
extern crate simplelog;
extern crate nix;
extern crate libc;
extern crate fnv;

pub mod core;
pub mod commands;
mod breakpoint;

pub type InferiorPid = nix::unistd::Pid;




#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_can_exec() {
        //let inferior = run(Path::new("/bin/echo"), &["42"]).unwrap();
        //assert_eq!(42, continue_execution(inferior));
    }
}
