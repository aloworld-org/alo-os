//! **The desktop's side of polkit's own helper**: the cookie polkit gave, the
//! password the person typed, and *SUCCESS* or *FAILURE*.
//!
//! polkit asks the desktop's authentication agent to authenticate somebody,
//! and an agent does that by running `polkit-agent-helper-1`, which asks PAM
//! and tells polkit the answer itself. The helper is setuid root; the desktop
//! runs with no capability and no new privileges, so it cannot start it with
//! any effect. It talks to `alo-checking-a-password` instead, which systemd
//! starts as root per connection and which hands the connection to the helper
//! **for the caller's own login** (ADR 0150).
//!
//! # The helper's own conversation
//!
//! Read from polkit 126's source on 2026-11-09
//! (`src/polkitagent/polkitagenthelper-pam.c`, `polkitagenthelperprivate.c`):
//! the cookie is one line on its standard input; each PAM question arrives as
//! `PAM_PROMPT_ECHO_OFF <text>` or `PAM_PROMPT_ECHO_ON <text>` and is answered
//! with one line; `PAM_ERROR_MSG` and `PAM_TEXT_INFO` are said and not
//! answered; and it ends with `SUCCESS` or `FAILURE`.
//!
//! **The password answers one hidden question and nothing else.** A second
//! hidden question — a password that has expired asking for a new one — and
//! any question asked with echo on are not something the person was asked, so
//! the conversation is ended rather than answered with their password.

use std::io::{BufRead, Read, Write};

/// Where `alo-checking-a-password` listens.
pub const THE_CHECKERS_SOCKET: &str = "/run/alo-checking-a-password.socket";

/// The most a line from the helper may be before it is no conversation.
const THE_LONGEST_LINE: u64 = 4096;

/// What the helper said about the password.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Checked {
    /// It was the person's, and polkit has been told.
    Yes,
    /// It was not.
    No,
}

/// Why no answer came about the password.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotChecked {
    /// The checker could not be reached.
    #[error("the password checker could not be reached: {0}")]
    NoChecker(String),
    /// The cookie could not be one line.
    #[error("polkit's cookie was not one line")]
    NotACookie,
    /// The helper asked something other than the one password.
    #[error("the helper asked for more than the password")]
    AskedForMore,
    /// The helper ended, or said something that is not its conversation.
    #[error("the helper ended without an answer: {0}")]
    Ended(String),
}

/// **One password, checked**: `cookie` sent, `password` answered to the one
/// hidden question, and the helper's verdict read.
///
/// # Errors
/// [`NotChecked`], each saying why there is no verdict. A wrong password is
/// not an error: it is [`Checked::No`].
pub fn checked(
    reading: &mut impl BufRead,
    writing: &mut impl Write,
    cookie: &str,
    password: &str,
) -> Result<Checked, NotChecked> {
    if cookie.is_empty() || cookie.contains(['\n', '\r', '\0']) {
        return Err(NotChecked::NotACookie);
    }
    let ended = |why: std::io::Error| NotChecked::Ended(why.to_string());
    writing
        .write_all(cookie.as_bytes())
        .and_then(|()| writing.write_all(b"\n"))
        .and_then(|()| writing.flush())
        .map_err(ended)?;
    let mut answered = false;
    loop {
        let mut line = String::new();
        let read = Read::take(&mut *reading, THE_LONGEST_LINE)
            .read_line(&mut line)
            .map_err(ended)?;
        if read == 0 {
            return Err(NotChecked::Ended("it closed".to_owned()));
        }
        let said = line.trim_end_matches(['\n', '\r']);
        let (word, _) = said.split_once(' ').unwrap_or((said, ""));
        match word {
            "SUCCESS" => return Ok(Checked::Yes),
            "FAILURE" => return Ok(Checked::No),
            "PAM_ERROR_MSG" | "PAM_TEXT_INFO" => {}
            "PAM_PROMPT_ECHO_OFF" if !answered => {
                answered = true;
                writing
                    .write_all(password.as_bytes())
                    .and_then(|()| writing.write_all(b"\n"))
                    .and_then(|()| writing.flush())
                    .map_err(ended)?;
            }
            "PAM_PROMPT_ECHO_OFF" | "PAM_PROMPT_ECHO_ON" => return Err(NotChecked::AskedForMore),
            _ => return Err(NotChecked::Ended("it said something else".to_owned())),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The helper's side, played: what it says, in order.
    fn the_helper_says(lines: &str, cookie: &str, password: &str) -> (Result<Checked, NotChecked>, String) {
        let mut reading = std::io::Cursor::new(lines.as_bytes().to_vec());
        let mut written = Vec::new();
        let came = checked(&mut reading, &mut written, cookie, password);
        (came, String::from_utf8(written).unwrap())
    }

    /// **The right password is the cookie, then the password, then SUCCESS.**
    #[test]
    fn the_right_password_is_yes() {
        let (came, written) = the_helper_says(
            "PAM_PROMPT_ECHO_OFF Password: \nSUCCESS\n",
            "cookie-1",
            "1234",
        );
        assert_eq!(came, Ok(Checked::Yes));
        assert_eq!(written, "cookie-1\n1234\n");
    }

    /// **A wrong password is no, not an error**, and what PAM says along the
    /// way is heard and not answered.
    #[test]
    fn a_wrong_password_is_no() {
        let (came, written) = the_helper_says(
            "PAM_PROMPT_ECHO_OFF Password: \nPAM_ERROR_MSG Authentication failure\nFAILURE\n",
            "c",
            "4321",
        );
        assert_eq!(came, Ok(Checked::No));
        assert_eq!(written, "c\n4321\n");
    }

    /// **The password answers one hidden question and nothing else.** A second
    /// one, or one asked aloud, ends the conversation without it.
    #[test]
    fn the_password_answers_one_hidden_question_only() {
        let (came, written) = the_helper_says(
            "PAM_PROMPT_ECHO_OFF Password: \nPAM_PROMPT_ECHO_OFF New password: \n",
            "c",
            "1234",
        );
        assert_eq!(came, Err(NotChecked::AskedForMore));
        assert_eq!(written, "c\n1234\n", "said once, never twice");
        let (came, written) = the_helper_says("PAM_PROMPT_ECHO_ON login: \n", "c", "1234");
        assert_eq!(came, Err(NotChecked::AskedForMore));
        assert_eq!(written, "c\n", "never said aloud");
    }

    /// **A helper that ends or babbles is no verdict**, and a cookie that is not
    /// one line is never sent.
    #[test]
    fn no_verdict_is_said_as_none() {
        assert!(matches!(the_helper_says("", "c", "1").0, Err(NotChecked::Ended(_))));
        assert!(matches!(the_helper_says("HELLO\n", "c", "1").0, Err(NotChecked::Ended(_))));
        let (came, written) = the_helper_says("SUCCESS\n", "a\nb", "1");
        assert_eq!(came, Err(NotChecked::NotACookie));
        assert_eq!(written, "");
    }
}
