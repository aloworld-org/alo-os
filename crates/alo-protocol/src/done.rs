//! What the machine found or did, as it goes back to a client.
//!
//! `alo_files::Answer`'s six, with every path put through [`crate::naming`] and
//! everything that could not be shown counted — and, since 2026-10-13, an
//! installation, which carries no path and so goes through none of that. Each
//! is made from what a verb really answered with and from nothing else: there
//! is no constructor here that takes a path or an application name, so a daemon
//! cannot answer with something no verb produced.
//!
//! # Every bound still says it was reached
//!
//! `alo-files`' rule, carried across the socket: a listing that was cut short,
//! a search that stopped early and an archive that left links behind all say so
//! here too, because **a bounded answer that does not say it was bounded reads
//! exactly like a complete one**. This crate adds one more count of its own —
//! what could not be shown — and it goes in the same place, for the same
//! reason.
//!
//! # A path that cannot be shown is not a failure
//!
//! Three of the six answer with a single path: where a file was renamed to,
//! where it was moved to, where an archive was written. When that one path
//! cannot be shown, the answer is the change **with no path in it** rather than
//! an error, because the change happened — the file really was moved — and
//! telling a client it failed would be telling it something untrue about the
//! disk. [`crate::naming`] is where the rule and its reasons are.

use alo_files::Answer;
use serde::{Deserialize, Serialize};

use crate::naming::{all_shown, shown};
use alo_software::{Installed, Sandboxing};

use crate::thing::Thing;

/// What a verb this machine carried out answered with.
///
/// Six of the seven are the file verbs. The seventh arrived with the second
/// executor, and it is additive on purpose: a client built against the six
/// reads an answer it has no arm for rather than failing to parse the six it
/// knows, which is what *contracts change additively* means on a wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum Done {
    /// What is in a folder.
    Listed {
        /// What is in it, by name, in the order a person would read it.
        things: Vec<Thing>,
        /// How many things were left out because their names could not be
        /// shown.
        could_not_be_named: usize,
        /// Whether the folder holds more than one answer carries.
        cut_short: bool,
    },
    /// What is in a file, as text.
    ///
    /// Contents rather than a name, so nothing is filtered out of it: see
    /// `naming.rs` for why those are different questions.
    Read {
        /// The file's contents.
        text: String,
    },
    /// The files a search found.
    Found {
        /// Where each one is.
        files: Vec<String>,
        /// How many were left out because their paths could not be shown.
        could_not_be_named: usize,
        /// Whether the search stopped before it had looked everywhere.
        cut_short: bool,
    },
    /// The file was renamed.
    Renamed {
        /// Where it now is, and nothing when that path cannot be shown.
        now_at: Option<String>,
    },
    /// The file was moved.
    Moved {
        /// Where it now is, and nothing when that path cannot be shown.
        now_at: Option<String>,
    },
    /// An archive was made.
    Archived {
        /// Where the archive is, and nothing when that path cannot be shown.
        at: Option<String>,
        /// How many things went into it.
        things: usize,
        /// How many were left out because they were links rather than things.
        left_out: usize,
        /// How many bytes the archive holds.
        bytes: u64,
    },
    /// An application was installed.
    Installed {
        /// What it is called where it came from — the identifier, which is what
        /// a person asked for and what every other surface knows it by.
        application: String,
        /// Whether it is sandboxed.
        ///
        /// On the wire rather than derivable, for the reason
        /// `alo_software::Installed` keeps it: a client marking an unsandboxed
        /// application has the fact handed to it, and a client that forgot to
        /// ask would quietly show a person the wrong thing about their own
        /// machine.
        sandboxed: bool,
    },
    /// An application was opened, brought to the front, or asked to close.
    ///
    /// **`acted` means *this machine asked*, never *it is done*** —
    /// `docs/contracts/application-door.md` says so, and
    /// `docs/contracts/agent-verbs.md` already promised it about the third act:
    /// *`close_application` asks; nothing kills anything.* A client that read
    /// this as *the window is gone* would be reading more than the compositor
    /// said.
    Reached {
        /// Which act: `open`, `focus` or `close`, as the door's own wire spells
        /// them.
        act: String,
        /// The application it was asked of.
        application: String,
    },
}

impl Done {
    /// What an installation answered, as it goes on the wire.
    ///
    /// **One constructor per family rather than one taking
    /// `alo_turn::WhatAVerbDid`**, and that is a layering decision rather than
    /// a style: `alo-turn` is a *dev*-dependency of this crate on purpose — the
    /// wire format does not know what a turn is, and a `Done` that could only
    /// be made from one would make the orchestrator something every client
    /// library had to build against. Whoever holds a `WhatAVerbDid` matches it
    /// and calls one of these. There are two such places —
    /// `alo_agentd::what_a_client_is_told_a_verb_did` and `alo-corridor`'s
    /// door — and the compiler names both of them when a third family arrives,
    /// which is what a closed enum is for.
    ///
    /// The identifier is **not** put through this crate's `naming`, unlike every
    /// path in [`Done::of_a_file_verb`]: `org.gnome.TextEditor` is decided by
    /// whoever packaged the application and is already held to its own shape by
    /// `alo_applications::Application`, so there is nothing in it to filter.
    #[must_use]
    pub fn of_an_installation(installed: &Installed) -> Self {
        Self::Installed {
            application: installed.application().identifier().to_owned(),
            // Matched rather than compared, so a third kind of sandboxing
            // cannot quietly go on the wire as `false`.
            sandboxed: match installed.sandboxing() {
                Sandboxing::Sandboxed => true,
                Sandboxing::Unsandboxed => false,
            },
        }
    }

    /// What one of the three application acts did, as it goes on the wire.
    ///
    /// **Takes the two words rather than a turn's type**, which is this file's
    /// rule and the reason it has one constructor per family: `alo-protocol`
    /// holds `alo-turn` as a dev-dependency, so whoever knows what a turn is
    /// does the choosing. Here that means `alo_corridor` and `alo_agentd` pass
    /// `reached.acting().on_the_wire()` and the application's identifier.
    ///
    /// Neither is put through this crate's `naming`, for
    /// [`Done::of_an_installation`]'s reason: the act is one of three literals
    /// this machine wrote, and the identifier is already held to its own shape
    /// by `alo_applications::Application`.
    #[must_use]
    pub fn of_an_application_act(act: &str, application: &str) -> Self {
        Self::Reached {
            act: act.to_owned(),
            application: application.to_owned(),
        }
    }

    /// What one of the six file verbs answered, as it goes on the wire.
    #[must_use]
    pub fn of_a_file_verb(answer: &Answer) -> Self {
        match answer {
            Answer::Listed(listing) => Self::Listed {
                things: listing.things().iter().map(Thing::of).collect(),
                could_not_be_named: listing.could_not_be_named(),
                cut_short: listing.was_cut_short(),
            },
            Answer::Read(text) => Self::Read { text: text.clone() },
            Answer::Found(search) => {
                let (files, could_not_be_named) = all_shown(search.files());
                Self::Found {
                    files,
                    could_not_be_named,
                    cut_short: search.was_cut_short(),
                }
            }
            Answer::Renamed(at) => Self::Renamed { now_at: shown(at) },
            Answer::Moved(at) => Self::Moved { now_at: shown(at) },
            Answer::Archived(archived) => Self::Archived {
                at: shown(archived.at()),
                things: archived.things(),
                left_out: archived.left_out(),
                bytes: archived.bytes(),
            },
        }
    }

    /// Whether this answer left something out.
    ///
    /// True when a bound was reached, when a name could not be shown, or when
    /// an archive left a link where it was. What a shell puts a caveat beside,
    /// asked once rather than matched for in six places — and a client that
    /// never asks still has every count in front of it.
    #[must_use]
    pub fn left_something_out(&self) -> bool {
        match self {
            Self::Listed {
                could_not_be_named,
                cut_short,
                ..
            }
            | Self::Found {
                could_not_be_named,
                cut_short,
                ..
            } => *could_not_be_named > 0 || *cut_short,
            Self::Read { .. } => false,
            Self::Renamed { now_at } | Self::Moved { now_at } => now_at.is_none(),
            Self::Archived { at, left_out, .. } => at.is_none() || *left_out > 0,
            // **An installation leaves nothing out.** It names one application
            // and says whether it is sandboxed, and both are always there —
            // there is no bound to reach, no path that might not be showable,
            // and nothing a shell should put a caveat beside. `false` is the
            // honest answer rather than the default one.
            Self::Installed { .. } => false,
            // **An application act leaves nothing out either, and the
            // temptation here is to say otherwise.** *This machine asked* is
            // not the same as *it is done* — an application may be putting up
            // its own *save your changes?* — so a caveat looks right.
            //
            // It is not, and the reason is where that caveat already lives:
            // the approval sentence a person read says **ask** Geary to close
            // (`docs/contracts/agent-verbs.md` puts the word in the sentence
            // rather than only in documentation, deliberately). A flag here
            // would put a second caveat beside every `open` and every `focus`
            // as well, which omit nothing at all — and a caveat on everything
            // is a caveat on nothing.
            Self::Reached { .. } => false,
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
    use alo_files::{Listing, Search};
    use std::path::PathBuf;

    // A `Listing`, a `Search` and an `Archived` are made by looking at a real
    // folder and have no public constructor, which is `alo-files`' guarantee
    // and not one to weaken for a test. So the conversions that need a full one
    // are exercised against a real filesystem in
    // `tests/what_the_daemon_answers_with.rs`, and what is here is everything
    // that can be built without one — including the whole of the rule about a
    // path that cannot be shown, which `crate::naming` owns and tests.

    /// A read crosses as it was read, control characters and all: a file with
    /// a line break in it is an ordinary file, and JSON is what keeps the
    /// message one line.
    #[test]
    fn what_is_in_a_file_crosses_as_it_is_written() {
        let contents = "Dear Anna,\n\n\tthe invoice is attached.\u{1b}[0m\n";
        let done = Done::of_a_file_verb(&Answer::Read(contents.to_owned()));
        let written = serde_json::to_string(&done).unwrap();
        assert!(!written.contains('\n'), "{written}");
        let back: Done = serde_json::from_str(&written).unwrap();
        assert_eq!(
            back,
            Done::Read {
                text: contents.to_owned()
            }
        );
        assert!(!back.left_something_out());
    }

    /// A search that found nothing crosses as a search that found nothing,
    /// rather than as an answer with nothing in it that a client has to guess
    /// at.
    #[test]
    fn a_search_that_found_nothing_says_so() {
        let done = Done::of_a_file_verb(&Answer::Found(Search::default()));
        assert_eq!(
            done,
            Done::Found {
                files: Vec::new(),
                could_not_be_named: 0,
                cut_short: false,
            }
        );
        assert!(!done.left_something_out());
    }

    /// **A change whose path cannot be shown is still the change.** The file
    /// was moved; what is missing is a way to spell where to, and an answer
    /// that reported a failure would be an answer that is untrue about the
    /// disk.
    #[test]
    fn a_change_with_an_unshowable_path_is_the_change_without_the_path() {
        let moved = Done::of_a_file_verb(&Answer::Moved(PathBuf::from("/home/anna/a\u{1b}.pdf")));
        assert_eq!(moved, Done::Moved { now_at: None });
        assert!(moved.left_something_out());

        let renamed = Done::of_a_file_verb(&Answer::Renamed(PathBuf::from("/home/anna/b.pdf")));
        assert_eq!(
            renamed,
            Done::Renamed {
                now_at: Some("/home/anna/b.pdf".to_owned())
            }
        );
        assert!(!renamed.left_something_out());
    }

    /// An archive that left a link where it was says so, and so does one whose
    /// own path cannot be shown — the two counts a client puts a caveat beside.
    #[test]
    fn an_archive_says_what_it_left_out() {
        let complete = Done::Archived {
            at: Some("/home/anna/Archive/2026.zip".to_owned()),
            things: 12,
            left_out: 0,
            bytes: 4096,
        };
        assert!(!complete.left_something_out());

        for archived in [
            Done::Archived {
                at: Some("/home/anna/Archive/2026.zip".to_owned()),
                things: 12,
                left_out: 1,
                bytes: 4096,
            },
            Done::Archived {
                at: None,
                things: 12,
                left_out: 0,
                bytes: 4096,
            },
        ] {
            assert!(archived.left_something_out(), "{archived:?}");
        }
    }

    /// A listing carries the count the disk made, and this crate adds nothing
    /// to it: a `Named` cannot hold a name that could not be shown, because it
    /// was refused where the folder was read.
    #[test]
    fn a_listing_carries_the_count_the_disk_made_and_makes_no_second_one() {
        let complete = Done::of_a_file_verb(&Answer::Listed(Listing::default()));
        assert_eq!(
            complete,
            Done::Listed {
                things: Vec::new(),
                could_not_be_named: 0,
                cut_short: false,
            }
        );
        assert!(!complete.left_something_out());

        let bounded = Done::Listed {
            things: Vec::new(),
            could_not_be_named: 3,
            cut_short: true,
        };
        assert!(bounded.left_something_out());
    }

    /// Every one of the six reads back as what was written, so a shell and a
    /// daemon built from this crate cannot disagree about what happened.
    #[test]
    fn every_answer_reads_back_as_what_was_written() {
        for done in [
            Done::of_a_file_verb(&Answer::Listed(Listing::default())),
            Done::of_a_file_verb(&Answer::Read("hello".to_owned())),
            Done::of_a_file_verb(&Answer::Found(Search::default())),
            Done::of_a_file_verb(&Answer::Renamed(PathBuf::from("/home/anna/a.pdf"))),
            Done::of_a_file_verb(&Answer::Moved(PathBuf::from("/home/anna/b.pdf"))),
            Done::Archived {
                at: Some("/home/anna/c.zip".to_owned()),
                things: 1,
                left_out: 0,
                bytes: 2,
            },
            Done::Installed {
                application: "org.gnome.TextEditor".to_owned(),
                sandboxed: true,
            },
        ] {
            let written = serde_json::to_string(&done).unwrap();
            let back: Done = serde_json::from_str(&written).unwrap();
            assert_eq!(back, done);
        }
    }

    /// **The shape an installation goes on the wire in**, asserted literally.
    ///
    /// `docs/contracts/daemon-protocol.md` prints this exact line, so a renamed
    /// field is a broken client rather than a failing test — and the round-trip
    /// above would not notice, because it writes and reads with the same code.
    ///
    /// Built here rather than from an `alo_software::Installed`, which has no
    /// public constructor **on purpose**: one of those is evidence that an
    /// installation really happened, and anything able to make one could claim
    /// it did. The mapping from that evidence to this shape is asserted in
    /// `tests/an_installation_as_a_client_sees_it.rs`, which gets its
    /// `Installed` the only way there is — by installing something.
    #[test]
    fn an_installation_is_the_identifier_and_whether_it_is_sandboxed() {
        assert_eq!(
            serde_json::to_string(&Done::Installed {
                application: "org.gnome.TextEditor".to_owned(),
                sandboxed: true,
            })
            .unwrap(),
            r#"{"installed":{"application":"org.gnome.TextEditor","sandboxed":true}}"#
        );
        assert_eq!(
            serde_json::to_string(&Done::Installed {
                application: "org.gnome.TextEditor".to_owned(),
                sandboxed: false,
            })
            .unwrap(),
            r#"{"installed":{"application":"org.gnome.TextEditor","sandboxed":false}}"#
        );
    }
}
