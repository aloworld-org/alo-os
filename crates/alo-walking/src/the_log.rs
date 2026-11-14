//! **Reading the machine's serial log**: the lines said after a point, and the
//! first of them that carries a phrase.
//!
//! The image sends the journal to the serial port (`console.toml` and the
//! journald drop-in in `image/`), so a service's own sentence — *Started
//! alo-screen-reader.service*, *8 of this machine's applications arrived* — is
//! a line here. A point in the log is a byte offset, taken from the log's
//! length at the moment a walk marks it, so *since the mark* means exactly the
//! bytes written after it.
//!
//! # Two writers share the port, and a line can be torn
//!
//! The kernel's own messages and journald's forwarded ones reach the same
//! serial port, and a kernel line can land **inside** a journal line. Measured
//! on the walk of 2026-11-14 on GitHub:
//!
//! ```text
//! [  127.851670] alo-shipping[2168]: 8 of this machine's applications arrived, 0 s[  128.234364] audit: type=1130 … addr=? terminal=? res=success'
//! till owed
//! ```
//!
//! A check that read that line as it stands finds *0 s* and not *0 still
//! owed*, and fails a machine that did nothing wrong. So lines are read as
//! **records**: a timestamp in the middle of a line starts a new record and
//! leaves the one it interrupted open, and the next line that starts without
//! a timestamp is the rest of the interrupted one
//! (`docs/quirks/the-serial-port-tears-a-journal-line.md`).
//!
//! And only **whole** lines are read: the last line of a log still being
//! written may be half of one, and a check must not judge half a sentence.

/// The records said after `from` bytes, as text, with torn lines mended; a
/// byte that is not UTF-8 reads as a replacement character rather than hiding
/// the line.
#[must_use]
pub fn lines_after(log: &[u8], from: usize) -> Vec<String> {
    let after = log.get(from..).unwrap_or_default();
    let whole = after
        .iter()
        .rposition(|byte| *byte == b'\n')
        .and_then(|end| after.get(..end))
        .unwrap_or_default();
    let text = String::from_utf8_lossy(whole);
    let mut records: Vec<String> = Vec::new();
    // The record a timestamp interrupted, by its place in `records`.
    let mut interrupted: Option<usize> = None;
    for line in text.lines() {
        let line = line.trim_end_matches('\r');
        let pieces = split_at_timestamps(line);
        let mut pieces = pieces.into_iter();
        let Some(first) = pieces.next() else {
            continue;
        };
        let rest: Vec<&str> = pieces.collect();
        // Where `first` went: the rest of an interrupted record, or a new one.
        let went_to = match interrupted.take() {
            Some(at) if !starts_with_a_timestamp(first) => {
                if let Some(record) = records.get_mut(at) {
                    record.push_str(first);
                }
                at
            }
            _ => {
                records.push(first.to_owned());
                records.len().saturating_sub(1)
            }
        };
        if !rest.is_empty() {
            // The record `first` belongs to was cut off by the ones after it,
            // and its rest is on the next line that has no timestamp.
            interrupted = Some(went_to);
            records.extend(rest.iter().map(|it| (*it).to_owned()));
        }
    }
    records
}

/// The first record after `from` that carries `phrase`.
#[must_use]
pub fn first_with(log: &[u8], from: usize, phrase: &str) -> Option<String> {
    lines_after(log, from)
        .into_iter()
        .find(|line| line.contains(phrase))
}

/// Every record after `from` that carries any of `phrases`.
#[must_use]
pub fn all_with(log: &[u8], from: usize, phrases: &[String]) -> Vec<String> {
    lines_after(log, from)
        .into_iter()
        .filter(|line| phrases.iter().any(|phrase| line.contains(phrase.as_str())))
        .collect()
}

/// A line cut where a kernel timestamp begins anywhere but at its start.
fn split_at_timestamps(line: &str) -> Vec<&str> {
    let mut pieces = Vec::new();
    let mut start = 0;
    for (at, _) in line.match_indices('[') {
        if at > start && line.get(at..).is_some_and(starts_with_a_timestamp) {
            pieces.extend(line.get(start..at));
            start = at;
        }
    }
    pieces.extend(line.get(start..));
    pieces
}

/// Whether text begins with the kernel's `[  127.851670]`: a bracket, spaces,
/// digits, a point, six digits, a bracket.
fn starts_with_a_timestamp(text: &str) -> bool {
    let Some(inside) = text.strip_prefix('[') else {
        return false;
    };
    let inside = inside.trim_start_matches(' ');
    let Some((seconds, rest)) = inside.split_once('.') else {
        return false;
    };
    let fraction = rest.get(..6);
    !seconds.is_empty()
        && seconds.chars().all(|it| it.is_ascii_digit())
        && fraction.is_some_and(|it| it.chars().all(|digit| digit.is_ascii_digit()))
        && rest.get(6..7) == Some("]")
}

#[cfg(test)]
#[path = "the_log_tests.rs"]
mod tests;
