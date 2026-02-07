//! Binary entry point for `nsip`.

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::process::ExitCode;

use nsip::{Config, add, divide};

/// Runs the application logic.
fn run() -> Result<(), nsip::Error> {
    let config = Config::new().with_verbose(true);

    if config.verbose {
        eprintln!("Running nsip with verbose mode enabled");
    }

    let sum = add(2, 3);
    println!("2 + 3 = {sum}");

    let quotient = divide(10, 2)?;
    println!("10 / 2 = {quotient}");

    Ok(())
}

/// Main entry point.
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        },
    }
}
