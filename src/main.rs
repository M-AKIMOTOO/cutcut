mod cli;
mod diagnostic;
mod runtime;
mod split;

use glow::{Config as GlowConfig, Reader as GlowReader};
use std::env;
use std::fs;
use std::process;

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
            print_detail();
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

fn print_detail() {
    match render_detail_with_glow() {
        Ok(rendered) => print!("{rendered}"),
        Err(_) => print!("{DETAIL}"),
    }
}

fn render_detail_with_glow() -> std::io::Result<String> {
    let path = env::temp_dir().join(format!("cutcut-detail-{}.md", process::id()));
    fs::write(&path, DETAIL)?;

    let reader = GlowReader::new(GlowConfig::new().pager(false).width(100));
    let rendered = reader.read_file(&path);
    let _ = fs::remove_file(&path);
    rendered
}

fn run_main() -> Result<(), AppError> {
    match parse_args(env::args())? {
        ParseResult::Config(config) => runtime::run(config),
        ParseResult::Help => Err(AppError::Help),
        ParseResult::Detail => Err(AppError::Detail),
    }
}
