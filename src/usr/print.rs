use crate::api::syscall;
use crate::api::process::ExitCode;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() < 2 {
        syscall::write(1, b"\n");
        Ok(())
    } else {
        syscall::write(1, args[1..].join(" ").as_bytes());
        syscall::write(1, b"\n");
        Ok(())
    }
}
