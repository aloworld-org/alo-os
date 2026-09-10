//! Every verb alo OS ships, and how a person does the same thing without the
//! agent.
//!
//! `docs/features.md` promises at v0.01 that **anything an agent verb can do, a
//! person can do by hand**, and [ADR
//! 0009](https://github.com/aloworld-org/alo-os/blob/main/docs/decisions/0009-a-good-computer-without-the-agent.md)
//! is where that was decided and widened: the agent is unavailable for six
//! reasons and only one of them is a choice — declined at setup, no model
//! downloaded, offline, provider down, key expired, **and the money ran out.**
//! In every one of them the machine must lose *convenience* and never
//! *capability*, because a capability an agent has and a person does not makes
//! whoever cannot pay a second-class user of a computer they own.
//!
//! `docs/by-hand.md` is that rule, verb by verb. This crate is what makes the
//! document true of the verbs rather than a list somebody wrote once.
//!
//! # Why a crate and not a rule somebody remembers
//!
//! Because it has been a rule somebody remembers since 2026-09-02, and the audit
//! in `docs/autonomy/v0-01-evidence.md` found what that is worth: of forty-one
//! v0.01 promises, this is the one that is *a standing rule with nothing checking
//! it*. Ten verbs ship today and not one of them had ever been asked the
//! question. A verb arriving tomorrow with no by-hand answer would break nothing,
//! fail nothing, and nobody would be told.
//!
//! So it is arithmetic instead. Every verb the machine declares must be answered
//! exactly once, and every answer must be about a verb the machine still
//! declares. **A verb added and not answered fails the gate in the change that
//! adds it**, which is the only moment anybody has the knowledge to answer it —
//! `docs/contracts/agent-verbs.md` asks the question there, as rule 7 of adding a
//! verb.
//!
//! # The verbs are read out of the machine, never out of a list kept here
//!
//! [`held`] is handed an `alo_capability::Verbs` — the same registry a daemon
//! enforces — so a verb added to a crate this check already knows is a verb it
//! sees, with nothing to update. Nothing in this crate holds a verb's name.
//!
//! That leaves the failure a list could never catch, and it is the one that
//! actually happens: **a new crate declares verbs and nothing connects it to this
//! check.** One floor down, `crates/alo-overlay` declared nine strings that
//! `alo-saying` did not collect, and the only reason a shell would not have shown
//! a key where a sentence belongs is that somebody noticed. So
//! [`declaring`] walks this workspace's own member list, and a crate declaring
//! verbs that was not handed in is a finding naming it.
//!
//! # What counts as an answer, and what deliberately does not
//!
//! **A promise in `docs/features.md`, quoted.** Not a sentence somebody wrote
//! here — ADR 0009's rule is not *have an idea about how a person might manage*,
//! it is *no surface may be left out because an agent can do it instead*, and the
//! only place a surface is committed to is the definition, with a tier. So the
//! answer quotes the definition, and **the release that owns it is read off the
//! line rather than asserted beside it.**
//!
//! **Or the debt, with the release that owes it.** One verb needs this today:
//! `archive_folder` makes an archive and the definition promises archives that
//! *open*, so stretching the one to cover the other would be this check lying in
//! the first change that used it. [`owed`] is why the second form costs
//! something to write.
//!
//! This crate does not judge whether a surface *really* lets a person do what the
//! verb does — nothing mechanical can, and pretending otherwise would be the rule
//! failing in a new way. What it removes is the failure that happened: a verb
//! nobody asked the question about, and an answer pointing at a promise that has
//! since gone.
//!
//! # It says nothing to a person
//!
//! Nothing here reaches a screen. The reader of a [`Finding`] is whoever is
//! adding a verb, in the repository, with the two documents open — so the
//! findings keep their English and their `Display`, exactly as
//! `alo_saying::NotCollected` and `alo_reconciling::Finding` do for the same
//! reason. There are no strings to externalise here, and `alo-saying` does not
//! collect this crate.
//!
//! # It reads no disk
//!
//! [`held`] is handed the two documents' text and a way to read a file by its
//! repository-relative path. The test in `tests/` is what puts the real
//! repository behind it. That is what lets every refusal below be shown
//! happening against a fixture: **a check that has never been seen to refuse
//! anything passes on the day it stops looking, in exactly the same colour.**

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod answer;
pub mod by_hand;
pub mod declaring;
pub mod document;
pub mod entry;
pub mod finding;
pub mod holding;
pub mod owed;
pub mod promised;
pub mod release;
pub mod wrapping;

pub use answer::AN_ANSWER;
pub use by_hand::ByHand;
pub use declaring::{THE_WORKSPACE, whoever_declares_verbs};
pub use document::{THE_ANSWERS, entries_in};
pub use entry::Entry;
pub use finding::Finding;
pub use holding::{Held, held};
pub use owed::Owed;
pub use promised::{Promised, promises_in, releases_among};
pub use release::Release;
pub use wrapping::one_line;
