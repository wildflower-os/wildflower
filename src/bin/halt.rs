#![no_std]
#![no_main]

use wildflower::api::power;
use wildflower::api::syscall;
use wildflower::entry_point;

entry_point!(main);

fn main(_args: &[&str]) {
    syscall::write(1, b"\x1b[93m"); // Yellow
    syscall::write(1, b"Halting WildflowerOS.");
    syscall::write(1, b"\x1b[0m"); // Reset
    syscall::write(1, b"\n");
    syscall::sleep(0.5);
    power::halt();
    loop {
        syscall::sleep(1.0)
    }
}
