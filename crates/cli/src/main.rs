//! RustMatte CLI - Command line interface for offline background removal.

use std::env;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    println!("RustMatte {VERSION}");
    println!("Offline, privacy-first AI background remover with a Rust core.");
    println!();
    println!("USAGE:");
    println!("    rustmatte [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Print help information");
    println!("    -V, --version    Print version information");
    println!();
    println!("Note: Application is in pre-alpha; background removal processing begins in Phase 1.");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "-V" | "--version" => {
            println!("rustmatte {VERSION}");
        }
        "-h" | "--help" => {
            print_help();
        }
        other => {
            eprintln!("Unknown argument: {other}");
            eprintln!("Run 'rustmatte --help' for usage.");
            std::process::exit(1);
        }
    }
}
