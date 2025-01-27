#![no_std]
#![no_main]

extern crate alloc;

use wildflower::api::io;
use wildflower::api::vga;
use wildflower::entry_point;
use wildflower::print;

entry_point!(main);

fn main(_args: &[&str]) {
    vga::graphic_mode();
    print!("\x1b]R\x1b[1A"); // Reset palette
    while io::stdin().read_char().is_none() {
        x86_64::instructions::hlt();
    }
    vga::text_mode();
}
