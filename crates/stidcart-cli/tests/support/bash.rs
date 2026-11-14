//! `bash` for the tests that run the repo's shell scripts (`scope.rs`). It takes no part in the
//! VICE boots, so a target uses it alone: `#[path = "support/bash.rs"] mod bash;`.

use std::process::Command;

/// A [`Command`] for the `bash` that PATH names.
///
/// On Windows, `Command::new("bash")` looks in the system directory before PATH, and finds
/// `C:\Windows\System32\bash.exe`, the WSL launcher, ahead of Git Bash. Setting PATH on the child
/// makes std search that first, even when the value is the one the child would inherit anyway.
/// Elsewhere the lookup already goes by PATH, so this changes nothing.
pub fn bash() -> Command {
    let mut cmd = Command::new("bash");
    if let Some(path) = std::env::var_os("PATH") {
        cmd.env("PATH", path);
    }
    cmd
}
