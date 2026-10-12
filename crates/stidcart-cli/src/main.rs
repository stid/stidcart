//! The `stidcart` binary: runs a command and turns its result into stderr text and an exit code.
//! The commands themselves are in the library (`lib.rs`).

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match stidcart_cli::run(&args) {
        Ok(Some(path)) => {
            eprintln!("wrote {path}");
            ExitCode::SUCCESS
        }
        Ok(None) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("stidcart: {msg}");
            ExitCode::FAILURE
        }
    }
}
