//! **How much can alo do?** — the sheet a person sees once, right after their
//! first sign-in, while no level for running code has been chosen.
//!
//! [ADR 0149](../../../docs/decisions/0149-full-access-is-everything-the-person-can-do.md)
//! decision 4, and the owner's four answers of 2026-11-09, quoted in
//! `docs/design/the-first-start.md`: shown **right after first sign-in**,
//! whenever no level has been chosen; the three choices with **Full access
//! already selected**; Settings' own lines under the other two and ADR 0149's
//! cost sentence under Full access; and **Continue only**.
//!
//! # The same setting through the same road as Settings
//!
//! This holds a [`HowCodeRunsHere`] — the very value Settings → Running code
//! holds — and Continue is [`HowCodeRunsHere::picked`], which is
//! `alo_remembering::the_level_kept` on `/var/lib/alo/how-code-runs.toml`.
//! There is no second copy of the answer and no second writer of the file.
//!
//! # Who is asked, and who is not
//!
//! Only a person with **no file at all** ([`HowCodeRunsHere::has_not_chosen`]).
//! A person who chose, in Settings or here, is never asked again; and a file
//! that **would not read** is not asked about either, because asking would
//! write over a file nobody has read — Settings refuses to replace one for the
//! same reason.
//!
//! # Moving is not choosing
//!
//! The arrow keys and Tab move the ring; only Enter, Space or a click on a row
//! chooses it (`docs/design/the-first-start.md`: *moving focus must never
//! silently commit a choice*). Nothing is written until Continue.
//!
//! # A choice that cannot be saved never traps anybody
//!
//! There is no Back and no Not now, so a sheet that kept a person until the
//! disk agreed would be a desktop taken away. The first refusal stays and says
//! so; the second goes on to the desktop with **nothing chosen** — which is
//! what the machine had before, and which asks again at the next sign-in.

use std::path::Path;

use alo_capability::HowCodeRuns;

use crate::a_press_on_a_setup_sheet::OnASheet;
use crate::how_code_runs_in_settings::HowCodeRunsHere;
use crate::settings_kept::SettingsKept;
use crate::sign_in_keys::SignInKey;

/// The three, in the order Settings and the design draw them: least to most
/// that alo may do.
pub(crate) const THE_THREE: [HowCodeRuns; 3] = HowCodeRuns::ALL;

/// **What the sheet arrives with chosen**: full access, the owner's *"First
/// start asks, full access pre-selected"* (ADR 0149).
pub(crate) const ARRIVES_CHOSEN: HowCodeRuns = HowCodeRuns::AtFullTrust;

/// Where the keyboard's ring is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TheRingIs {
    /// On a choice, by its place in [`THE_THREE`].
    OnAChoice(usize),
    /// On *Continue*.
    OnContinue,
}

/// What a key or a press did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Went {
    /// The sheet is still up.
    Stayed,
    /// The level was written down, and the desktop opens.
    Kept(HowCodeRuns),
    /// It could not be written twice, and the desktop opens with nothing
    /// chosen.
    OnWithNothingChosen,
}

/// Whether the first sign-in asks, and why not when it does not.
#[derive(Debug)]
pub(crate) enum ToAsk {
    /// Nobody has chosen: the sheet.
    Ask(HowMuchAloCanDo),
    /// This level was chosen already.
    AlreadyChosen(HowCodeRuns),
    /// The file is there and would not read, so nothing is asked and nothing
    /// is written over it.
    WouldNotRead,
}

/// The sheet's state.
#[derive(Debug)]
pub(crate) struct HowMuchAloCanDo {
    /// The setting, as Settings holds it.
    here: HowCodeRunsHere,
    /// The choice showing.
    chosen: HowCodeRuns,
    /// Where the ring is.
    ring: TheRingIs,
    /// Whether the last Continue could not be saved.
    not_saved: bool,
}

/// The place of a level in [`THE_THREE`].
fn place_of(level: HowCodeRuns) -> usize {
    THE_THREE.iter().position(|it| *it == level).unwrap_or(0)
}

impl HowMuchAloCanDo {
    /// Whether the person whose level is kept `at` is asked.
    pub(crate) fn to_ask(at: &Path) -> ToAsk {
        let here = HowCodeRunsHere::at_sign_in(at);
        if here.has_not_chosen() {
            return ToAsk::Ask(Self {
                here,
                chosen: ARRIVES_CHOSEN,
                ring: TheRingIs::OnAChoice(place_of(ARRIVES_CHOSEN)),
                not_saved: false,
            });
        }
        here.chosen()
            .map_or(ToAsk::WouldNotRead, ToAsk::AlreadyChosen)
    }

    /// The choice showing.
    pub(crate) const fn chosen(&self) -> HowCodeRuns {
        self.chosen
    }

    /// Where the ring is.
    pub(crate) const fn ring(&self) -> TheRingIs {
        self.ring
    }

    /// Whether the last Continue could not be saved.
    pub(crate) const fn was_not_saved(&self) -> bool {
        self.not_saved
    }

    /// One key.
    pub(crate) fn pressed(&mut self, key: SignInKey) -> Went {
        let last = THE_THREE.len() - 1;
        match key {
            SignInKey::Later | SignInKey::OtherField => {
                self.ring = match self.ring {
                    TheRingIs::OnAChoice(it) if it < last => TheRingIs::OnAChoice(it + 1),
                    TheRingIs::OnAChoice(_) => TheRingIs::OnContinue,
                    TheRingIs::OnContinue => TheRingIs::OnAChoice(0),
                };
                Went::Stayed
            }
            SignInKey::Earlier | SignInKey::FieldBefore => {
                self.ring = match self.ring {
                    TheRingIs::OnAChoice(0) => TheRingIs::OnContinue,
                    TheRingIs::OnAChoice(it) => TheRingIs::OnAChoice(it - 1),
                    TheRingIs::OnContinue => TheRingIs::OnAChoice(last),
                };
                Went::Stayed
            }
            SignInKey::Enter | SignInKey::Letter(' ') => match self.ring {
                TheRingIs::OnAChoice(it) => {
                    self.chose(it);
                    self.ring = TheRingIs::OnContinue;
                    Went::Stayed
                }
                TheRingIs::OnContinue => self.continued(),
            },
            // **Nothing else means anything here**: there is no field to type
            // in and no way back to clear to.
            SignInKey::Letter(_)
            | SignInKey::Erase
            | SignInKey::Clear
            | SignInKey::Nothing
            | SignInKey::TheScreenReader => Went::Stayed,
        }
    }

    /// One press of the pointer, on what the last frame drew.
    pub(crate) fn pressed_on(&mut self, on: OnASheet) -> Went {
        match on {
            OnASheet::Choice(it) if it < THE_THREE.len() => {
                self.chose(it);
                self.ring = TheRingIs::OnAChoice(it);
                Went::Stayed
            }
            OnASheet::WayOn => {
                self.ring = TheRingIs::OnContinue;
                self.continued()
            }
            // **No way back is drawn**, so a press there is a press on nothing;
            // and *Accessibility* opens nothing yet, as on every setup sheet.
            OnASheet::Choice(_) | OnASheet::WayBack | OnASheet::Accessibility => Went::Stayed,
        }
    }

    /// The row at `it` chosen; the refusal it may have been showing is over.
    fn chose(&mut self, it: usize) {
        if let Some(level) = THE_THREE.get(it) {
            self.chosen = *level;
            self.not_saved = false;
        }
    }

    /// *Continue*: the level written through Settings' own road.
    fn continued(&mut self) -> Went {
        match self.here.picked(self.chosen) {
            SettingsKept::Written => Went::Kept(self.chosen),
            SettingsKept::Refused
            | SettingsKept::NotWritten
            | SettingsKept::NowhereToKeep
            | SettingsKept::NotOffered => {
                if self.not_saved {
                    Went::OnWithNothingChosen
                } else {
                    self.not_saved = true;
                    Went::Stayed
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "how_much_alo_can_do_tests.rs"]
mod tests;
