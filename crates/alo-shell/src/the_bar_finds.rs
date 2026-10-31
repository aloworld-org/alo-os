//! What the alo Bar finds for what somebody typed, and in what order.
//!
//! **The Bar's open state is designed** — `alo Bar / Open` (`81:890`) inside
//! `09 · Find, open, or ask` (`78:1671`): a panel over the canvas with a
//! heading, the field, a line saying where the results came from, and rows of
//! `Name · where it is`. This file is the half that decides what those rows
//! are; `crate::bar_raster` draws them and `crate::a_key_reaches_the_bar`
//! carries one out.
//!
//! # Applications first, and the rows say which kind each is
//!
//! The design's own mock shows documents and windows beside applications —
//! *Launch strategy · Docs*, *Opening scene · Blender*. **This slice finds
//! applications only**, and the row still names the source, so adding a second
//! kind later adds rows rather than changing the shape of one. A row that did
//! not say where a result came from would have to change the day a second
//! source arrived, and every translation with it.
//!
//! # It decides; it reads nothing
//!
//! No scan and no subprocess. What this machine has installed reaches the seat
//! through `Server::what_this_machine_has`, which is *told rather than read*
//! for the reason that field states: reading it means running the sandbox's
//! tool and waiting, on the compositor's input loop, and a press that waited on
//! a subprocess would stop every window on the machine drawing.
//!
//! # The name and the identifier, both
//!
//! This said, until 2026-10-31, that *the day entries are read, this matches
//! on the name and the identifier both, and the rows stop looking like
//! addresses.* **They are read now**: `alo-desktop` reads every desktop entry
//! on the machine and tells the seat what each calls itself
//! (`crate::Server::the_machine_now_offers`), so *writer* finds *LibreOffice
//! Writer* and the row says so. An application whose entry gave no name is
//! matched and shown by its identifier, as before. Guessing a display name
//! from an identifier would be inventing one, which is the thing
//! `crate::which_disks_may_be_offered` refuses for a disk and refuses here for
//! the same reason.
//!
//! **What is still not matched is what an application *is*.** *browser* does
//! not find Firefox and *files* does not find Dolphin: that is an entry's
//! `GenericName`, `Keywords` or `Categories`, and nothing in
//! `docs/features.md` or the Bar's design promises a search by them. It is a
//! question for the owner, not a gap this file closes by guessing.

use alo_strings::{Filling, Strings};

use alo_dock::words;

/// One row in the Bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AResult {
    /// What a person reads, which the design draws as `Name · where it is`.
    pub shown: String,
    /// What carrying it out does.
    pub does: WhatARowDoes,
}

/// What pressing Enter on a row does.
///
/// **An enumeration rather than a closure**, so that what a row does can be
/// compared in a test and matched exhaustively when a second kind of result
/// arrives — a row holding a function to call is a row nothing can assert
/// about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatARowDoes {
    /// Start this application, by the identifier the machine knows it by.
    OpensAnApplication(String),
}

/// What the Bar finds for `typed`, in the order a person should read them.
///
/// **An empty query finds nothing**, deliberately: the Bar before anybody has
/// typed shows its prompt, not every application on the machine. A list that
/// appeared on focus would make the commonest thing a person does — press the
/// chord and start typing — a list flashing past under their words.
pub(crate) fn what_the_bar_finds(
    typed: &str,
    installed: &[String],
    called: &TheirNames,
    strings: &Strings,
) -> Vec<AResult> {
    let looking_for = typed.trim().to_lowercase();
    if looking_for.is_empty() {
        return Vec::new();
    }

    what_matches(typed, installed, called)
        .into_iter()
        .map(|identifier| AResult {
            shown: strings
                .say(
                    &words::AN_APPLICATION_ROW.key(),
                    &Filling::of("what", readable(identifier, called)),
                )
                .into_text(),
            does: WhatARowDoes::OpensAnApplication(identifier.clone()),
        })
        .collect()
}

/// Which applications what was typed matches, best first.
///
/// **Separate from the rows above, and that separation is the point.** Enter
/// has to know what would open without wording anything — it has no
/// vocabulary to hand and does not need one — while the rows have to be said.
/// One matcher means the thing a person is shown and the thing Enter opens
/// cannot be worked out differently.
///
/// **Three ranks, and the order inside each is the machine's own.** Somebody
/// typing `fire` means Firefox before they mean anything with *fire* in the
/// middle of it, and `org.moz` is what a person types only when they already
/// know what they are after. Sorting inside a rank would be a second opinion
/// about what is likeliest, which this product does not have.
pub fn what_matches<'a>(
    typed: &str,
    installed: &'a [String],
    called: &TheirNames,
) -> Vec<&'a String> {
    let looking_for = typed.trim().to_lowercase();
    if looking_for.is_empty() {
        return Vec::new();
    }
    let mut ranked: Vec<(u8, &String)> = installed
        .iter()
        .filter_map(|identifier| {
            let whole = identifier.to_lowercase();
            let last = whole.rsplit('.').next().unwrap_or(&whole).to_owned();
            // **The name the entry gives, and each word of it.** *writer*
            // finds *LibreOffice Writer* as readily as *libre* does, which is
            // what a person typing the name they see expects.
            let name = called
                .get(identifier)
                .map(|name| name.to_lowercase())
                .unwrap_or_default();
            let a_word_of_the_name_starts =
                name.split_whitespace().any(|word| word.starts_with(&looking_for));
            if last.starts_with(&looking_for) || a_word_of_the_name_starts {
                Some((0, identifier))
            } else if last.contains(&looking_for) || name.contains(&looking_for) {
                Some((1, identifier))
            } else if whole.contains(&looking_for) {
                Some((2, identifier))
            } else {
                None
            }
        })
        .collect();
    ranked.sort_by_key(|(rank, _)| *rank);
    ranked
        .into_iter()
        .map(|(_, identifier)| identifier)
        .collect()
}

/// What a row calls an application: the name its entry gives, or the part of
/// its identifier a person would recognise when it gives none.
///
/// `org.mozilla.firefox` reads as `Firefox` when its entry says so, and as
/// `firefox` when nothing does. **The fallback is not capitalised and not
/// prettified**: `org.gnome.TextEditor` reads as `TextEditor` because that is
/// what the identifier says, and turning it into `Text Editor` would be this
/// product inventing a name for somebody else's application.
fn readable<'a>(identifier: &'a str, called: &'a TheirNames) -> &'a str {
    match called.get(identifier) {
        Some(name) => name,
        None => identifier.rsplit('.').next().unwrap_or(identifier),
    }
}

/// What each installed application calls itself, by identifier.
pub type TheirNames = std::collections::BTreeMap<String, String>;

/// How many results there are, said as the line under the field.
///
/// The design draws *On this computer · 3 open windows* there. **It says what
/// was searched and how much was found**, so a person who typed something that
/// finds nothing reads a sentence rather than meeting an empty space.
pub(crate) fn where_these_came_from(how_many: usize, strings: &Strings) -> String {
    let word = if how_many == 1 {
        words::ONE_APPLICATION_HERE
    } else {
        words::APPLICATIONS_HERE
    };
    strings
        .say(&word.key(), &Filling::of("how_many", how_many.to_string()))
        .into_text()
}

/// The same line, when nothing on this machine is chosen to answer a question.
///
/// **In the place the count goes, worded to the same shape**, so that the two
/// read as one line changing rather than two different lines appearing. It is
/// about questions only — typing an application's name still opens it, and those
/// rows are directly below this line.
pub(crate) fn nothing_can_answer_a_question(strings: &Strings) -> String {
    strings
        .say(
            &words::NOTHING_CAN_ANSWER_A_QUESTION.key(),
            &Filling::nothing(),
        )
        .into_text()
}

#[cfg(test)]
#[path = "the_bar_finds_tests.rs"]
mod tests;
