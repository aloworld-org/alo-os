//! The section where a person picks how code the agent writes or fetches runs.
//!
//! [ADR 0064](../../../docs/decisions/0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)
//! makes running code a separate grant the person gives at one of three
//! levels, and law 2 makes it the person's alone. `docs/features.md` marks it
//! ★ and has named the same two things owed since 2026-10-14: *the setting a
//! person makes it in, the surface they make it on.* The setting landed on
//! 2026-10-22 as `/var/lib/alo/how-code-runs.toml`. **This is the surface, and
//! until it existed the choice could not be made by anybody** — the mechanism
//! was built, enforced, recorded and tested, and no screen named it.
//!
//! # It does not use `crate::settings_keepers::Keeper`, and that is the point
//!
//! Every other section does. `Keeper` requires `shipped()` — what the section
//! draws as the release ships it — and `put_back_as_shipped`, which writes that
//! value over a file that did not read. Both are right for the dock and the
//! accent, where there is always some state and the shipped one is as good as
//! any.
//!
//! **They are wrong here, and bending this section to fit would destroy the
//! thing the file was built to keep.** `alo_remembering::how_code_runs` keeps
//! **three** states apart: a level the person chose, **no file at all** for a
//! person who has not chosen, and a refusal. A `shipped()` default would make
//! the first indistinguishable from the second, and `put_back_as_shipped`
//! would turn *has not chosen* into *chose the sealed box* by writing a file —
//! which is a default dressed as their decision, and law 5's own words are
//! that absence is stronger than a default.
//!
//! So this section holds what it read and nothing else, and a person who has
//! never opened it has **no row ticked**.
//!
//! # What is drawn, and what every level keeps
//!
//! The three levels come from `alo_capability::HowCodeRuns::ALL`, in the order
//! that ADR lists them, which is least to most the agent may do. Each carries
//! its own name and, where it has one, **its cost in plain words** — law 5's
//! *offered with the risk in plain words*. Only full trust has a cost to say,
//! and it is said rather than acted on: nothing here refuses a level or delays
//! it.
//!
//! Beneath them go the three protections that stay on at every level —
//! `HowCodeRuns::what_stays_on`: the record, the snapshot taken before each
//! run, and the egress indicator. They are drawn because ADR 0064 keeps them
//! on *at full trust too*, and a person reading only the cost of full trust
//! would not know what they still have.

use std::path::{Path, PathBuf};

use crate::settings_kept::SettingsKept;
use alo_capability::HowCodeRuns;

/// What this machine's setting says, as the section found it.
///
/// Three values because the file keeps three states apart, and the section
/// must not flatten them: see this file's header for why `Keeper` is not used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TheLevelHere {
    /// The level this person chose.
    Chose(HowCodeRuns),
    /// **No file, which is a person who has not chosen** — not the sealed box,
    /// and not an error. Nothing is ticked for them.
    HasNotChosen,
    /// The file is there and would not read.
    ///
    /// **It carries no reason, and that is a gap rather than a decision.**
    /// `alo_remembering::NotRemembered` derives only `Debug` and `Error`: it
    /// has no `said(&Strings)`, so there is no sentence in the person's own
    /// language for *why* their level could not be read. `alo-choosing` words
    /// its refusals and this crate does not, which is where the fix belongs —
    /// a sentence invented here would be hardcoded English in a drawing crate,
    /// and `format!("{why:?}")` shown to a person is worse than that.
    ///
    /// So the section can say *this did not read* and cannot yet say why, and
    /// the drawing is honest about which of those it has.
    WouldNotRead,
}

/// The section, as one value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HowCodeRunsHere {
    /// What was read at sign-in, or since changed.
    found: TheLevelHere,
    /// Where it is kept, so that choosing needs nothing handed to it.
    at: PathBuf,
}

impl HowCodeRunsHere {
    /// What the section found at sign-in.
    ///
    /// A file that will not read is **not** replaced with anything: ADR 0038's
    /// rule, and here it is stronger than elsewhere, because the only thing to
    /// replace it with would be a level nobody picked.
    pub(crate) fn at_sign_in(at: &Path) -> Self {
        let found = match alo_remembering::the_level_remembered(at) {
            Ok(Some(chose)) => TheLevelHere::Chose(chose),
            Ok(None) => TheLevelHere::HasNotChosen,
            Err(_) => TheLevelHere::WouldNotRead,
        };
        Self {
            found,
            at: at.to_owned(),
        }
    }

    /// The level this person chose, or [`None`] for one who has not chosen and
    /// for a file that would not read.
    ///
    /// **Both of those answer `None` and they are not the same thing** — the
    /// difference is in [`TheLevelHere`] itself, which the section keeps
    /// whole. What they share is that no row is ticked, which is the
    /// only question this answers.
    pub(crate) const fn chosen(&self) -> Option<HowCodeRuns> {
        match self.found {
            TheLevelHere::Chose(chose) => Some(chose),
            TheLevelHere::HasNotChosen | TheLevelHere::WouldNotRead => None,
        }
    }

    // **There is no `did_not_read` accessor, and there was until it had no
    // caller.** The section knows the difference -- `TheLevelHere::WouldNotRead`
    // is its own value and the tests assert on it -- but nothing can draw that
    // difference yet, because `alo_remembering::NotRemembered` has no words. An
    // accessor whose only consumer is a sentence nobody can write is the fault
    // `alo-reached` refuses, and it made this lane's own pull request red hours
    // before this file existed. It comes back in the change that gives the
    // refusals words, which is where it will have something to say.

    /// This person picked a level.
    ///
    /// **Written first and drawn second.** A tick that moved before the file
    /// was written would tell somebody their machine runs code at a level it
    /// does not, and on this setting that is the difference between a sealed
    /// box and full trust. On a refusal the tick stays where it was.
    ///
    /// Answers in the same vocabulary as every other section —
    /// [`SettingsKept`] — so the drawing needs no sentence of this file's own.
    /// **What it cannot yet do is say *why* a change was not written**, for
    /// [`TheLevelHere::WouldNotRead`]'s reason: the crate that owns the file
    /// has no words for its refusals.
    pub(crate) fn picked(&mut self, chose: HowCodeRuns) -> SettingsKept {
        match alo_remembering::the_level_kept(&self.at, chose) {
            Ok(()) => {
                self.found = TheLevelHere::Chose(chose);
                SettingsKept::Written
            }
            Err(_) => SettingsKept::NotWritten,
        }
    }
}

#[cfg(test)]
#[path = "how_code_runs_in_settings_tests.rs"]
mod tests;
