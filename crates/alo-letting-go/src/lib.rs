//! Letting go of what an undo could have put back — the half of ★ *undo what
//! the agent did* that was decided and never built.
//!
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md)'s first
//! two accepted terms both end in a snapshot being **removed**: what falls
//! outside a bounded window goes, and under a named amount of free space the
//! oldest go first. `alo_keeping_up::HowFarBack` decided precisely which ones —
//! with no clock and no file, and it is right — and on 2026-09-21 a search of
//! every crate for a snapshot deletion found none. **The window was built; the
//! forgetting was not.** So a machine decided that a snapshot had expired and
//! then kept it for ever, which is the disk filling quietly that the owner's
//! question at acceptance was about. This crate is the forgetting.
//!
//! | | |
//! |---|---|
//! | [`the_folder`] | Where the machine keeps what an undo would put back, and the two small files that say what is in it |
//! | [`found`] | That folder, read off a disk — and everything it would not read and therefore left alone |
//! | [`Changes`], [`Settings`], [`keeping`] | `undo.toml`: the person's one setting, and how their file is kept (ADR 0038) |
//! | [`FileNotRead`], [`FileNotWritten`] | What they are told when that file will not read, in their own language |
//! | [`WhatGoes`] | Which of their kept turns go, and in what order — asking `alo-keeping-up`, never deciding again |
//! | [`THE_FLOOR`], [`AskingTheDisk`] | How much room this machine keeps, and the two questions it asks a filesystem |
//! | [`Removing`], [`WithTheBase`] | The base's own `btrfs subvolume delete`, with fixed arguments and no shell |
//! | [`sweep`], [`Swept`] | One firing of the timer, whole |
//! | [`THE_RECORD`], [`WritingItDown`] | Where it writes down which turns can no longer be put back |
//!
//! # It is a unit on a timer, and it is not a verb
//!
//! ADR 0045's seventh term, amended on 2026-09-21 after the first real `btrfs`
//! install measured why: **taking a read-only snapshot needs no capability and
//! removing one needs `CAP_SYS_ADMIN`**, and a read-only snapshot does not
//! yield to `rm -rf` either. `alo-turn` runs as the person and must not hold
//! that capability all day for an act performed once a day. And the broker's
//! fixed list does not gain one either, for a reason stronger than surface
//! area: **an agent that can forget an undo can erase the evidence of what it
//! did.** Undo is the record's counterpart, and a destructive verb over the
//! written-down past is the one verb whose approval a person is least able to
//! judge, because what it destroys is the thing they would judge it by.
//!
//! So there is **no request, no approval, no grant and no entry point**.
//! `tests/nothing_here_is_a_verb.rs` holds `alo_broker::SystemVerb`'s own list
//! to that: no name on it begins `undo.`, and the list has not grown. Expiry is
//! housekeeping. A person changes the window in their settings, and the unit
//! obeys it.
//!
//! # What a person is told, and where
//!
//! Two roads, and neither of them is this crate inventing a sentence about
//! somebody's turns. The **record** says which turns can no longer be put back,
//! naming them in the words the person approved when they ran
//! (`alo_record::Entry::let_go`, worded by `alo-recounting`). Their own
//! **settings** say how far back the machine keeps them, and the eight strings
//! [`words`] declares are all about that one file. The unit itself says what it
//! did to the journal, in English, for whoever administers the machine — the
//! shape `alo-brokerd`'s unit programs already have.
//!
//! # What is deliberately not here
//!
//! **Taking a bracket.** That is `alo-turn`'s, at the moment a changing turn
//! begins, and it needs no capability. This crate only ever removes.
//!
//! **Deciding a window.** `alo_keeping_up::HowFarBack` is the one answer to
//! *what does seven days or fifty changing turns still reach*, and nothing here
//! works it out again.
//!
//! **Forgetting everything, as one act.** `alo_keeping_up::WhatWasKept::forgetting`
//! is the person's own deliberate act with a sentence on it, and building it is
//! its own change — additive, through the same remover.
//!
//! **A clock.** [`sweep`] is handed the moment, for `alo-keeping-up`'s reason:
//! one firing is measured against one moment rather than however many the walk
//! took.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod changes;
pub mod deciding;
pub mod found;
pub mod keeping;
pub mod removing;
pub mod sweeping;
pub mod the_disk;
pub mod the_folder;
pub mod unkept;
pub mod words;
pub mod writing_it_down;

#[cfg(test)]
mod testing;

pub use changes::{Changes, Settings};
pub use deciding::WhatGoes;
pub use found::{Everyone, Found, Whose};
pub use removing::{NotRemoved, Removing, WithTheBase};
pub use sweeping::{Swept, WhatItDid, sweep};
pub use the_disk::{AskingTheDisk, OnThisMachine, THE_FLOOR, WhatItIs};
pub use the_folder::{NotWhatItSays, THE_FOLDER, TheTurn, Theirs};
pub use unkept::{FileNotRead, FileNotWritten};
pub use words::{EVERY_WORD, WordsError, declare_into, letting_go_words};
pub use writing_it_down::{THE_RECORD, TheMachinesRecord, WritingItDown};
