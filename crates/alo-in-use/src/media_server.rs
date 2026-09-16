//! The machine's own media server, reached.
//!
//! [`TheMediaServer`] is [`crate::Streams`] on a real machine: it asks the
//! rented media server for its record of the graph and hands the answer to
//! [`crate::heard`]. That is the whole of it, on purpose — nothing is decided
//! here, and nothing about the server is patched, wrapped or extended (ADR
//! 0011: the media server is rented, configured and never written).
//!
//! The record is asked for through the server's own tool rather than through a
//! library binding, and that is a decision worth stating. The tool is part of
//! the thing we rent, it prints exactly the record the server holds, and it
//! keeps this crate from linking a C library into every process that wants to
//! draw a status area. A binding would be the same record with a build
//! dependency and an ABI in front of it.
//!
//! # No shell, no environment of the caller's, and the C locale
//!
//! The program is started directly, so no argument is interpreted by anything
//! but the tool. Its environment is cleared and given back only what it needs
//! to find the machine's own installation and to answer in the C locale: what
//! it prints is read to decide things, and a translated field name would be a
//! record nothing recognised. The shape is `alo_software::TheRentedTool`'s,
//! copied rather than re-decided.
//!
//! # A machine with no media server says so
//!
//! Three different things can go wrong and they are three different sentences,
//! because a person fixes them differently and because *the indicator could not
//! answer* must never look like *nothing is watching*: the tool is not there
//! ([`crate::NotHeard::NothingHandlesSoundAndVideo`]), it is there and failed
//! ([`crate::NotHeard::NoAnswer`]), or it answered something unreadable
//! ([`crate::NotHeard::NotUnderstood`]).

use std::io::ErrorKind;
use std::process::{Command, Stdio};

use crate::heard;
use crate::refusing::NotHeard;
use crate::streams::Streams;
use crate::uses::Use;

/// The rented media server's own tool for writing out its record.
const THE_TOOL: &str = "pw-dump";

/// Where a machine's own programs are, for a cleared environment.
const WHERE_ITS_PROGRAMS_ARE: &str = "/usr/bin:/bin";

/// This machine's media server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheMediaServer {
    /// The program asked for the record.
    program: String,
}

impl Default for TheMediaServer {
    fn default() -> Self {
        Self {
            program: THE_TOOL.to_owned(),
        }
    }
}

impl TheMediaServer {
    /// The media server on the machine this is running on.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self::default()
    }

    /// The same, reached through a program named here.
    ///
    /// Test-only, and deliberately: a machine has one media server, and a
    /// public way of pointing the indicator at another program would be a
    /// second answer to *is my camera on*.
    #[cfg(test)]
    fn reached_by(program: &str) -> Self {
        Self {
            program: program.to_owned(),
        }
    }

    /// What the tool answered, turned into uses.
    ///
    /// Split from starting it so that every way of answering badly is a test
    /// rather than a paragraph: a machine cannot be made to fail a rented tool
    /// on demand, and the reading of a failure is the half that has to be
    /// right.
    fn what_it_answered(
        &self,
        succeeded: bool,
        said: &[u8],
        instead: &[u8],
    ) -> Result<Vec<Use>, NotHeard> {
        if !succeeded {
            return Err(NotHeard::NoAnswer {
                said: format!(
                    "{} failed: {}",
                    self.program,
                    String::from_utf8_lossy(instead).trim()
                ),
            });
        }
        heard::in_use_in(&String::from_utf8_lossy(said))
    }
}

impl Streams for TheMediaServer {
    fn in_use_now(&mut self) -> Result<Vec<Use>, NotHeard> {
        let answered = Command::new(&self.program)
            .env_clear()
            .env("LC_ALL", "C")
            .env("LANG", "C")
            .env("PATH", WHERE_ITS_PROGRAMS_ARE)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();
        match answered {
            Ok(output) => {
                self.what_it_answered(output.status.success(), &output.stdout, &output.stderr)
            }
            Err(why) if why.kind() == ErrorKind::NotFound => {
                Err(NotHeard::NothingHandlesSoundAndVideo {
                    said: format!("{} is not on this machine: {why}", self.program),
                })
            }
            Err(why) => Err(NotHeard::NoAnswer {
                said: format!("{} could not be started: {why}", self.program),
            }),
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
    use crate::used::Used;

    /// **A machine with nothing handling sound and video says so**, and says it
    /// in its own sentence rather than by showing an empty indicator. This is
    /// the refusal every machine without a media server takes, including every
    /// machine this test suite runs on.
    #[test]
    fn a_machine_with_no_media_server_refuses_rather_than_looking_quiet() {
        let mut nowhere = TheMediaServer::reached_by("alo-there-is-no-such-program");
        let refused = nowhere.in_use_now().unwrap_err();
        assert!(
            matches!(refused, NotHeard::NothingHandlesSoundAndVideo { .. }),
            "{refused:?}"
        );
        assert!(refused.diagnosis().contains("alo-there-is-no-such-program"));
    }

    /// **A record that came back is read**, and is the only thing that is.
    #[test]
    fn a_record_that_came_back_is_read() {
        let server = TheMediaServer::on_this_machine();
        let record = br#"[{"id": 31, "type": "PipeWire:Interface:Node",
             "info": {"state": "running", "props": {"media.class": "Audio/Source"}}}]"#;
        let in_use = server.what_it_answered(true, record, b"").unwrap();
        assert_eq!(in_use.len(), 1);
        assert_eq!(in_use.first().map(Use::what), Some(Used::Microphone));
    }

    /// **A tool that ran and failed is not an answer**, and what it said is
    /// kept for whoever is fixing it.
    #[test]
    fn a_tool_that_ran_and_failed_is_refused_with_what_it_said() {
        let server = TheMediaServer::on_this_machine();
        let refused = server
            .what_it_answered(false, b"", b"cannot connect to the daemon\n")
            .unwrap_err();
        assert!(matches!(refused, NotHeard::NoAnswer { .. }), "{refused:?}");
        assert!(refused.diagnosis().contains("cannot connect to the daemon"));
        assert!(refused.diagnosis().contains(THE_TOOL));
    }

    /// **A tool that answered nonsense is refused too**, rather than read as a
    /// machine on which nothing is watching.
    #[test]
    fn a_tool_that_answered_nonsense_is_refused() {
        let server = TheMediaServer::on_this_machine();
        let refused = server
            .what_it_answered(true, b"this is not a record", b"")
            .unwrap_err();
        assert!(
            matches!(refused, NotHeard::NotUnderstood { .. }),
            "{refused:?}"
        );
    }

    /// The machine's media server is reached through the rented tool and
    /// through nothing else — the one place that name appears.
    #[test]
    fn the_machines_media_server_is_the_rented_tool() {
        assert_eq!(TheMediaServer::on_this_machine().program, THE_TOOL);
        assert_eq!(TheMediaServer::default(), TheMediaServer::on_this_machine());
    }
}
