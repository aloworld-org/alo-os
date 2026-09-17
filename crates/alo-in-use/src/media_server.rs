//! The machine's own media server, reached — through the crate that owns
//! reaching it.
//!
//! [`TheMediaServer`] is [`crate::Streams`] on a real machine: it asks
//! `alo-media-server` for the record and hands it to [`crate::heard`]. That is
//! the whole of it, on purpose — nothing is decided here, and nothing about the
//! server is patched, wrapped or extended (ADR 0011).
//!
//! # What moved out of this file, and why
//!
//! Starting the tool, clearing its environment, passing the runtime directory
//! through, and reading a record that may arrive as more than one list all used
//! to be here, in a copy shared with `alo-sound`, `alo-cameras` and
//! `alo-capturing`. The copies drifted: this one did not pass
//! `XDG_RUNTIME_DIR`, and spent a day telling a machine with a working server
//! that its server would not answer. They are `alo-media-server`'s now, and
//! there is one of them.
//!
//! # The four facts, turned into this crate's three
//!
//! `alo-media-server` tells four things apart: no tool, no server running, a
//! server that would not answer, and an answer that will not read. This crate
//! has three sentences, because two of those are one thing to a **person**:
//! either way, nothing on this machine is handling sound and video, and there is
//! nothing to watch or listen. The difference is for whoever is fixing the
//! machine, and it is kept in the diagnosis beside the sentence rather than
//! thrown away.

use alo_media_server::{AsksIt, NotAsked};

use crate::heard;
use crate::refusing::NotHeard;
use crate::streams::Streams;
use crate::uses::Use;

/// This machine's media server.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TheMediaServer {
    /// The server, reached.
    asking: alo_media_server::TheMediaServer,
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
            asking: alo_media_server::TheMediaServer::reached_by(program),
        }
    }
}

impl Streams for TheMediaServer {
    fn in_use_now(&mut self) -> Result<Vec<Use>, NotHeard> {
        let record = self.asking.record().map_err(what_it_means)?;
        heard::in_use_in(record.objects())
    }
}

/// What one of `alo-media-server`'s four facts means to an indicator.
///
/// Two of them are one sentence here and that is deliberate: *there is no such
/// tool* and *nothing is listening* are both **nothing on this machine handles
/// sound and video**, which is what a person needs to read. Which of the two it
/// was stays in the diagnosis, for whoever is fixing the machine.
fn what_it_means(why: NotAsked) -> NotHeard {
    let said = why.said().to_owned();
    match why {
        NotAsked::NothingHandlesIt { .. } | NotAsked::NoServerIsRunning { .. } => {
            NotHeard::NothingHandlesSoundAndVideo { said }
        }
        NotAsked::ItWouldNotAnswer { .. } => NotHeard::NoAnswer { said },
        NotAsked::ItAnsweredSomethingUnreadable { .. } => NotHeard::NotUnderstood { said },
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A machine with no media server is not an empty indicator.**
    #[test]
    fn a_machine_with_nothing_to_ask_says_so_rather_than_reading_as_a_quiet_room() {
        let mut nowhere = TheMediaServer::reached_by("alo-in-use-no-such-tool");
        let why = nowhere
            .in_use_now()
            .expect_err("there is no such tool on this machine");
        assert!(matches!(why, NotHeard::NothingHandlesSoundAndVideo { .. }));
        assert!(why.diagnosis().contains("alo-in-use-no-such-tool"));
    }

    /// **Each of the four facts becomes the sentence a person should read.**
    #[test]
    fn the_four_facts_become_three_sentences_and_none_of_them_is_silence() {
        let said = || "because".to_owned();
        for (fact, sentence) in [
            (
                NotAsked::NothingHandlesIt { said: said() },
                "nothing handles it",
            ),
            (
                NotAsked::NoServerIsRunning { said: said() },
                "nothing handles it",
            ),
            (NotAsked::ItWouldNotAnswer { said: said() }, "no answer"),
            (
                NotAsked::ItAnsweredSomethingUnreadable { said: said() },
                "not understood",
            ),
        ] {
            let meant = what_it_means(fact);
            let read = match meant {
                NotHeard::NothingHandlesSoundAndVideo { .. } => "nothing handles it",
                NotHeard::NoAnswer { .. } => "no answer",
                NotHeard::NotUnderstood { .. } => "not understood",
            };
            assert_eq!(read, sentence);
        }
    }
}
