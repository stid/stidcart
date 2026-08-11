//! `stidcart` — the command-line harness over `stidcart-core`.
//!
//! This is a test harness, not a product. It exists so the core can be driven end to end and
//! diffed against a reference implementation; the interface mirrors that tool's `build` closely
//! enough to compare outputs.

use std::path::Path;
use std::process::ExitCode;

use stidcart_core::{CartBuilder, Program};

const USAGE: &str = "\
usage: stidcart build [-e NAME] -o OUTPUT INPUT.prg...

  build           create a cartridge image from program files
  -e NAME         cartridge name shown by the menu (default: \"(no EF-Name)\")
  -o OUTPUT       where to write the image
  INPUT.prg       programs to include; the entry name is the file stem
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
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

/// Returns the path written, or `None` when there was nothing to write.
fn run(args: &[String]) -> Result<Option<String>, String> {
    let Some(command) = args.first() else {
        return Err(format!("no command given\n{USAGE}"));
    };
    if command != "build" {
        return Err(format!("unknown command '{command}'\n{USAGE}"));
    }

    let mut ef_name: Option<String> = None;
    let mut output: Option<String> = None;
    let mut inputs: Vec<String> = Vec::new();

    let mut it = args[1..].iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-e" => ef_name = Some(it.next().ok_or("-e needs a name")?.clone()),
            "-o" => output = Some(it.next().ok_or("-o needs a path")?.clone()),
            "-h" | "--help" => return Err(USAGE.to_string()),
            other if other.starts_with('-') => return Err(format!("unknown option '{other}'")),
            other => inputs.push(other.to_string()),
        }
    }

    // Matching the reference tool: a request with no programs succeeds and writes nothing at all.
    // Whether that is the right long-term behaviour is a product question, deliberately not
    // decided here.
    if inputs.is_empty() {
        return Ok(None);
    }

    let output = output.ok_or("no output path; pass -o OUTPUT")?;

    let mut builder = CartBuilder::new();
    if let Some(name) = &ef_name {
        builder = builder.with_ef_name(name);
    }
    for path in &inputs {
        let data = std::fs::read(path).map_err(|e| format!("reading '{path}': {e}"))?;
        builder = builder.with_program(Program::new(&entry_name(path), data));
    }

    let bytes = builder.build().map_err(|e| e.to_string())?;
    std::fs::write(&output, &bytes).map_err(|e| format!("writing '{output}': {e}"))?;

    Ok(Some(output))
}

/// The menu name a program gets when the request does not give one: the file name with its
/// directory and extension removed, case and spacing preserved.
fn entry_name(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_name_strips_directory_and_extension() {
        assert_eq!(entry_name("fixtures/My Game v2.prg"), "My Game v2");
        assert_eq!(entry_name("hello.prg"), "hello");
        assert_eq!(entry_name("/tmp/a/b/thing.PRG"), "thing");
    }

    #[test]
    fn an_unknown_command_is_an_error() {
        assert!(run(&["frobnicate".to_string()]).is_err());
    }
}
