//! **A secret typed on the machine never leaves in what the walk keeps.**
//!
//! A walk that types an AI provider's key types it into a field that draws
//! dots, and the report says only how many characters were typed. But the
//! serial log is the machine's journal, and nothing in a walk can promise that
//! no service on the machine ever writes what it was given. So the log the
//! walk keeps is a copy with every secret it typed taken out — the one kept on
//! the runner is raw and is never uploaded.

/// What a secret is replaced with.
pub const REMOVED: &[u8] = b"[a secret the walk typed, removed]";

/// `log` with every occurrence of each secret replaced by [`REMOVED`]. An
/// empty secret removes nothing.
#[must_use]
pub fn without(log: &[u8], secrets: &[String]) -> Vec<u8> {
    let mut kept = log.to_vec();
    for secret in secrets {
        let secret = secret.as_bytes();
        if secret.is_empty() {
            continue;
        }
        let mut out = Vec::with_capacity(kept.len());
        let mut at = 0;
        while at < kept.len() {
            if kept.get(at..at + secret.len()) == Some(secret) {
                out.extend_from_slice(REMOVED);
                at += secret.len();
            } else {
                out.extend(kept.get(at).copied());
                at += 1;
            }
        }
        kept = out;
    }
    kept
}

#[cfg(test)]
#[path = "redacting_tests.rs"]
mod tests;
