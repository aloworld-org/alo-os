//! **This machine's media server, asked for its record.**
//!
//! [`TheMediaServer`] runs the server's own tool for writing out what it holds,
//! and turns what happens into one of `crate::NotAsked`'s four facts. That is
//! the whole of it: nothing here decides what a node means, and nothing here
//! says anything to a person.
//!
//! # The server's own tool, rather than a library binding
//!
//! It is part of the thing we rent, it prints exactly what the server holds, and
//! it keeps a C library out of every process that wants to draw a status area or
//! a volume slider. A binding would be the same answer with a build dependency
//! and an ABI in front of it. ADR 0011: the media server is rented, configured
//! and never written.
//!
//! # Telling a machine with no server from a server that failed
//!
//! The tool says `can't connect` and then why. That wording is what separates
//! *this machine has no media server running* from *the server would not
//! answer*, and it is a thin thread: where it changes, this falls back to *would
//! not answer*, which is still a refusal and still not an empty list. It is
//! written down rather than hidden because the day it changes, somebody reading
//! this file needs to know what it was resting on.

use std::io::ErrorKind;

use crate::reaching::ATool;
use crate::record::{self, TheRecord};
use crate::refusing::NotAsked;

/// The server's own tool for writing out its record.
pub const THE_TOOL: &str = "pw-dump";

/// **What this machine's media server holds**, asked.
///
/// A trait so that everything above it can be decided against an answer rather
/// than against a machine, and tested on machines with no sound card at all.
pub trait AsksIt {
    /// The record, now.
    ///
    /// # Errors
    /// [`NotAsked`], as one of four facts. Never an empty record standing in for
    /// a question that could not be asked.
    fn record(&self) -> Result<TheRecord, NotAsked>;
}

/// **This machine's media server.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheMediaServer {
    /// The program asked for the record.
    tool: String,
}

impl Default for TheMediaServer {
    fn default() -> Self {
        Self {
            tool: THE_TOOL.to_owned(),
        }
    }
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
            tool: tool.to_owned(),
        }
    }

    /// What the tool answered, turned into a record or into one of the four
    /// facts.
    ///
    /// Split from running it so that every way of answering badly is a test
    /// rather than a paragraph: a machine cannot be made to fail a rented tool
    /// on demand, and reading a failure is the half that has to be right.
    #[must_use = "this is the answer"]
    pub fn what_it_answered(
        &self,
        succeeded: bool,
        said: &[u8],
        instead: &[u8],
    ) -> Result<TheRecord, NotAsked> {
        if succeeded {
            return record::read(&String::from_utf8_lossy(said));
        }
        let instead = String::from_utf8_lossy(instead).trim().to_owned();
        let said = format!("{} failed: {instead}", self.tool);
        if nothing_was_listening(&instead) {
            return Err(NotAsked::NoServerIsRunning { said });
        }
        Err(NotAsked::ItWouldNotAnswer { said })
    }
}

impl AsksIt for TheMediaServer {
    fn record(&self) -> Result<TheRecord, NotAsked> {
        match ATool::named(&self.tool).started_here().output() {
            Ok(output) => {
                self.what_it_answered(output.status.success(), &output.stdout, &output.stderr)
            }
            Err(why) if why.kind() == ErrorKind::NotFound => Err(NotAsked::NothingHandlesIt {
                said: format!("{} is not on this machine", self.tool),
            }),
            Err(why) => Err(NotAsked::ItWouldNotAnswer {
                said: why.to_string(),
            }),
        }
    }
}

/// Whether what the tool said is *there is no server here to talk to*.
fn nothing_was_listening(said: &str) -> bool {
    let said = said.to_lowercase();
    said.contains("can't connect") || said.contains("cannot connect")
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A machine with no such tool says so**, rather than answering an empty
    /// record.
    #[test]
    fn a_machine_with_no_tool_at_all_is_the_first_fact() {
        let nowhere = TheMediaServer::reached_by("alo-media-server-no-such-tool");
        let why = nowhere.record().expect_err("there is no such tool");
        assert!(matches!(why, NotAsked::NothingHandlesIt { .. }));
        assert!(why.is_a_machine_without_one());
        assert!(why.diagnosis().contains("alo-media-server-no-such-tool"));
    }

    /// **A tool that could not connect is a machine with no server running**,
    /// which is an ordinary build host and not a fault.
    #[test]
    fn a_tool_that_could_not_connect_is_the_second_fact() {
        let server = TheMediaServer::on_this_machine();
        for said in [
            "can't connect: Host is down",
            "can't connect: No such file or directory",
            "Cannot connect to PipeWire",
        ] {
            let why = server
                .what_it_answered(false, b"", said.as_bytes())
                .expect_err("nothing is listening");
            assert!(
                matches!(why, NotAsked::NoServerIsRunning { .. }),
                "{said}: {why:?}"
            );
            assert!(why.is_a_machine_without_one());
        }
    }

    /// **A tool that ran and failed for any other reason is the third fact**,
    /// and what it said is kept for whoever is fixing the machine.
    #[test]
    fn a_tool_that_failed_another_way_is_the_third_fact() {
        let server = TheMediaServer::on_this_machine();
        let why = server
            .what_it_answered(false, b"", b"the graph could not be read")
            .expect_err("it failed");
        assert!(matches!(why, NotAsked::ItWouldNotAnswer { .. }));
        assert!(!why.is_a_machine_without_one());
        assert!(why.diagnosis().contains("the graph could not be read"));
        assert!(why.diagnosis().contains(THE_TOOL));
    }

    /// **And an answer that will not read is the fourth**, never an empty
    /// record: a machine whose record would not parse is not a quiet machine.
    #[test]
    fn an_answer_that_will_not_read_is_the_fourth_fact() {
        let server = TheMediaServer::on_this_machine();
        let why = server
            .what_it_answered(true, b"this is not a record", b"")
            .expect_err("that is not a record");
        assert!(matches!(
            why,
            NotAsked::ItAnsweredSomethingUnreadable { .. }
        ));
        assert!(
            !why.is_a_machine_without_one(),
            "an unreadable record read as a machine with no server would take an indicator off a \
             screen while a camera may be on"
        );
    }

    /// **And an answer that reads is a record.**
    #[test]
    fn an_answer_that_reads_is_a_record() {
        let server = TheMediaServer::on_this_machine();
        let record = server
            .what_it_answered(
                true,
                br#"[{"id": 1, "type": "PipeWire:Interface:Node"}]"#,
                b"",
            )
            .expect("a record");
        assert_eq!(record.how_many(), 1);
    }
}
