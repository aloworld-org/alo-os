//! **The person's password, kept in step with the system's list**
//! (ADR 0150, the owner's *"Keep both in step"*).
//!
//! Sign-in is checked against alo's own accounts (ADR 0024); the system's list,
//! `/etc/shadow`, is what polkit and `sudo` check. So at every sign-in that
//! succeeded, the sign-in screen sends the person's number and the password they
//! just typed to `alo-keeping-the-system-password`, which writes it into the
//! system's list through `chpasswd` and answers whether it did.
//!
//! **At every sign-in, not only when an account is made or a password
//! changes**, because that is what makes a machine already in use come into
//! step on its own: its system password is locked (`!*`, what `sysusers`
//! leaves), and the first sign-in after the update writes it. Nothing can ask
//! for the password before that sign-in, because nothing can ask without a
//! desktop to draw the sheet (ADR 0150, *Machines already in use*).
//!
//! # The line, and the answer
//!
//! The screen sends `<number>\n<password>\n`; the keeper answers one line,
//! [`answer_of`]: `kept`, or `not-kept <why>`. The password is never written
//! anywhere but `chpasswd`'s standard input, and never into a log.

use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

/// Where the keeper listens: a socket only root may open
/// (`image/usr/lib/systemd/system/alo-keeping-the-system-password.socket`).
pub const THE_KEEPERS_SOCKET: &str = "/run/alo-keeping-the-system-password.socket";

/// The system's own tool for writing a password into its list.
pub const CHPASSWD: &str = "/usr/sbin/chpasswd";

/// How `chpasswd` is told to hash: the base's own method
/// (`ENCRYPT_METHOD YESCRYPT` in `/etc/login.defs`), named so that `chpasswd`
/// hashes it itself and does not ask PAM, whose password-quality rules would
/// refuse a four-digit PIN the person signs in with every day. Measured on
/// 2026-11-09 in `localhost/alo-os:dev`: `chpasswd -c YESCRYPT` wrote a
/// `$y$` hash for `1234` holding `CAP_DAC_OVERRIDE` alone.
pub const THE_METHOD: [&str; 2] = ["-c", "YESCRYPT"];

/// The most bytes a password may have: the sign-in screen's own bound
/// (`alo_shell`'s `PASSWORD_BYTES`).
pub const THE_LONGEST_PASSWORD: usize = 1024;

/// The most bytes the screen's line may have: a number, the password and two
/// line ends.
pub const THE_LONGEST_LINE: usize = 16 + THE_LONGEST_PASSWORD + 2;

/// How long the sign-in screen waits for the keeper, at most.
pub const THE_LONGEST_IT_WAITS: Duration = Duration::from_secs(10);

/// **Why the system's list was not given the password.** Each is a word on
/// the wire and in the state file (`crate::in_step`), and names no password.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotKept {
    /// Somebody other than root asked.
    #[error("only the sign-in screen may ask, and this caller is not root")]
    NotTheSignInScreen,
    /// The number is not an ordinary person's login.
    #[error("that number is not a person's login on this machine")]
    NotAPerson,
    /// What arrived was not a number and a password on two lines.
    #[error("what arrived was not a number and a password")]
    NotTwoLines,
    /// The password has a character the system's list cannot hold, or none.
    #[error("the password has a character the system's list cannot hold")]
    CannotHoldIt,
    /// `chpasswd` would not start.
    #[error("chpasswd would not start")]
    DidNotStart,
    /// `chpasswd` started and refused.
    #[error("chpasswd refused")]
    Refused,
    /// The keeper could not be reached, or did not answer.
    #[error("the keeper did not answer")]
    NoAnswer,
}

impl NotKept {
    /// Every reason, for the tests that read each back.
    pub const EVERY_ONE: [Self; 7] = [
        Self::NotTheSignInScreen,
        Self::NotAPerson,
        Self::NotTwoLines,
        Self::CannotHoldIt,
        Self::DidNotStart,
        Self::Refused,
        Self::NoAnswer,
    ];

    /// The word this reason is on the wire and in the state file.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::NotTheSignInScreen => "not-the-sign-in-screen",
            Self::NotAPerson => "not-a-person",
            Self::NotTwoLines => "not-two-lines",
            Self::CannotHoldIt => "cannot-hold-it",
            Self::DidNotStart => "did-not-start",
            Self::Refused => "refused",
            Self::NoAnswer => "no-answer",
        }
    }

    /// The reason a word names, if it names one.
    #[must_use]
    pub fn of_word(word: &str) -> Option<Self> {
        Self::EVERY_ONE.into_iter().find(|why| why.word() == word)
    }
}

/// **The number and the password in what the screen sent**, or why it is not
/// one. Borrowed, so the password is never copied here.
///
/// # Errors
/// [`NotKept::NotTwoLines`] for anything but `<number>\n<password>\n`, and
/// [`NotKept::CannotHoldIt`] for a password that is empty, longer than
/// [`THE_LONGEST_PASSWORD`], or holds a `:`, a line end or a NUL — the
/// characters the system's list and `chpasswd`'s own line are divided by.
pub fn what_was_sent(sent: &str) -> Result<(u32, &str), NotKept> {
    let rest = sent.strip_suffix('\n').ok_or(NotKept::NotTwoLines)?;
    let (number, password) = rest.split_once('\n').ok_or(NotKept::NotTwoLines)?;
    let number = number.parse::<u32>().map_err(|_| NotKept::NotTwoLines)?;
    the_system_can_hold(password)?;
    Ok((number, password))
}

/// Whether the system's list can hold `password` as it is.
///
/// # Errors
/// [`NotKept::CannotHoldIt`], as [`what_was_sent`] says.
pub fn the_system_can_hold(password: &str) -> Result<(), NotKept> {
    if password.is_empty()
        || password.len() > THE_LONGEST_PASSWORD
        || password.contains([':', '\n', '\r', '\0'])
    {
        return Err(NotKept::CannotHoldIt);
    }
    Ok(())
}

/// The one line the keeper answers with.
#[must_use]
pub fn answer_of(kept: Result<(), NotKept>) -> String {
    match kept {
        Ok(()) => "kept\n".to_owned(),
        Err(why) => format!("not-kept {}\n", why.word()),
    }
}

/// What a keeper's answer says.
///
/// # Errors
/// The reason it gave, and [`NotKept::NoAnswer`] for a line that is neither.
pub fn answered(line: &str) -> Result<(), NotKept> {
    match line.trim_end() {
        "kept" => Ok(()),
        said => said
            .strip_prefix("not-kept ")
            .and_then(NotKept::of_word)
            .map_or(Err(NotKept::NoAnswer), Err),
    }
}

/// **The sign-in screen's side**: send `person` and `password` to the keeper at
/// `socket` and read its answer, waiting at most [`THE_LONGEST_IT_WAITS`].
///
/// # Errors
/// What the keeper answered, and [`NotKept::NoAnswer`] when it could not be
/// reached or said nothing; [`NotKept::CannotHoldIt`] before anything is sent.
pub fn handed_to_the_keeper(socket: &Path, person: u32, password: &str) -> Result<(), NotKept> {
    the_system_can_hold(password)?;
    let mut stream = UnixStream::connect(socket).map_err(|_| NotKept::NoAnswer)?;
    stream
        .set_read_timeout(Some(THE_LONGEST_IT_WAITS))
        .and_then(|()| stream.set_write_timeout(Some(THE_LONGEST_IT_WAITS)))
        .map_err(|_| NotKept::NoAnswer)?;
    // Three writes rather than one formatted line, so the password is never
    // copied into a buffer of this function's own.
    stream
        .write_all(format!("{person}\n").as_bytes())
        .and_then(|()| stream.write_all(password.as_bytes()))
        .and_then(|()| stream.write_all(b"\n"))
        .and_then(|()| stream.flush())
        .map_err(|_| NotKept::NoAnswer)?;
    let mut line = String::new();
    BufReader::new(stream.take(256))
        .read_line(&mut line)
        .map_err(|_| NotKept::NoAnswer)?;
    answered(&line)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "keeping_tests.rs"]
mod tests;
