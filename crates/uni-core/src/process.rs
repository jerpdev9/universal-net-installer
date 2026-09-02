//! Thin, instrumented wrapper around [`std::process::Command`].
//!
//! Every crate that shells out to a system tool (`lsblk`, `lspci`, `nmcli`,
//! ...) goes through here instead of calling `std::process::Command`
//! directly. That gives us one place to add tracing, timeouts and argument
//! redaction (secrets such as Wi-Fi passwords must never reach the logs).

use std::process::Command;

use crate::error::{CoreError, Result};

/// Runs `command` with `args` and returns trimmed stdout on success.
///
/// The full argument list is recorded at `debug` level. Use
/// [`run_redacted`] instead when any argument may contain a secret.
pub fn run(command: &str, args: &[&str]) -> Result<String> {
    tracing::debug!(command, ?args, "running command");
    run_inner(command, args)
}

/// Same as [`run`], but never logs `args` (only the command name).
///
/// Use this for anything that carries a credential, e.g. `nmcli device
/// wifi connect <ssid> password <secret>`.
pub fn run_redacted(command: &str, args: &[&str]) -> Result<String> {
    tracing::debug!(command, "running command (arguments redacted)");
    run_inner(command, args)
}

fn run_inner(command: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|source| CoreError::CommandNotRunnable {
            command: command.to_string(),
            source,
        })?;

    if !output.status.success() {
        return Err(CoreError::CommandFailed {
            command: command.to_string(),
            status: output.status.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Returns `true` if `command` is available on `PATH`.
pub fn is_available(command: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {command}"))
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_captures_stdout() {
        let out = run("printf", &["hello"]).unwrap();
        assert_eq!(out, "hello");
    }

    #[test]
    fn run_reports_nonzero_exit() {
        let err = run("sh", &["-c", "exit 3"]).unwrap_err();
        match err {
            CoreError::CommandFailed { status, .. } => assert_eq!(status, 3),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn run_reports_missing_binary() {
        let err = run("uni-installer-definitely-not-a-real-binary", &[]).unwrap_err();
        assert!(matches!(err, CoreError::CommandNotRunnable { .. }));
    }

    #[test]
    fn is_available_detects_sh() {
        assert!(is_available("sh"));
        assert!(!is_available("uni-installer-definitely-not-a-real-binary"));
    }
}
