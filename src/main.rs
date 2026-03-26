mod cli;
mod diagnostic;
mod runtime;
mod split;

use std::env;

use crate::cli::{DETAIL, HELP, ParseResult, parse_args};
use crate::diagnostic::{AppError, print_diagnostic};

const BANNER: &str = r"  ____ _   _ _____ ____ _   _ _____
 / ___| | | |_   _/ ___| | | |_   _|
| |   | | | | | || |   | | | | | |
| |___| |_| | | || |___| |_| | | |
 \____|\___/  |_| \____|\___/  |_|";

fn main() {
    match run_main() {
        Ok(()) => {}
        Err(AppError::Help) => {
            print_banner();
            print!("{HELP}");
        }
        Err(AppError::Detail) => {
            print_banner();
            print!("{DETAIL}");
        }
        Err(AppError::Diagnostic(diagnostic)) => {
            print_diagnostic(&diagnostic);
            std::process::exit(1);
        }
        Err(AppError::Io(error)) => {
            eprintln!("error: I/O failure");
            eprintln!("detail: {error}");
            eprintln!();
            eprintln!("Try:");
            eprintln!("  - Pass input text after the options");
            eprintln!("  - Or pipe data into stdin");
            eprintln!();
            eprintln!("Examples:");
            eprintln!("  cutcut -d '/' aa/bb/cc");
            eprintln!("  printf 'aa/bb/cc\\n' | cutcut -d '/' -f 2");
            std::process::exit(1);
        }
    }
}

fn print_banner() {
    println!("{BANNER}");
    println!();
}

fn run_main() -> Result<(), AppError> {
    match parse_args(env::args())? {
        ParseResult::Config(config) => runtime::run(config),
        ParseResult::Help => Err(AppError::Help),
        ParseResult::Detail => Err(AppError::Detail),
    }
}
