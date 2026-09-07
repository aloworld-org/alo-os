//! Process boundaries, streamed logs and a deadline that never starts a second worker.
use crate::Result;
use std::{
    fs::File,
    io::Write,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

pub fn git(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

pub fn checked(program: &str, args: &[&str], log: &mut File) -> Result<()> {
    writeln!(log, "COMMAND: {program} {}", args.join(" "))?;
    let exit = Command::new(program)
        .args(args)
        .stdout(log.try_clone()?)
        .stderr(log.try_clone()?)
        .status()?;
    if !exit.success() {
        return Err(format!("{program} failed ({exit}); see gates.log; nothing pushed").into());
    }
    Ok(())
}

pub fn worker(codex: &str, result: &Path, directory: &Path) -> Result<()> {
    let events = File::create(directory.join("events.jsonl"))?;
    let errors = File::create(directory.join("worker.log"))?;
    let mut child = Command::new(codex)
        .args([
            "exec",
            "--sandbox",
            "danger-full-access",
            "--json",
            "--color",
            "never",
            "--output-last-message",
        ])
        .arg(result)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(events)
        .stderr(errors)
        .spawn()?;
    if let Some(mut input) = child.stdin.take() {
        input.write_all(include_bytes!("../../../docs/autonomy/WORKER.md"))?;
    }
    let started = Instant::now();
    loop {
        if let Some(exit) = child.try_wait()? {
            if !exit.success() {
                return Err(
                    format!("Worker failed ({exit}); see worker.log; work preserved").into(),
                );
            }
            return Ok(());
        }
        if started.elapsed() > Duration::from_secs(6 * 60 * 60) {
            // Stop the complete worker tree on this Windows host, never another
            // session by process name. The supervisor halts even if termination fails.
            let _ = Command::new("taskkill")
                .args(["/PID", &child.id().to_string(), "/T", "/F"])
                .status();
            return Err(
                "Worker exceeded six hours; inspect remaining processes before restarting".into(),
            );
        }
        thread::sleep(Duration::from_secs(2));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failed_gate_returns_an_error_and_keeps_diagnostics() -> Result<()> {
        let path = std::env::temp_dir().join(format!("dev-loop-gate-{}.log", std::process::id()));
        let mut file = File::create(&path)?;
        let outcome = checked("git", &["--this-option-does-not-exist"], &mut file);
        assert!(outcome.is_err());
        drop(file);
        let text = std::fs::read_to_string(&path)?;
        assert!(text.contains("COMMAND: git --this-option-does-not-exist"));
        std::fs::remove_file(path)?;
        Ok(())
    }
}
