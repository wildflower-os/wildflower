#![no_std]
#![no_main]

extern crate alloc;

use geodate::geodate;
use wildflower::api::clock;
use wildflower::api::syscall;
use wildflower::entry_point;

entry_point!(main);

fn main(args: &[&str]) {
    if args.len() < 2 {
        syscall::write(1, b"Usage: geodate <longitude> [<timestamp>]\n");
        return;
    }

    let format = "%h%y-%m-%d %c:%b";
    let longitude = args[1].parse().expect("Could not parse longitude");
    let timestamp = if args.len() == 3 {
        args[2].parse().expect("Could not parse timestamp")
    } else {
        clock::epoch_time()
    };

    let t = geodate::get_formatted_date(format, timestamp as i64, longitude);
    syscall::write(1, t.as_bytes());
    syscall::write(1, b"\n");
}
