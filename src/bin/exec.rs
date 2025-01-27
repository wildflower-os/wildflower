#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use wildflower::api::io;
use wildflower::api::process;
use wildflower::api::syscall;
use wildflower::entry_point;

entry_point!(main);

fn main(_args: &[&str]) {
    loop {
        syscall::write(1, "\n> ".as_bytes());
        let line = io::stdin().read_line();
        let cmd = line.trim();
        if cmd == "quit" {
            syscall::exit(process::ExitCode::Success);
        } else {
            let args: Vec<&str> = cmd.split(' ').collect();
            let mut path = String::from("/bin/");
            path.push_str(args[0]);
            let _ = process::spawn(&path, &args);
        }
    }
}
