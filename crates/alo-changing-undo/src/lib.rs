//! What a person sees of what their machine is keeping so that the agent's
//! changes can be put back, and the two things they may do about it.
//!
//! ★ *Every piece of this was built and none of it was in front of anybody.*
//! `alo_keeping_up::HowFarBack` decides how far back an undo reaches,
//! `alo_letting_go` reads and writes the file a person changes it in and holds
//! the one act that forgets it, `alo-measuring` counts what that is costing, and
//! `alo_keeping_up::WhatWasKept::forgetting` is the sentence a person approves.
//! Task 15 of `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` is the pane,
//! and ADR 0045 point 5 is why there is one: *what an undo may keep is visible
//! and forgettable*, and until this a person could not find out how far back
//! their machine kept what the agent changed, could not see what it was holding,
//! and had no way to ask for it back short of editing a settings file by hand.
//!
//! | | |
//! |---|---|
//! | [`WhatIsKept`] | How far back, what it is holding, and whether this machine keeps at all |
//! | [`Holding`] | `alo-measuring`'s answer about the cost, carried and never recomputed |
//! | [`Wanted`] | A window a person asked for, already checked |
//! | [`Offered`], [`Approved`] | The one act: offered, then approved once or declined |
//! | [`words`] | The three sentences that are this crate's, each with a note for whoever translates it |
//!
//! # Three sentences, because everything else was already written
//!
//! The sentence a person approves is `alo_keeping_up::WhatWasKept::forgetting`.
//! A pair of numbers that is not a window is that crate's refusal. *This machine
//! keeps nothing of what your files were* is its sentence too — the same fact an
//! undo refuses with, so a person meets one wording of it however they arrived.
//! Why a size is not the whole truth is `alo_measuring::Counted`'s. This crate
//! adds the three things only a pane could say, and *says* the rest.
//!
//! # It ships no dependency on `alo-letting-go`, and that is the design
//!
//! `alo-saying` is the machine's one vocabulary, so it ships every crate that
//! declares a word — which puts every such crate inside the process an agent's
//! turn runs in. `alo-letting-go` holds the one privileged remover on this
//! machine, and
//! `crates/alo-letting-go/tests/a_turn_cannot_arrive_at_this_road.rs` holds that
//! the only road into it from there is its vocabulary.
//!
//! So a pane that read the person's file or performed the act would carry that
//! road into a turn's process, and that test would fail — **correctly**. This
//! crate therefore does neither. It is the model of what a person reads and
//! decides: [`Wanted`] is a window for a caller to keep with
//! the crate that owns their settings file, and [`Approved`] is a person's warrant for a
//! caller to carry out. The caller is the session the person is sitting at,
//! which is not a turn. `tests/a_turn_cannot_arrive_at_this_pane.rs` holds this
//! crate to the same promise from the other side, by name, as task 15 asked.
//!
//! That the file and the act stay out of here is also why nothing in this crate
//! can be a second copy of them: it has no copy at all.
//!
//! # Nothing here is an agent verb, and no road reaches it
//!
//! ADR 0045's second term is that an undo is *the person's*, with **no agent
//! verb** that undoes and none that proposes one; its seventh keeps forgetting
//! off the broker's list as well, because an agent that can forget an undo can
//! erase the evidence of what it did. A pane narrows neither. This crate
//! declares no verb, is absent from `alo-declared` — which is every verb alo OS
//! ships — opens no socket, and reads no file.
//!
//! # Drawing is the shell's
//!
//! This is the pane's model, as `alo-changing-updates` is. How a number of bytes
//! reads in a person's language is a drawing decision and arrives here already
//! worded ([`WhatIsKept::holding_said`]), because a second opinion in this
//! repository about what `5183545344` should say would be one more answer nobody
//! could reconcile.

pub mod forgetting;
pub mod wanted;
pub mod what_is_kept;
pub mod words;

pub use forgetting::{Approved, Offered};
pub use wanted::Wanted;
pub use what_is_kept::{Holding, WhatIsKept};
pub use words::{EVERY_WORD, WordsError, changing_undo_words, declare_into};
