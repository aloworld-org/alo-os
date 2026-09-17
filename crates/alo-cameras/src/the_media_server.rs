//! This machine's own media server, asked what it can see with.
//!
//! The same shape `alo-sound` uses for the same server, and the reason is the
//! same: the server's own tool is part of what we rent, it prints exactly what
//! the server holds, and it keeps a C library out of every process that wants to
//! draw a list of cameras.
//!
//! **Three crates now reach this server through their own copy of these twenty
//! lines** — this one, `alo-sound`, and `alo-in-use`. That is worth a sentence
//! rather than a shrug: none of them owns *reaching the media server*, and a
//! crate that did would be a better home for it than three copies. It is a
//! proposal for whoever holds the media stack, not a change to make from inside
//! one plan.

use std::io::ErrorKind;
use std::process::{Command, Stdio};

use crate::seen::{self, NotSeen, Seen};

/// The server's own tool for writing out its record.
const READ_WITH: &str = "pw-dump";

/// Where a machine's own programs are, for a cleared environment.
const WHERE_ITS_PROGRAMS_ARE: &str = "/usr/bin:/bin";

/// Where the media server is listening, which a session sets.
const WHERE_THE_SERVER_IS: &str = "XDG_RUNTIME_DIR";

/// **What this machine can see with**, asked.
pub trait Cameras {
    /// The cameras this machine has now.
    ///
    /// # Errors
    /// [`NotSeen`] where there is no media server, it would not answer, or it
    /// answered something this crate cannot read. Never an empty list standing
    /// in for a question that could not be asked.
    fn now(&self) -> Result<Seen, NotSeen>;
}

/// **This machine's media server.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheMediaServer {
    /// The program asked for the record.
    reading: String,
}

impl Default for TheMediaServer {
    fn default() -> Self {
        Self {
            reading: READ_WITH.to_owned(),
        }
    }
}

impl TheMediaServer {
    /// The media server on the machine this is running on.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self::default()
    }
}

impl Cameras for TheMediaServer {
    fn now(&self) -> Result<Seen, NotSeen> {
        let answered = Command::new(&self.reading)
            .env_clear()
            .env("LC_ALL", "C")
            .env("LANG", "C")
            .env("PATH", WHERE_ITS_PROGRAMS_ARE)
            .envs(
                std::env::var_os(WHERE_THE_SERVER_IS)
                    .map(|where_it_is| (WHERE_THE_SERVER_IS.to_owned(), where_it_is)),
            )
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();
        match answered {
            Ok(output) if output.status.success() => {
                seen::in_the_record(&String::from_utf8_lossy(&output.stdout))
            }
            Ok(output) => Err(NotSeen::NoAnswer(format!(
                "{} failed: {}",
                self.reading,
                String::from_utf8_lossy(&output.stderr).trim()
            ))),
            Err(why) if why.kind() == ErrorKind::NotFound => Err(NotSeen::NothingAnswers(format!(
                "{} is not on this machine",
                self.reading
            ))),
            Err(why) => Err(NotSeen::NoAnswer(why.to_string())),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A machine with no media server says so rather than reading as a
    /// machine with no cameras**, which is the difference between *you have no
    /// camera* and *this could not be asked*.
    #[test]
    fn a_machine_with_nothing_to_ask_refuses_rather_than_reading_as_no_cameras() {
        let nowhere = TheMediaServer {
            reading: "alo-cameras-no-such-tool".to_owned(),
        };
        let why = nowhere.now().expect_err("there is no such tool");
        assert!(matches!(why, NotSeen::NothingAnswers(_)));
        assert!(why.to_string().contains("alo-cameras-no-such-tool"));
    }
}
