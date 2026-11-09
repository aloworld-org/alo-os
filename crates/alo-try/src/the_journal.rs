//! **Where an account meant for whoever administers the machine goes**: the
//! system journal, in its own native form, rather than the person's terminal.
//!
//! The owner's decision of 2026-11-06 for a `try` the sealed box cannot run
//! here is *say so plainly* — one sentence, which `crate::when_the_box_cannot_run_it`
//! words. The kernel's account of the refusal is still worth keeping: it names
//! the control group and the error, which is what somebody fixing the box needs.
//! So it is written here, under `SYSLOG_IDENTIFIER=alo-try`, where
//! `journalctl -t alo-try` finds it.
//!
//! # The form
//!
//! systemd's native protocol: one datagram to [`THE_JOURNALS_SOCKET`] holding
//! `FIELD=value` lines. A value with a line break needs the binary form, and an
//! account is one line, so a line break in it becomes a space instead — which
//! also means nothing in an account can add a field of its own.
//!
//! # When the journal is not there
//!
//! [`written_to_the_journal`] says so, and the caller says the account on the
//! terminal instead: an account that could not be kept anywhere is not
//! dropped.

use std::os::unix::net::UnixDatagram;
use std::path::Path;

/// Where systemd's journal takes entries in its native form.
pub const THE_JOURNALS_SOCKET: &str = "/run/systemd/journal/socket";

/// What every entry from this program is found by.
pub const THE_IDENTIFIER: &str = "alo-try";

/// syslog's *warning*: a run the person asked for did not happen.
const A_WARNING: u8 = 4;

/// One journal entry carrying `account`, in the native form.
#[must_use]
pub fn the_entry(account: &str) -> Vec<u8> {
    let account: String = account
        .chars()
        .map(|one| if one == '\n' || one == '\r' { ' ' } else { one })
        .collect();
    format!("SYSLOG_IDENTIFIER={THE_IDENTIFIER}\nPRIORITY={A_WARNING}\nMESSAGE={account}\n")
        .into_bytes()
}

/// Write `account` to the journal listening at `socket`.
///
/// # Errors
/// Whatever the socket answered: no journal there, or one that would not take
/// the entry.
pub fn written_to_the_journal(socket: &Path, account: &str) -> std::io::Result<()> {
    let entry = the_entry(account);
    let sent = UnixDatagram::unbound()?.send_to(&entry, socket)?;
    if sent == entry.len() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::WriteZero,
            "the journal took part of the entry",
        ))
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
#[path = "the_journal_tests.rs"]
mod tests;
