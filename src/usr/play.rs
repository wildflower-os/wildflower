use crate::{api::process::ExitCode, usr};

pub fn main(mut args: &[&str]) -> Result<(), ExitCode> {
    if args.len() != 2 {
        help();
        return Err(ExitCode::UsageError);
    }
    if args[1] == "-h" || args[1] == "--help" {
        help();
        return Ok(());
    }
    let game = args[1];
    args = &args[1..];
    match game {
        "2048" => {
            usr::pow::main(args)
        }
        "chess" => {
            usr::chess::main(args)
        }
        "life" => {
            usr::life::main(args)
        }
        _ => {
            help();
            return Err(ExitCode::UsageError);
        }
    }
}

fn help() {
    println!("play - play a game");
    println!("Usage:");
    println!("  play 2048");
    println!("  play chess");
    println!("  play life");
}
