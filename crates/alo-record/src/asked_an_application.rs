//! **A person's own command asked an application to do something**, and what
//! came of it (`docs/contracts/terminal-commands.md`, section 4).
//!
//! Written by `alo-app` itself, to the person's own record file — the file
//! `alo-try` already writes — because a record file has one writer and the
//! process that did the work is the one that writes it down (ADR 0125).
//!
//! **No agent, and no field for one**: the person acted. When alo runs
//! `alo-app` on a person's behalf, that is alo's turn and alo's `ran` entry,
//! never this one.
//!
//! **Changes and refusals, never looks** (the owner's answer 5). Listing what
//! is installed, showing what an application offers, listing its controls and
//! reading its windows write nothing, because a record of what a person looked
//! at is a record of the person. So there is no constructor here for a look.
//!
//! **A refusal keeps the ability and why, and not the arguments**, for
//! `record-file.md`'s reason: what never validated is whatever arrived.
//!
//! **What the application sent while the command ran** is kept as the desktop
//! counted it (ADR 0148) — or why it could not be counted. The command does
//! not report an egress of its own (ADR 0125); it writes down the number the
//! service that showed it answered.

use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::entry::Entry;
use crate::happened::Happened;
use crate::line::Line;

/// Which road carried the act: ADR 0001 §6's, best first, and the window's own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Road {
    /// An ability an adapter declares, over the application's own interface.
    Declared,
    /// An ability the application publishes about itself on D-Bus.
    Discovered,
    /// A control, through the accessibility tree.
    Tree,
    /// A control, by reading the screen — the least reliable road, and every
    /// reader of the record shows it as such.
    Screen,
    /// The window's own open, focus or close, through the compositor.
    Window,
}

/// What the application, or the machine, answered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Answered {
    /// It was done.
    Done,
    /// It was refused, in the sentence the person saw.
    Refused {
        /// The sentence.
        said: Line,
    },
    /// The request was sent and nothing answered — recorded as having run,
    /// because it may have.
    NoAnswer,
}

/// What the application sent while the command ran, as the desktop counted it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SentDuring {
    /// Counted for the whole command: this many bytes, headers included.
    Counted {
        /// How many.
        bytes: u64,
    },
    /// Counted for only part of it — the desktop's limit was reached, or part
    /// of the application could not be counted — so this is at least what it
    /// sent.
    AtLeast {
        /// How many were counted.
        bytes: u64,
    },
    /// Not counted at all, and why, in the words the person was shown.
    NotCounted {
        /// Why.
        why: Line,
    },
}

/// One argument an ability was given: its declared name and the value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Given {
    /// The argument's name, as the ability declares it.
    pub name: Line,
    /// What it was given.
    pub value: Line,
}

impl Entry {
    /// A person's own command asked an application to do something, and this
    /// is what came of it.
    ///
    /// `ability` is the declared name of what was asked — `open_document`,
    /// `press`, `open` — so one thing has one name everywhere. `arguments` are
    /// what it was given, as the person typed them after validation.
    ///
    /// Additive; `format` stays `1`. `docs/contracts/record-file.md`'s *a new
    /// kind of `happened` is additive* is the decision.
    #[must_use]
    pub fn asked_an_application(
        application: &str,
        ability: &str,
        road: Road,
        arguments: &[(String, String)],
        answered: Answered,
        sent: SentDuring,
        at: SystemTime,
    ) -> Self {
        Self::new(
            at,
            Happened::AskedAnApplication {
                application: Line::of(application),
                ability: Line::of(ability),
                road: Some(road),
                arguments: arguments
                    .iter()
                    .map(|(name, value)| Given {
                        name: Line::of(name),
                        value: Line::of(value),
                    })
                    .collect(),
                answered,
                sent: Some(sent),
            },
        )
    }

    /// A person's own command was refused before it reached the application —
    /// a value that did not validate, an application not installed, a control
    /// that is not there.
    ///
    /// **No arguments and no road**: nothing was carried, and what never
    /// validated is whatever arrived. `why` is the sentence the person saw.
    #[must_use]
    pub fn an_application_was_not_asked(
        application: &str,
        ability: &str,
        why: &str,
        at: SystemTime,
    ) -> Self {
        Self::new(
            at,
            Happened::AskedAnApplication {
                application: Line::of(application),
                ability: Line::of(ability),
                road: None,
                arguments: Vec::new(),
                answered: Answered::Refused { said: Line::of(why) },
                sent: None,
            },
        )
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_762_000_000)
    }

    /// **What was done crosses the file whole**, with the road and the count,
    /// and no agent in it.
    #[test]
    fn an_act_is_written_with_its_road_its_arguments_and_what_was_sent() {
        let entry = Entry::asked_an_application(
            "org.gnome.TextEditor",
            "open_document",
            Road::Declared,
            &[("document".to_owned(), "/home/alo/Notes/today.txt".to_owned())],
            Answered::Done,
            SentDuring::Counted { bytes: 0 },
            noon(),
        );
        assert_eq!(entry.agent(), None, "a person's own act named an agent");
        let written = serde_json::to_string(entry.happened()).unwrap();
        assert!(written.contains("\"asked-an-application\""), "{written}");
        assert!(written.contains("\"road\":\"declared\""), "{written}");
        assert!(written.contains("\"document\""), "{written}");
        let back: Happened = serde_json::from_str(&written).unwrap();
        assert_eq!(&back, entry.happened());
    }

    /// **A refusal keeps the ability and why, and never the arguments.**
    #[test]
    fn a_refusal_keeps_why_and_not_what_arrived() {
        let entry = Entry::an_application_was_not_asked(
            "org.gnome.TextEditor",
            "press",
            "Text Editor shows no button named Save.",
            noon(),
        );
        match entry.happened() {
            Happened::AskedAnApplication {
                arguments,
                road,
                answered,
                sent,
                ..
            } => {
                assert!(arguments.is_empty());
                assert_eq!(*road, None);
                assert_eq!(sent, &None);
                assert!(matches!(answered, Answered::Refused { said } if said.is("Text Editor shows no button named Save.")));
            }
            other => panic!("not an application entry: {other:?}"),
        }
        assert!(entry.happened().was_stopped());
    }

    /// **What the application sent leaves a trace in law 1's query**, and an
    /// act during which nothing was sent does not.
    #[test]
    fn only_an_act_during_which_something_was_sent_counts_as_egress() {
        let quiet = Entry::asked_an_application(
            "org.mozilla.Thunderbird",
            "press",
            Road::Tree,
            &[],
            Answered::Done,
            SentDuring::Counted { bytes: 0 },
            noon(),
        );
        let sent = Entry::asked_an_application(
            "org.mozilla.Thunderbird",
            "press",
            Road::Tree,
            &[],
            Answered::Done,
            SentDuring::AtLeast { bytes: 51_200 },
            noon(),
        );
        assert!(!quiet.happened().caused_egress());
        assert!(sent.happened().caused_egress());
    }
}
