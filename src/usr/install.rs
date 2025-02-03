use crate::api::console::Style;
use crate::api::fs;
use crate::api::io;
use crate::api::process::ExitCode;
use crate::api::syscall;
use crate::{api, sys, usr};

use alloc::format;
use alloc::string::String;

macro_rules! copy_file {
    ($path:expr, $verbose:expr) => {{
        copy_file($path, include_bytes!(concat!("../../dsk", $path)), $verbose);
    }};
}

pub fn copy_files(verbose: bool) {
    create_dir("/bin", verbose); // Binaries
    create_dir("/dev", verbose); // Devices
    create_dir("/ini", verbose); // Initializers
    create_dir("/lib", verbose); // Libraries
    create_dir("/net", verbose); // Network
    create_dir("/src", verbose); // Sources
    create_dir("/tmp", verbose); // Temporaries
    create_dir("/usr", verbose); // User directories
    create_dir("/var", verbose); // Variables
    create_dir("/hfs", verbose); // Hidden File System

    // NOTE: Here are the binaries to be included in the build image, but they
    // can also be installed later with the `pkg install` command.

    //copy_file!("/bin/blank", verbose);
    //copy_file!("/bin/browse", verbose);
    copy_file!("/bin/clear", verbose);
    //copy_file!("/bin/exec", verbose);
    //copy_file!("/bin/fetch", verbose);
    //copy_file!("/bin/geocal", verbose);
    //copy_file!("/bin/geodate", verbose);
    copy_file!("/bin/halt", verbose);
    //copy_file!("/bin/hello", verbose);
    //copy_file!("/bin/mandelbrot", verbose);
    copy_file!("/bin/ntp", verbose);
    copy_file!("/bin/pkg", verbose);
    copy_file!("/bin/print", verbose);
    copy_file!("/bin/reboot", verbose);
    copy_file!("/bin/sleep", verbose);

    create_dir("/dev/ata", verbose); // Drives
    create_dir("/dev/ata/0", verbose);
    create_dir("/dev/ata/1", verbose);
    create_dir("/dev/clk", verbose); // Clock
    create_dir("/dev/net", verbose); // Network
    create_dir("/dev/vga", verbose);

    create_dev("/dev/ata/0/0", "ata-0-0", verbose);
    create_dev("/dev/ata/0/1", "ata-0-1", verbose);
    create_dev("/dev/ata/1/0", "ata-1-0", verbose);
    create_dev("/dev/ata/1/1", "ata-1-1", verbose);
    create_dev("/dev/clk/boot", "clk-boot", verbose);
    create_dev("/dev/clk/epoch", "clk-epoch", verbose);
    create_dev("/dev/clk/rtc", "clk-rtc", verbose);
    create_dev("/dev/console", "console", verbose);
    create_dev("/dev/net/tcp", "net-tcp", verbose);
    create_dev("/dev/net/udp", "net-udp", verbose);
    create_dev("/dev/net/gw", "net-gw", verbose);
    create_dev("/dev/net/ip", "net-ip", verbose);
    create_dev("/dev/net/mac", "net-mac", verbose);
    create_dev("/dev/net/usage", "net-usage", verbose);
    create_dev("/dev/null", "null", verbose);
    create_dev("/dev/random", "random", verbose);
    create_dev("/dev/speaker", "speaker", verbose);
    create_dev("/dev/vga/buffer", "vga-buffer", verbose);
    create_dev("/dev/vga/font", "vga-font", verbose);
    create_dev("/dev/vga/mode", "vga-mode", verbose);
    create_dev("/dev/vga/palette", "vga-palette", verbose);

    copy_file!("/ini/banner.txt", verbose);
    copy_file!("/ini/boot.sh", verbose);
    copy_file!("/ini/lisp.lsp", verbose);
    copy_file!("/ini/shell.sh", verbose);
    copy_file!("/ini/version.txt", verbose);

    create_dir("/ini/palettes", verbose);
    copy_file!("/ini/palettes/default.sh", verbose);
    copy_file!("/ini/palettes/gruvbox-dark.sh", verbose);
    copy_file!("/ini/palettes/gruvbox-light.sh", verbose);

    create_dir("/ini/fonts", verbose);
    //copy_file!("/ini/fonts/lat15-terminus-8x16.psf", verbose);
    copy_file!("/ini/fonts/zap-light-8x16.psf", verbose);
    //copy_file!("/ini/fonts/zap-vga-8x16.psf", verbose);

    create_dir("/lib/lisp", verbose);
    copy_file!("/lib/lisp/alias.lsp", verbose);
    copy_file!("/lib/lisp/core.lsp", verbose);
    copy_file!("/lib/lisp/file.lsp", verbose);
    //copy_file!("/lib/lisp/legacy.lsp", verbose);
    copy_file!("/lib/lisp/math.lsp", verbose);

    copy_file!("/tmp/alice.txt", verbose);
    //copy_file!("/tmp/machines.txt", verbose);
    //copy_file!("/tmp/roads.txt", verbose);

    //create_dir("/tmp/chess", verbose);
    //copy_file!("/tmp/chess/mi2.epd", verbose);
    //copy_file!("/tmp/chess/mi3.epd", verbose);
    //copy_file!("/tmp/chess/mi4.epd", verbose);
    //copy_file!("/tmp/chess/puru.epd", verbose);

    //create_dir("/tmp/lisp", verbose);
    //copy_file!("/tmp/lisp/colors.lsp", verbose);
    //copy_file!("/tmp/lisp/doc.lsp", verbose);
    //copy_file!("/tmp/lisp/factorial.lsp", verbose);
    //copy_file!("/tmp/lisp/fibonacci.lsp", verbose);
    //copy_file!("/tmp/lisp/geotime.lsp", verbose);
    //copy_file!("/tmp/lisp/pi.lsp", verbose);
    //copy_file!("/tmp/lisp/sum.lsp", verbose);
    //copy_file!("/tmp/lisp/tak.lsp", verbose);

    //create_dir("/tmp/life", verbose);
    //copy_file!("/tmp/life/centinal.cells", verbose);
    //copy_file!("/tmp/life/flower-of-eden.cells", verbose);
    //copy_file!("/tmp/life/garden-of-eden.cells", verbose);
    //copy_file!("/tmp/life/glider-gun.cells", verbose);
    //copy_file!("/tmp/life/pentadecathlon.cells", verbose);
    //copy_file!("/tmp/life/queen-bee-shuttle.cells", verbose);
    //copy_file!("/tmp/life/ship-in-a-bottle.cells", verbose);
    //copy_file!("/tmp/life/thunderbird.cells", verbose);
    //copy_file!("/tmp/life/wing.cells", verbose);

    //create_dir("/tmp/beep", verbose);
    //copy_file!("/tmp/beep/tetris.sh", verbose);
    //copy_file!("/tmp/beep/starwars.sh", verbose);
    //copy_file!("/tmp/beep/mario.sh", verbose);

    create_dir("/var/log", verbose);

    create_dir("/var/www", verbose);
    copy_file!("/var/www/index.html", verbose);
    copy_file!("/var/www/moros.css", verbose);
    copy_file!("/var/www/moros.png", verbose);

    create_dir("/var/pkg", verbose);
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let csi_color = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Welcome to WildflowerOS v{} installation program!{}",
        csi_color,
        env!("CARGO_PKG_VERSION"),
        csi_reset
    );
    println!();

    let mut has_confirmed = false;
    for &arg in args {
        match arg {
            "-y" | "--yes" => has_confirmed = true,
            _ => continue,
        }
    }
    if !has_confirmed {
        print!("Proceed? [y/N] ");
        has_confirmed = io::stdin().read_line().trim() == "y";
        println!();
    }

    if has_confirmed {
        if !sys::fs::is_mounted() {
            println!("{}Listing disks ...{}", csi_color, csi_reset);
            usr::shell::exec("disk list").ok();
            println!("/dev/mem        RAM DISK");
            println!();

            println!("{}Formatting disk ...{}", csi_color, csi_reset);
            print!("Enter path of disk to format: ");
            let path = io::stdin().read_line();
            if path.trim_end() == "/dev/mem" {
                usr::shell::exec(&format!("memory format"))?;
            } else {
                usr::shell::exec(&format!("disk format {}", path.trim_end()))?;
            }
            println!();
        }

        println!("{}Populating filesystem...{}", csi_color, csi_reset);
        let verbose = true;
        copy_files(verbose);

        if sys::process::user().is_none() {
            println!();
            println!("{}Creating user...{}", csi_color, csi_reset);
            let res = usr::user::main(&["user", "create"]);
            if res == Err(ExitCode::Failure) {
                return res;
            }
        }

        println!();
        println!("{}Installation successful!{}", csi_color, csi_reset);
        println!();
        println!("Quit the console or reboot to apply changes");
    }

    Ok(())
}

fn create_dir(path: &str, verbose: bool) {
    if fs::exists(path) {
        return;
    }
    if verbose {
        println!("Creating '{}'", path);
    }
    if let Some(handle) = api::fs::create_dir(path) {
        syscall::close(handle);
    }
}

fn create_dev(path: &str, name: &str, verbose: bool) {
    if fs::exists(path) {
        return;
    }
    if verbose {
        println!("Creating '{}'", path);
    }
    if let Some(handle) = fs::create_device(path, name) {
        syscall::close(handle);
    }
}

fn copy_file(path: &str, buf: &[u8], verbose: bool) {
    if fs::exists(path) {
        return;
    }
    if verbose {
        println!("Fetching '{}'", path);
    }
    if path.ends_with(".txt") {
        if let Ok(text) = String::from_utf8(buf.to_vec()) {
            let text = text.replace("{x.x.x}", env!("CARGO_PKG_VERSION"));
            fs::write(path, text.as_bytes()).ok();
        } else {
            fs::write(path, buf).ok();
        }
    } else {
        fs::write(path, buf).ok();
    }
}
