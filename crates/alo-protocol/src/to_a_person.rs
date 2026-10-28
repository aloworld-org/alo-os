//! What the daemon says back to the person's shell.
//!
//! Fourteen answers to fourteen requests: what a change did once they approved it,
//! that a change they declined is written down, everything still waiting for
//! them, how much is granted after the machine read its list again, the four
//! about pairing — the proposal waiting with its code, what became of a
//! confirmation, what became of a revocation, and everything paired and
//! waiting — the machine now chosen to answer their questions, a machine named
//! or its name taken away, the workspaces on the network and the one they
//! opened, what their own machine says about itself on the network, and, for
//! any of them, the refusal in the language they read.
//!
//! Wherever a paired machine is named here — on the list, chosen to answer,
//! named — its identity comes with the name the person gave it beside it,
//! `called`, absent until they give one.
//!
//! The four about pairing are on this side and no other, for the reason
//! `waiting` is: a pairing is the person's list of which machines may ask this
//! one (ADR 0003), and an agent told what is paired — or handed a code to
//! confirm — would be an agent reading the person's own list.
//!
//! # A person is never handed a model's answer here
//!
//! It costs nothing to leave out and it keeps one thing straight: a question
//! belongs to the turn that asked it, and the answer comes back on the
//! connection the question went out on. A shell that could be handed one would
//! be a second place an answer can appear, and *where the answer came from is
//! said where the answer appears* would then be a promise made in two places.
//!
//! # What is waiting is a read, and it is the person's
//!
//! [`ToAPerson::waiting`] is the answer to the one request item 21b added:
//! `waiting`, on the person's door. It is what a shell draws — the number and
//! the sentence for each change — and it is on this side because ADR 0001 §5
//! puts the answering there. An agent asking for it is
//! `NotUnderstood::NotForAnAgent`, in the same words as an agent trying to
//! approve something.

use std::time::SystemTime;

use alo_capability::Waiting;
use alo_strings::{Said, Strings};

use crate::advertised::Advertised;
use crate::done::Done;
use crate::frame;
use crate::pairing::{AfterConfirming, AfterNaming, AfterRevoking, Paired, WaitingToPair};
use crate::refusing::NotUnderstood;
use crate::standing::Standing;
use crate::told::Told;
use crate::wording::Wording;
use crate::workspaces::FoundWorkspace;

/// One thing the daemon told a person's shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToAPerson {
    /// What the change they approved did.
    Did(Done),
    /// Everything they have been asked and have not answered.
    Waiting {
        /// In the order they were proposed, which is the order they are read
        /// in.
        changes: Vec<Standing>,
    },
    /// A place to return to was kept before the person's next command, or was
    /// not and this is why.
    ///
    /// [ADR 0130](../../../docs/decisions/0130-what-takes-the-bracket-before-a-persons-own-command.md)
    /// option B, accepted 2026-10-22.
    ///
    /// **`why_not` is [`None`] when one was kept, and that is deliberate.** A
    /// sentence before every command a person runs is noise they would learn to
    /// scroll past, and the thing they need to hear is the one that does not
    /// happen. ADR 0045's third term already points this way for the agent's
    /// own turns — *a machine too full to snapshot still runs the turn* — so
    /// the command runs either way and the only question is whether they were
    /// told.
    APlaceToReturnTo {
        /// Whether a place was kept.
        kept: bool,
        /// What to tell them when one was not, in their own language, worded by
        /// the service so the screen and the record cannot become two accounts
        /// of one moment.
        why_not: Option<Wording>,
    },
    /// The change they declined is written down, and nothing ran.
    Declined,
    /// What is granted was read again, and this is how many grants that came to.
    ///
    /// The answer to `granted` on this door. A count and no list — what is
    /// granted is drawn by the surface that lists it, out of the person's own
    /// file, and a list here would be a second copy of it crossing a socket for
    /// nobody to check against.
    Granted {
        /// How many grants are in force after the reading.
        holding: u64,
    },
    /// It did not happen, and this is what to say about it.
    Refused(Wording),
    /// The pairing they proposed is waiting, with the code to show them.
    Pairing(WaitingToPair),
    /// What became of the confirmation they gave.
    Confirmed {
        /// Waiting for the other person, paired, or paired until a restart.
        became: AfterConfirming,
    },
    /// What became of the revocation they made.
    Revoked {
        /// Revoked, or revoked until a restart.
        became: AfterRevoking,
    },
    /// Everything this machine is paired with, and every proposal waiting.
    Pairings {
        /// In the order they were made.
        paired: Vec<Paired>,
        /// In the order they began waiting.
        waiting: Vec<WaitingToPair>,
    },
    /// The machine they chose now answers their questions, by the identity
    /// written down in their settings.
    ChosenToAnswer {
        /// The machine chosen, by its identity.
        machine: String,
        /// What they called it, if they gave it a name.
        called: Option<String>,
    },
    /// A machine this one is paired with was named, or its name taken away.
    MachineNamed {
        /// The machine, by its identity.
        machine: String,
        /// What it is now called, if anything.
        called: Option<String>,
        /// Kept, or kept until a restart.
        became: AfterNaming,
    },
    /// The workspaces discovery found on the local network at the moment.
    Workspaces {
        /// In the order they answered; empty when nothing was found.
        found: Vec<FoundWorkspace>,
    },
    /// The workspace they opened, at the one address it answered from when
    /// the daemon looked — for their session to hand to the workspace client.
    WorkspaceOpened(FoundWorkspace),
    /// What their machine says about itself on the local network: its
    /// identity, the port its presence names, and the workspace it hosts, that
    /// it hosts none, or why one installed is not advertised.
    Advertised(Advertised),
    /// A model answered the question they asked themselves.
    ///
    /// The same three fields `ToAnAgent`'s answer carries, and on the wire the
    /// same `answered` message — because it is the same thing arriving at a
    /// different door, and a second shape for it would be two things to keep
    /// in step. What differs is which door may send it, and that is the
    /// daemon's to decide from `SO_PEERCRED`.
    ///
    /// `came_from` is **rendered by the daemon**, in the language the person
    /// reads, for the reason every refusal on this door is: the screen and the
    /// record cannot become two accounts of one moment, and a client that
    /// worded it would be a second place the sentence is chosen.
    Answered {
        /// What the model said, in its own words.
        text: String,
        /// Where the answer came from, in the language the person reads.
        came_from: Wording,
        /// Which model was asked, as it was named when the question was put.
        model: String,
    },
    /// What this service's indicator shows now, oldest first (ADR 0134): the
    /// answer to [`FromAPerson::WatchWhatLeaves`](crate::FromAPerson), and every
    /// message after it on that connection. **The whole list every time.**
    Leaving {
        /// One line per thing leaving.
        lines: Vec<alo_egress::Reported>,
    },
}

impl ToAPerson {
    /// What this machine says about itself on the local network.
    #[must_use]
    pub const fn advertised(advertised: Advertised) -> Self {
        Self::Advertised(advertised)
    }

    /// What this machine says about itself, when that is what they asked.
    #[must_use]
    pub const fn advertisement(&self) -> Option<&Advertised> {
        match self {
            Self::Advertised(advertised) => Some(advertised),
            _ => None,
        }
    }

    /// The workspace they opened, at the address measured at that moment.
    #[must_use]
    pub const fn workspace_opened(workspace: FoundWorkspace) -> Self {
        Self::WorkspaceOpened(workspace)
    }

    /// The workspace they opened, when that is what they asked.
    #[must_use]
    pub const fn opened_workspace(&self) -> Option<&FoundWorkspace> {
        match self {
            Self::WorkspaceOpened(workspace) => Some(workspace),
            _ => None,
        }
    }

    /// The workspaces found on the local network, in the order they answered.
    #[must_use]
    pub const fn workspaces(found: Vec<FoundWorkspace>) -> Self {
        Self::Workspaces { found }
    }

    /// The workspaces found, when that is what they asked.
    #[must_use]
    pub fn workspaces_found(&self) -> Option<&[FoundWorkspace]> {
        match self {
            Self::Workspaces { found } => Some(found),
            _ => None,
        }
    }

    /// The pairing they proposed, waiting with the code known.
    #[must_use]
    pub const fn pairing(waiting: WaitingToPair) -> Self {
        Self::Pairing(waiting)
    }

    /// What became of their confirmation.
    #[must_use]
    pub const fn confirmed(became: AfterConfirming) -> Self {
        Self::Confirmed { became }
    }

    /// What became of their revocation.
    #[must_use]
    pub const fn revoked(became: AfterRevoking) -> Self {
        Self::Revoked { became }
    }

    /// Everything paired and everything waiting.
    #[must_use]
    pub const fn pairings(paired: Vec<Paired>, waiting: Vec<WaitingToPair>) -> Self {
        Self::Pairings { paired, waiting }
    }

    /// The machine they chose to answer their questions, by its identity, with
    /// the name they gave it if they gave one.
    #[must_use]
    pub fn chosen_to_answer(machine: &str, called: Option<&str>) -> Self {
        Self::ChosenToAnswer {
            machine: machine.to_owned(),
            called: called.map(ToOwned::to_owned),
        }
    }

    /// The machine now answering their questions, when that is what they were
    /// told.
    #[must_use]
    pub fn machine_chosen_to_answer(&self) -> Option<&str> {
        match self {
            Self::ChosenToAnswer { machine, .. } => Some(machine),
            _ => None,
        }
    }

    /// A machine named, or its name taken away when `called` is nothing.
    #[must_use]
    pub fn machine_named(machine: &str, called: Option<&str>, became: AfterNaming) -> Self {
        Self::MachineNamed {
            machine: machine.to_owned(),
            called: called.map(ToOwned::to_owned),
            became,
        }
    }

    /// What the machine they chose, or the machine they named, is called now —
    /// when that is what they were told and it has a name.
    #[must_use]
    pub fn called(&self) -> Option<&str> {
        match self {
            Self::ChosenToAnswer { called, .. } | Self::MachineNamed { called, .. } => {
                called.as_deref()
            }
            _ => None,
        }
    }

    /// What became of a name they gave or took away, and which machine it was
    /// — when that is what they were told.
    #[must_use]
    pub fn became_of_naming(&self) -> Option<(&str, AfterNaming)> {
        match self {
            Self::MachineNamed {
                machine, became, ..
            } => Some((machine, *became)),
            _ => None,
        }
    }

    /// The proposal waiting, when that is what they were told.
    ///
    /// The one answered to `pair`; what `pairings` lists is
    /// [`ToAPerson::waiting_to_pair`].
    #[must_use]
    pub const fn proposed_pairing(&self) -> Option<&WaitingToPair> {
        match self {
            Self::Pairing(waiting) => Some(waiting),
            _ => None,
        }
    }

    /// What became of their confirmation, when that is what they were told.
    #[must_use]
    pub const fn became_of_confirming(&self) -> Option<AfterConfirming> {
        match self {
            Self::Confirmed { became } => Some(*became),
            _ => None,
        }
    }

    /// What became of their revocation, when that is what they were told.
    #[must_use]
    pub const fn became_of_revoking(&self) -> Option<AfterRevoking> {
        match self {
            Self::Revoked { became } => Some(*became),
            _ => None,
        }
    }

    /// What is paired, when that is what they asked.
    #[must_use]
    pub fn paired(&self) -> Option<&[Paired]> {
        match self {
            Self::Pairings { paired, .. } => Some(paired),
            _ => None,
        }
    }

    /// What is waiting to be paired, when that is what they asked.
    #[must_use]
    pub fn waiting_to_pair(&self) -> Option<&[WaitingToPair]> {
        match self {
            Self::Pairings { waiting, .. } => Some(waiting),
            _ => None,
        }
    }

    /// What the change they approved did.
    ///
    /// Takes the [`Done`] rather than what a verb answered with, because which
    /// family answered is a question `alo-turn` can ask and this crate cannot:
    /// it holds `alo-turn` as a *dev*-dependency so that a client library never
    /// has to build the orchestrator. [`Done::of_a_file_verb`] and
    /// [`Done::of_an_installation`] are what a caller picks between.
    #[must_use]
    pub fn did(done: Done) -> Self {
        Self::Did(done)
    }

    /// Everything waiting for them, in the order it was proposed.
    ///
    /// Takes what `alo_capability::Approvals::waiting_at` and
    /// `alo_turn::Turning::waiting_at` hand out, so the list a shell draws is
    /// the list the turn holds rather than one assembled beside it — and every
    /// change in it carries its own sentence.
    #[must_use]
    pub fn waiting<'a>(
        changes: impl Iterator<Item = &'a Waiting>,
        strings: &Strings,
        now: SystemTime,
    ) -> Self {
        Self::Waiting {
            changes: changes
                .map(|waiting| Standing::of(waiting, strings, now))
                .collect(),
        }
    }

    /// Everything waiting for them that a paired machine proposed, in the
    /// order it was proposed, each change saying which machine.
    ///
    /// [`ToAPerson::waiting`] with every change stamped
    /// [`Standing::from_a_machine`]: what `alo_turn::Arriving::waiting_at`
    /// hands out is one machine's changes, and the person answering them is
    /// owed the name (ADR 0003: a change waits for the **receiving** machine's
    /// person, and what they approve is the sentence).
    #[must_use]
    pub fn waiting_from_a_machine<'a>(
        changes: impl Iterator<Item = &'a Waiting>,
        machine: &str,
        strings: &Strings,
        now: SystemTime,
    ) -> Self {
        Self::Waiting {
            changes: changes
                .map(|waiting| Standing::of(waiting, strings, now).from_a_machine(machine))
                .collect(),
        }
    }

    /// What is granted, read again, and how many grants that came to.
    #[must_use]
    pub const fn granted(holding: u64) -> Self {
        Self::Granted { holding }
    }

    /// A model answered the question the person asked themselves.
    ///
    /// `came_from` arrives **already rendered**, for
    /// [`crate::ToAnAgent::answered`]'s reason and the same one this door gives
    /// for every refusal: this crate does not depend on `alo-asking`, which
    /// carries an HTTP client and a TLS stack, so the crate reading the
    /// untrusted side of a socket stays small enough to audit.
    #[must_use]
    pub fn answered(text: &str, came_from: &Said, model: &str) -> Self {
        Self::Answered {
            text: text.to_owned(),
            came_from: Wording::of(came_from),
            model: model.to_owned(),
        }
    }

    /// The answer, when that is what came back.
    #[must_use]
    pub fn an_answer(&self) -> Option<(&str, &Wording, &str)> {
        match self {
            Self::Answered {
                text,
                came_from,
                model,
            } => Some((text, came_from, model)),
            _ => None,
        }
    }

    /// What an indicator shows, as the desktop is told it.
    #[must_use]
    pub fn leaving(showing: &[alo_egress::Shown]) -> Self {
        Self::Leaving {
            lines: showing.iter().map(alo_egress::Reported::of).collect(),
        }
    }

    /// The lines told, when this is what leaves.
    #[must_use]
    pub fn lines_leaving(&self) -> Option<&[alo_egress::Reported]> {
        match self {
            Self::Leaving { lines } => Some(lines),
            _ => None,
        }
    }

    /// It did not happen, in the words of whoever refused it.
    #[must_use]
    pub fn refused(said: &Said) -> Self {
        Self::Refused(Wording::of(said))
    }

    /// Read one line as something the daemon told a person's shell.
    ///
    /// # Errors
    /// [`NotUnderstood`] — the envelope's five refusals, and
    /// [`NotUnderstood::NotAnAnswerForAPerson`] for a well-formed answer that
    /// is only ever given to an agent.
    pub fn read(line: &str) -> Result<Self, NotUnderstood> {
        match frame::reply(line)? {
            Told::Did(done) => Ok(Self::Did(done)),
            Told::Waiting { changes } => Ok(Self::Waiting { changes }),
            Told::APlaceToReturnTo { kept, why_not } => {
                Ok(Self::APlaceToReturnTo { kept, why_not })
            }
            Told::Declined {} => Ok(Self::Declined),
            Told::Granted { holding } => Ok(Self::Granted { holding }),
            Told::Refused(wording) => Ok(Self::Refused(wording)),
            Told::Pairing(waiting) => Ok(Self::Pairing(waiting)),
            Told::Confirmed { became } => Ok(Self::Confirmed { became }),
            Told::Revoked { became } => Ok(Self::Revoked { became }),
            Told::Pairings { paired, waiting } => Ok(Self::Pairings { paired, waiting }),
            Told::ChosenToAnswer { machine, called } => {
                Ok(Self::ChosenToAnswer { machine, called })
            }
            Told::MachineNamed {
                machine,
                called,
                became,
            } => Ok(Self::MachineNamed {
                machine,
                called,
                became,
            }),
            Told::Workspaces { found } => Ok(Self::Workspaces { found }),
            Told::WorkspaceOpened(workspace) => Ok(Self::WorkspaceOpened(workspace)),
            Told::Advertised(advertised) => Ok(Self::Advertised(advertised)),
            Told::Answered {
                text,
                came_from,
                model,
            } => Ok(Self::Answered {
                text,
                came_from,
                model,
            }),
            Told::Leaving { lines } => Ok(Self::Leaving { lines }),
            Told::Proposed(_) => Err(NotUnderstood::NotAnAnswerForAPerson),
        }
    }

    /// This answer as the line that carries it.
    ///
    /// # Errors
    /// A `serde_json::Error`. See `frame.rs` for why it is handed back rather
    /// than swallowed, and for the bound a client holds this to when it reads
    /// one.
    pub fn written(&self) -> Result<String, serde_json::Error> {
        frame::spoken(self.clone().into())
    }

    /// What the machine did, when it did something.
    #[must_use]
    pub fn done(&self) -> Option<&Done> {
        match self {
            Self::Did(done) => Some(done),
            _ => None,
        }
    }

    /// How many grants are in force, when the machine has just read them again.
    #[must_use]
    pub const fn holding(&self) -> Option<u64> {
        match self {
            Self::Granted { holding } => Some(*holding),
            _ => None,
        }
    }

    /// What is waiting for them, when that is what they asked.
    #[must_use]
    pub fn changes(&self) -> Option<&[Standing]> {
        match self {
            Self::Waiting { changes } => Some(changes),
            _ => None,
        }
    }

    /// What to say to the person, when nothing was done.
    #[must_use]
    pub fn refusal(&self) -> Option<&Wording> {
        match self {
            Self::Refused(wording) => Some(wording),
            _ => None,
        }
    }
}

impl From<ToAPerson> for Told {
    fn from(told: ToAPerson) -> Self {
        match told {
            ToAPerson::Did(done) => Self::Did(done),
            ToAPerson::Waiting { changes } => Self::Waiting { changes },
            ToAPerson::APlaceToReturnTo { kept, why_not } => {
                Self::APlaceToReturnTo { kept, why_not }
            }
            ToAPerson::Declined => Self::Declined {},
            ToAPerson::Granted { holding } => Self::Granted { holding },
            ToAPerson::Refused(wording) => Self::Refused(wording),
            ToAPerson::Pairing(waiting) => Self::Pairing(waiting),
            ToAPerson::Confirmed { became } => Self::Confirmed { became },
            ToAPerson::Revoked { became } => Self::Revoked { became },
            ToAPerson::Pairings { paired, waiting } => Self::Pairings { paired, waiting },
            ToAPerson::ChosenToAnswer { machine, called } => {
                Self::ChosenToAnswer { machine, called }
            }
            ToAPerson::MachineNamed {
                machine,
                called,
                became,
            } => Self::MachineNamed {
                machine,
                called,
                became,
            },
            ToAPerson::Workspaces { found } => Self::Workspaces { found },
            ToAPerson::WorkspaceOpened(workspace) => Self::WorkspaceOpened(workspace),
            ToAPerson::Advertised(advertised) => Self::Advertised(advertised),
            ToAPerson::Answered {
                text,
                came_from,
                model,
            } => Self::Answered {
                text,
                came_from,
                model,
            },
            ToAPerson::Leaving { lines } => Self::Leaving { lines },
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
    use crate::ToAnAgent;
    use crate::testing::{a_change_waiting, an_hour_in, in_english, the_change, the_moment};
    use crate::words;
    use alo_files::Answer;
    use alo_strings::Filling;

    /// One sentence this crate really declares.
    fn a_sentence() -> Said {
        in_english().say(&words::NOT_READABLE.key(), &Filling::nothing())
    }

    /// The five, written and read back.
    #[test]
    fn the_five_a_person_is_told_read_back() {
        let (approvals, strings) = a_change_waiting();
        for told in [
            ToAPerson::did(Done::of_a_file_verb(&Answer::Renamed(
                "/home/anna/Invoices/march-final.pdf".into(),
            ))),
            ToAPerson::waiting(approvals.waiting_at(the_moment()), &strings, the_moment()),
            ToAPerson::Declined,
            ToAPerson::granted(2),
            ToAPerson::refused(&a_sentence()),
        ] {
            let written = told.written().unwrap();
            assert_eq!(ToAPerson::read(&written).unwrap(), told, "{written}");
        }
    }

    /// **The machine chosen to answer comes back to the person, by its
    /// identity, and never to an agent.**
    #[test]
    fn the_machine_chosen_to_answer_is_told_to_the_person_and_not_to_an_agent() {
        let told = ToAPerson::chosen_to_answer("0f1e2d3c4b5a69788796a5b4c3d2e1f0", None);
        assert_eq!(
            told.machine_chosen_to_answer(),
            Some("0f1e2d3c4b5a69788796a5b4c3d2e1f0")
        );
        let written = told.written().unwrap();
        assert!(
            written
                .contains(r#""chosen-to-answer":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0"}"#),
            "{written}"
        );
        assert_eq!(ToAPerson::read(&written).unwrap(), told);
        assert_eq!(
            ToAnAgent::read(&written),
            Err(NotUnderstood::NotAnAnswerForAnAgent)
        );
        assert!(ToAPerson::Declined.machine_chosen_to_answer().is_none());

        // With a name, the name comes beside the identity.
        let called = ToAPerson::chosen_to_answer(
            "0f1e2d3c4b5a69788796a5b4c3d2e1f0",
            Some("the studio machine"),
        );
        let written = called.written().unwrap();
        assert!(
            written.contains(
                r#""machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","called":"the studio machine""#
            ),
            "{written}"
        );
        assert_eq!(
            ToAPerson::read(&written).unwrap().called(),
            Some("the studio machine")
        );
    }

    /// **A machine named, and a name taken away, come back to the person with
    /// the identity and what became of it — and never to an agent.**
    #[test]
    fn a_machine_named_is_told_to_the_person_and_not_to_an_agent() {
        let named = ToAPerson::machine_named(
            "0f1e2d3c4b5a69788796a5b4c3d2e1f0",
            Some("the studio machine"),
            AfterNaming::Kept,
        );
        let written = named.written().unwrap();
        assert!(
            written.contains(r#""machine-named":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","called":"the studio machine","became":"kept"}"#),
            "{written}"
        );
        let back = ToAPerson::read(&written).unwrap();
        assert_eq!(back, named);
        assert_eq!(back.called(), Some("the studio machine"));
        assert_eq!(
            back.became_of_naming(),
            Some(("0f1e2d3c4b5a69788796a5b4c3d2e1f0", AfterNaming::Kept))
        );
        assert_eq!(
            ToAnAgent::read(&written),
            Err(NotUnderstood::NotAnAnswerForAnAgent)
        );

        let cleared = ToAPerson::machine_named(
            "0f1e2d3c4b5a69788796a5b4c3d2e1f0",
            None,
            AfterNaming::KeptUntilARestart,
        );
        let written = cleared.written().unwrap();
        assert!(!written.contains("called"), "{written}");
        assert_eq!(ToAPerson::read(&written).unwrap(), cleared);
        assert!(ToAPerson::Declined.became_of_naming().is_none());
    }

    /// **The workspaces found come back to the person — never to an agent —
    /// each with where discovery measured it answers**, and nothing found is a
    /// list with nothing on it rather than an absence.
    #[test]
    fn the_workspaces_found_are_told_to_the_person_and_not_to_an_agent() {
        let found = vec![
            FoundWorkspace::of(
                "0f1e2d3c4b5a69788796a5b4c3d2e1f0",
                "192.168.1.20:8443".parse().unwrap(),
                "1",
            )
            .hosted_by(Some("the studio machine")),
        ];
        let told = ToAPerson::workspaces(found.clone());
        assert_eq!(told.workspaces_found(), Some(found.as_slice()));
        let written = told.written().unwrap();
        assert!(
            written.contains(r#""workspaces":{"found":[{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","answers_at":"192.168.1.20:8443","speaks":"1","called":"the studio machine"}]}"#),
            "{written}"
        );
        assert_eq!(ToAPerson::read(&written).unwrap(), told);
        assert_eq!(
            ToAnAgent::read(&written),
            Err(NotUnderstood::NotAnAnswerForAnAgent)
        );

        let nothing = ToAPerson::workspaces(Vec::new()).written().unwrap();
        assert_eq!(
            nothing,
            r#"{"format":1,"tells":{"workspaces":{"found":[]}}}"#
        );
        assert_eq!(
            ToAPerson::read(&nothing).unwrap().workspaces_found(),
            Some([].as_slice())
        );
        assert!(ToAPerson::Declined.workspaces_found().is_none());
    }

    /// **The workspace opened comes back to the person — never to an agent —
    /// with the one address measured**, in the shape a listed workspace has.
    #[test]
    fn the_workspace_opened_is_told_to_the_person_and_not_to_an_agent() {
        let opened = FoundWorkspace::of(
            "0f1e2d3c4b5a69788796a5b4c3d2e1f0",
            "192.168.1.20:8443".parse().unwrap(),
            "1",
        );
        let told = ToAPerson::workspace_opened(opened.clone());
        assert_eq!(told.opened_workspace(), Some(&opened));
        let written = told.written().unwrap();
        assert_eq!(
            written,
            r#"{"format":1,"tells":{"workspace-opened":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","answers_at":"192.168.1.20:8443","speaks":"1"}}}"#
        );
        assert_eq!(ToAPerson::read(&written).unwrap(), told);
        assert_eq!(
            ToAnAgent::read(&written),
            Err(NotUnderstood::NotAnAnswerForAnAgent)
        );
        assert!(ToAPerson::Declined.opened_workspace().is_none());
    }

    /// **What this machine advertises comes back to the person — never to an
    /// agent — as the identity, the presence's port and the workspace**, and
    /// nothing else.
    #[test]
    fn what_this_machine_advertises_is_told_to_the_person_and_not_to_an_agent() {
        let advertised = Advertised::of(
            "0f1e2d3c4b5a69788796a5b4c3d2e1f0",
            7_610,
            crate::HostedWorkspace::hosts(std::num::NonZeroU16::new(8_443).unwrap()),
        );
        let told = ToAPerson::advertised(advertised.clone());
        assert_eq!(told.advertisement(), Some(&advertised));
        let written = told.written().unwrap();
        assert_eq!(
            written,
            r#"{"format":1,"tells":{"advertised":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","port":7610,"workspace":{"hosts":{"port":8443}}}}}"#
        );
        assert_eq!(ToAPerson::read(&written).unwrap(), told);
        assert_eq!(
            ToAnAgent::read(&written),
            Err(NotUnderstood::NotAnAnswerForAnAgent)
        );
        assert!(ToAPerson::Declined.advertisement().is_none());
    }

    /// **What a shell draws is the number and the sentence**, one for each
    /// change the person has not answered — which is the request item 21b owed
    /// the person's side, answered in the shape it is drawn in.
    #[test]
    fn what_is_waiting_carries_a_number_and_a_sentence_for_each_change() {
        let (approvals, strings) = a_change_waiting();
        let told = ToAPerson::waiting(approvals.waiting_at(the_moment()), &strings, the_moment());
        let changes = told.changes().unwrap();
        assert_eq!(changes.len(), 1);
        let only = changes.first().unwrap();
        assert_eq!(only.number(), the_change(&approvals).id.as_u64());
        assert!(only.sentence().text().contains("march-final.pdf"));
        assert_eq!(only.lapses_in(), Some(300));
    }

    /// **What a paired machine proposed says which machine on every change**,
    /// and reads back as written.
    #[test]
    fn what_a_paired_machine_proposed_names_the_machine_on_every_change() {
        let (approvals, strings) = a_change_waiting();
        let told = ToAPerson::waiting_from_a_machine(
            approvals.waiting_at(the_moment()),
            "the reception machine",
            &strings,
            the_moment(),
        );
        let changes = told.changes().unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(
            changes.first().unwrap().from(),
            Some("the reception machine")
        );
        let written = told.written().unwrap();
        assert!(
            written.contains(r#""from":"the reception machine""#),
            "{written}"
        );
        assert_eq!(ToAPerson::read(&written).unwrap(), told);
    }

    /// A list with nothing on it is an answer and not an absence: a shell that
    /// could not tell *nothing is waiting* from *the daemon said nothing* would
    /// draw the last list it saw.
    #[test]
    fn nothing_waiting_is_an_answer_of_its_own() {
        let (approvals, strings) = a_change_waiting();
        let told = ToAPerson::waiting(approvals.waiting_at(an_hour_in()), &strings, an_hour_in());
        assert_eq!(told.changes().unwrap().len(), 0);
        let written = told.written().unwrap();
        assert_eq!(ToAPerson::read(&written).unwrap(), told);
    }

    /// **A change that is only proposed is the agent's side of a turn**, and a
    /// person's door does not read one: what is waiting for them arrives as
    /// `ToAPerson::Waiting`, with the sentence they are answering.
    ///
    /// *This test also asserted that a model's answer is refused here, and
    /// that stopped being true on 2026-10-19.* A person asks their own
    /// questions now ([ADR 0126](../../../docs/decisions/0126-a-persons-own-question-is-a-third-authority-on-the-indicator-and-in-the-record.md)
    /// re-measured ADR 0125's route), so an answer reaches this door too —
    /// the same `answered` message, because it is the same thing arriving
    /// somewhere else. What has not changed is **which door may send it**, and
    /// that is `SO_PEERCRED`'s and the daemon's rather than this crate's. The
    /// sentence that was here is kept above rather than deleted, because the
    /// next reader will want to know it was a rule and not an oversight.
    #[test]
    fn a_change_only_proposed_is_refused_here() {
        let (approvals, strings) = a_change_waiting();
        let proposed = ToAnAgent::proposed(the_change(&approvals), &strings, the_moment())
            .written()
            .unwrap();
        assert_eq!(
            ToAPerson::read(&proposed),
            Err(NotUnderstood::NotAnAnswerForAPerson)
        );
    }

    /// **One `answered` message, read the same at both doors.**
    ///
    /// Written by the agent's side and read by the person's, which is the shape
    /// this deliberately has: a second message for the person's answer would be
    /// two things to keep in step, and the thing that differs is who may send
    /// it rather than what it says.
    #[test]
    fn one_answer_reads_the_same_whichever_door_it_arrives_at() {
        let answered = ToAnAgent::answered("three", &a_sentence(), "mistral-small-latest")
            .written()
            .unwrap();
        let (text, came_from, model) = ToAPerson::read(&answered)
            .unwrap()
            .an_answer()
            .map(|(text, came_from, model)| {
                (
                    text.to_owned(),
                    came_from.text().to_owned(),
                    model.to_owned(),
                )
            })
            .unwrap();
        assert_eq!(text, "three");
        assert_eq!(came_from, a_sentence().text());
        assert_eq!(model, "mistral-small-latest");
    }

    /// **Declining carries nothing about why**, because nothing was asked. The
    /// wire has no field for a reason, so nothing can start collecting one.
    #[test]
    fn a_declined_change_says_nothing_about_why() {
        let written = ToAPerson::Declined.written().unwrap();
        assert_eq!(written, r#"{"format":1,"tells":{"declined":{}}}"#);
        assert_eq!(ToAPerson::read(&written).unwrap(), ToAPerson::Declined);
    }

    /// What the change did comes back to the person who approved it, which is
    /// the other half of *a read answers inside the turn*: a change answers to
    /// whoever said yes.
    #[test]
    fn what_an_approved_change_did_comes_back_to_the_person() {
        let told = ToAPerson::did(Done::of_a_file_verb(&Answer::Renamed(
            "/home/anna/Invoices/march-final.pdf".into(),
        )));
        assert_eq!(
            told.done().unwrap(),
            &Done::Renamed {
                now_at: Some("/home/anna/Invoices/march-final.pdf".to_owned())
            }
        );
        assert!(told.changes().is_none());
        assert!(told.refusal().is_none());
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected value is the failure being reported"
)]
#[path = "a_persons_own_question_tests.rs"]
mod a_persons_own_question;
