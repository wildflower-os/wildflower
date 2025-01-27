use crate::api;
use crate::api::clock::DATE_TIME_ZONE;
use crate::api::console::Style;
use crate::api::process::ExitCode;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() > 2 {
        return help();
    }
    let format = if args.len() > 1 {
        args[1]
    } else {
        DATE_TIME_ZONE
    };
    if format == "-h" || format == "--help" {
        return help();
    }
    println!("{}", api::time::format_offset_time(api::time::now()));
    Ok(())
}

fn help() -> Result<(), ExitCode> {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} date {}[<format>]{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
    Ok(())
}
