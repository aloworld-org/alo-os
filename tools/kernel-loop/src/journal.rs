//! What the loop is doing, what it did, and the file that asks it to stop.
//!
//! Everything here is append-only or a single flag. Nothing rewrites a line
//! that has already been written: a log a program can edit is a log that can be
//! made to say a run went better than it did.

use std::fmt::Write as _;
use std::io::Write as _;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::lock;

/// What one iteration came to.
pub enum Went {
    /// A task was gated and pushed, at this commit, under this name.
    Published(String, String),

    /// Nothing was handed over, so there was nothing to publish.
    NothingReady,
}

/// The log every iteration appends to.
const THE_LOG: &str = "loop.log";

/// The file whose existence asks the loop to finish and begin nothing else.
const THE_STOP: &str = "stop";

/// Write one line into the log, with the moment on it.
///
/// Best effort on purpose: a loop that stopped because it could not write its
/// own log would be a loop that stops when a disk is full, in the middle of a
/// task, which is worse than a missing line.
pub fn note(ours: &Path, what: &str) {
    let line = format!("{} {what}\n", now());
    println!("alo-kernel-loop: {what}");
    let _ = std::io::stdout().flush();
    if let Ok(mut log) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(ours.join(THE_LOG))
    {
        let _ = log.write_all(line.as_bytes());
    }
}

/// Seconds since the epoch, which is all a log line needs to be ordered by.
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or(0)
}

/// Ask the loop to finish what it is doing and begin nothing else.
///
/// # Errors
/// Whatever the machine said about writing the file.
pub fn asked_to_stop(ours: &Path) -> Result<(), String> {
    std::fs::create_dir_all(ours)
        .map_err(|why| format!("{} could not be made: {why}", ours.display()))?;
    std::fs::write(ours.join(THE_STOP), b"stop\n")
        .map_err(|why| format!("the stop could not be written: {why}"))
}

/// Whether somebody has asked it to stop.
#[must_use]
pub fn was_asked_to_stop(ours: &Path) -> bool {
    ours.join(THE_STOP).exists()
}

/// Take away a stop left by a run that has already ended.
///
/// A stop belongs to the loop that was running when it was asked for. Clearing
/// it as a run begins is what makes `stop` always mean *the loop running now*
/// rather than *the next one to start*.
pub fn the_stop_is_cleared(ours: &Path) {
    drop(std::fs::remove_file(ours.join(THE_STOP)));
}

/// What is happening, and what happened last.
///
/// # Errors
/// A sentence when the loop's own directory cannot be read at all.
pub fn said(ours: &Path) -> Result<String, String> {
    let mut saying = String::new();
    let running = match lock::whose(ours) {
        Some(whose) => format!("running, as process {whose}"),
        None => "not running".to_owned(),
    };
    let stopping = if was_asked_to_stop(ours) {
        " — and has been asked to stop"
    } else {
        ""
    };
    let _ = writeln!(saying, "alo-kernel-loop is {running}{stopping}.");

    match std::fs::read_to_string(ours.join(THE_LOG)) {
        Ok(log) => {
            let lines: Vec<&str> = log.lines().collect();
            let from = lines.len().saturating_sub(20);
            let _ = writeln!(
                saying,
                "\nthe last {} lines of its log:",
                lines.len() - from
            );
            for line in lines.iter().skip(from) {
                let _ = writeln!(saying, "  {line}");
            }
        }
        Err(_) => {
            let _ = writeln!(saying, "\nit has not written a log here yet.");
        }
    }
    Ok(saying)
}
