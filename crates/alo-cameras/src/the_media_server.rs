//! This machine's media server, asked what it can see with.
//!
//! The record — running the tool, clearing its environment, passing the runtime
//! directory through, reading an answer that may arrive as more than one list —
//! is `alo-media-server`'s. This is the half that is this crate's: turning that
//! crate's four facts into the two sentences a list of cameras needs.

use alo_media_server::{AsksIt, NotAsked};

use crate::seen::{self, NotSeen, Seen};

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
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TheMediaServer {
    /// The server, asked by the crate that owns asking.
    asking: alo_media_server::TheMediaServer,
}

impl TheMediaServer {
    /// The media server on the machine this is running on.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self::default()
    }

    /// The same, reached through a program named here — for a test standing in
    /// for a machine.
    #[doc(hidden)]
    #[must_use]
    pub fn reached_by(tool: &str) -> Self {
        Self {
            asking: alo_media_server::TheMediaServer::reached_by(tool),
        }
    }
}

impl Cameras for TheMediaServer {
    fn now(&self) -> Result<Seen, NotSeen> {
        let record = self.asking.record().map_err(what_it_means)?;
        seen::in_the_record(record.objects())
    }
}

/// What one of `alo-media-server`'s four facts means to a list of cameras.
///
/// A machine with no tool and a machine with no server running are one sentence
/// here — neither can say what this machine sees with — and which it was stays
/// in the diagnosis for whoever is fixing it.
fn what_it_means(why: NotAsked) -> NotSeen {
    let said = why.said().to_owned();
    match why {
        NotAsked::NothingHandlesIt { .. } | NotAsked::NoServerIsRunning { .. } => {
            NotSeen::NothingAnswers(said)
        }
        NotAsked::ItWouldNotAnswer { .. } => NotSeen::NoAnswer(said),
        NotAsked::ItAnsweredSomethingUnreadable { .. } => NotSeen::NotUnderstood(said),
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A machine with nothing to ask says so rather than reading as a machine
    /// with no cameras**, which is the difference between *you have no camera*
    /// and *this could not be asked*.
    #[test]
    fn a_machine_with_nothing_to_ask_refuses_rather_than_reading_as_no_cameras() {
        let nowhere = TheMediaServer::reached_by("alo-cameras-no-such-tool");
        let why = nowhere.now().expect_err("there is no such tool");
        assert!(matches!(why, NotSeen::NothingAnswers(_)));
        assert!(why.to_string().contains("alo-cameras-no-such-tool"));
    }

    /// **And each of the four facts becomes the sentence it should.**
    #[test]
    fn the_four_facts_become_the_sentences_a_list_of_cameras_needs() {
        let said = || "because".to_owned();
        assert!(matches!(
            what_it_means(NotAsked::NoServerIsRunning { said: said() }),
            NotSeen::NothingAnswers(_)
        ));
        assert!(matches!(
            what_it_means(NotAsked::ItWouldNotAnswer { said: said() }),
            NotSeen::NoAnswer(_)
        ));
        assert!(matches!(
            what_it_means(NotAsked::ItAnsweredSomethingUnreadable { said: said() }),
            NotSeen::NotUnderstood(_)
        ));
    }
}
