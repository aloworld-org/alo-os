//! **Reading the machine's serial log**: the lines said after a point, and the
//! first of them that carries a phrase.
//!
//! The image sends the journal to the serial port (`console.toml` and the
//! journald drop-in in `image/`), so a service's own sentence — *Started
//! alo-screen-reader.service*, *8 of this machine's applications arrived* — is
//! a line here. A point in the log is a byte offset, taken from the log's
//! length at the moment a walk marks it, so *since the mark* means exactly the
//! bytes written after it.

/// The lines said after `from` bytes, as text; a byte that is not UTF-8 reads
/// as a replacement character rather than hiding the line.
#[must_use]
pub fn lines_after(log: &[u8], from: usize) -> Vec<String> {
    let after = log.get(from..).unwrap_or_default();
    String::from_utf8_lossy(after)
        .lines()
        .map(|line| line.trim_end_matches('\r').to_owned())
        .collect()
}

/// The first line after `from` that carries `phrase`.
#[must_use]
pub fn first_with(log: &[u8], from: usize, phrase: &str) -> Option<String> {
    lines_after(log, from)
        .into_iter()
        .find(|line| line.contains(phrase))
}

/// Every line after `from` that carries any of `phrases`.
#[must_use]
pub fn all_with(log: &[u8], from: usize, phrases: &[String]) -> Vec<String> {
    lines_after(log, from)
        .into_iter()
        .filter(|line| phrases.iter().any(|phrase| line.contains(phrase.as_str())))
        .collect()
}

#[cfg(test)]
#[path = "the_log_tests.rs"]
mod tests;
