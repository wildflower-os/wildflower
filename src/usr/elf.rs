use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;

use crate::usr;
use object::{Object, ObjectSection};

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() != 2 {
        help();
        return Err(ExitCode::UsageError);
    }
    if args[1] == "-h" || args[1] == "--help" {
        help();
        return Ok(());
    }

    let color = Style::color("yellow");
    let reset = Style::reset();
    let pathname = args[1];
    if let Ok(buf) = fs::read_to_bytes(pathname) {
        let bin = buf.as_slice();
        if let Ok(obj) = object::File::parse(bin) {
            println!("ELF entry address: {:#X}", obj.entry());
            for section in obj.sections() {
                if let Ok(name) = section.name() {
                    if name.is_empty() {
                        continue;
                    }
                    let addr = section.address() as usize;
                    let size = section.size();
                    let align = section.align();
                    println!();
                    println!(
                        "{}{}{} (addr: {:#X}, size: {}, align: {})",
                        color, name, reset, addr, size, align
                    );
                    if let Ok(data) = section.data() {
                        usr::hex::print_hex_at(data, addr);
                    }
                }
            }
            Ok(())
        } else {
            error!("Could not parse ELF");
            Err(ExitCode::Failure)
        }
    } else {
        error!("Could not read file '{}'", pathname);
        Err(ExitCode::Failure)
    }
}

fn help() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} elf {}<binary>{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
}
