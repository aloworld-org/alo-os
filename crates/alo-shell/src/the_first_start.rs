//! **The first-start sheets a machine with nobody on it shows**: the overview
//! of frame `09 · Language, region and keyboard`, and the three pickers behind
//! its rows — `09b` the language, `09c` the region, `09a` the keyboard.
//!
//! # The design's shape, and the owner's
//!
//! `09` draws three rows, each naming what is set now — *Language · English*,
//! *Region · Belgium*, *Keyboard · Belgian AZERTY* — over *Continue* and
//! *Back*. Each row opens its picker; each picker's way on returns to the
//! overview with the row changed, and its *Back* returns with nothing changed.
//! *Continue* goes on to the first account. The owner decided on 2026-10-28 to
//! build it to the design, and that **a keyboard different from the language
//! must work** — English read on a German keyboard.
//!
//! # What follows what
//!
//! - **The language decides the defaults of the other two**, until they are
//!   chosen: the region is where the machine is or the language's usual
//!   region (`alo_formats::where_the_machine_is`), and the keyboard is the one
//!   the language is offered first (`alo_keyboards::offered_with`). A region
//!   or keyboard a person chose stays chosen when they change language.
//! - **The screen speaks the language from the press that chose it**, and
//!   the compositor applies a keyboard the moment it is chosen, so the test
//!   row and every field after are typed through it.
//!
//! # Nothing here writes anything
//!
//! What was chosen leaves through [`TheFirstStart::the_language_chosen`],
//! [`TheFirstStart::the_keyboard_chosen`] and
//! [`TheFirstStart::the_region_chosen`], and the compositor keeps it.

use alo_formats::RegionCode;
use alo_keyboards::Layout;

use crate::sign_in_keys::SignInKey;
use crate::the_keyboard_picker::{TheKeyboardPicker, TheRing};
use crate::the_languages_offered::{TheLanguagesOffered, WhereTheRingIs};
use crate::the_regions_offered::TheRegionsOffered;

/// Which sheet is up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sheet {
    /// Frame `09`: the three rows.
    Overview,
    /// Frame `09b`, and the language sheet as it was when it opened, for Back.
    Language(TheLanguagesOffered),
    /// Frame `09c`.
    Region(TheRegionsOffered),
    /// Frame `09a`.
    Keyboard(TheKeyboardPicker),
}

/// The overview's three rows, in the design's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    /// The language alo OS speaks.
    Language,
    /// The region dates and numbers are written like.
    Region,
    /// The keyboard typed on.
    Keyboard,
}

impl Row {
    /// The three, top to bottom.
    pub const ALL: [Self; 3] = [Self::Language, Self::Region, Self::Keyboard];
}

/// What a key did to the sheets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Went {
    /// The sheets are still up.
    Stayed,
    /// *Continue*: on to the first account.
    Onward,
}

/// **Everything the first-start sheets hold.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheFirstStart {
    /// Which sheet is up.
    sheet: Sheet,
    /// The overview's cursor, a row of [`Row::ALL`].
    on: usize,
    /// The overview's ring: on the rows, or on *Continue*.
    ring: WhereTheRingIs,
    /// The languages, and the one chosen.
    languages: TheLanguagesOffered,
    /// The region a person took, where they took one.
    region: Option<RegionCode>,
    /// The keyboard a person chose, where they chose one.
    keyboard: Option<Layout>,
    /// The region the machine's own clock says it is in.
    where_the_machine_is: Option<RegionCode>,
    /// What each keyboard is called, in each language the machine has
    /// `xkeyboard-config`'s catalogue for — read once, at boot.
    names: alo_keyboards::TheKeyboardNames,
}

impl TheFirstStart {
    /// The overview, with nothing chosen, on a machine whose clock says it is
    /// in `where_the_machine_is`.
    #[must_use]
    pub fn arriving(where_the_machine_is: Option<RegionCode>) -> Self {
        Self {
            sheet: Sheet::Overview,
            on: 0,
            ring: WhereTheRingIs::OnTheList,
            languages: TheLanguagesOffered::every_official(),
            region: None,
            keyboard: None,
            where_the_machine_is,
            names: alo_keyboards::TheKeyboardNames::default(),
        }
    }

    /// Name keyboards with `names` from now on.
    pub fn naming_keyboards_with(&mut self, names: alo_keyboards::TheKeyboardNames) {
        self.names = names;
    }

    /// The machine's own clock says it is in `region`, or nowhere.
    pub fn on_a_machine_in(&mut self, region: Option<RegionCode>) {
        self.where_the_machine_is = region;
    }

    /// **What a keyboard is called** for the person reading the sheets.
    pub fn the_name_of(&self, keyboard: &Layout) -> String {
        self.names.of(keyboard, self.the_language())
    }

    /// Which sheet is up.
    pub fn sheet(&self) -> &Sheet {
        &self.sheet
    }

    /// The overview's cursor row.
    pub fn row(&self) -> Row {
        Row::ALL.get(self.on).copied().unwrap_or(Row::Language)
    }

    /// The overview's ring.
    pub fn ring(&self) -> WhereTheRingIs {
        self.ring
    }

    /// **The language the overview names**: chosen, or the one the machine
    /// already speaks.
    pub fn the_language(&self) -> &'static str {
        self.languages
            .chosen_or_already_spoken()
            .map_or(alo_strings::union::THE_SOURCE, |it| it.tag)
    }

    /// What the language calls itself, for its row.
    pub fn the_language_calls_itself(&self) -> &'static str {
        self.languages
            .chosen_or_already_spoken()
            .map_or("", |it| it.calls_itself)
    }

    /// **The region the overview names**: taken, or where the machine is, or
    /// the language's usual region.
    pub fn the_region(&self) -> Option<RegionCode> {
        self.region.clone().or_else(|| {
            self.where_the_machine_is.clone().or_else(|| {
                alo_formats::where_the_machine_is::the_usual_region_of(self.the_language())
            })
        })
    }

    /// **The keyboard the overview names**: chosen on the keyboard sheet, or
    /// the one the language is offered first.
    pub fn the_keyboard(&self) -> Layout {
        if let Sheet::Keyboard(picker) = &self.sheet
            && let Some(chosen) = picker.chosen()
        {
            return chosen.clone();
        }
        self.keyboard.clone().unwrap_or_else(|| {
            alo_strings::Language::written(self.the_language())
                .map(|it| alo_keyboards::Keyboards::offered_with(&it).in_use().clone())
                .unwrap_or_else(|_| alo_keyboards::Keyboards::shipped().in_use().clone())
        })
    }

    /// **The language a person chose**, for the compositor to keep — on the
    /// language sheet as soon as a row is chosen, so the screen speaks it.
    pub fn the_language_chosen(&self) -> Option<&'static str> {
        self.languages.chosen().map(|it| it.tag)
    }

    /// **The keyboard to type on now**: chosen, or the chosen language's —
    /// and [`None`] while nobody has chosen either, so the machine keeps the
    /// keyboard it booted with.
    pub fn the_keyboard_chosen(&self) -> Option<Layout> {
        let something_chosen = self.keyboard.is_some()
            || self.languages.chosen().is_some()
            || matches!(&self.sheet, Sheet::Keyboard(picker) if picker.has_been_chosen());
        something_chosen.then(|| self.the_keyboard())
    }

    /// **The region a person took** with *Use this region*.
    pub fn the_region_chosen(&self) -> Option<&RegionCode> {
        self.region.as_ref()
    }

    /// The languages, for the language sheet's drawing.
    pub fn languages(&self) -> &TheLanguagesOffered {
        &self.languages
    }

    /// **One key**, on whichever sheet is up.
    pub fn pressed(&mut self, key: SignInKey) -> Went {
        let sheet = std::mem::replace(&mut self.sheet, Sheet::Overview);
        match sheet {
            Sheet::Overview => self.on_the_overview(key),
            Sheet::Language(before) => {
                self.on_the_language_sheet(key, before);
                Went::Stayed
            }
            Sheet::Region(offered) => {
                self.on_the_region_sheet(key, offered);
                Went::Stayed
            }
            Sheet::Keyboard(picker) => {
                self.on_the_keyboard_sheet(key, picker);
                Went::Stayed
            }
        }
    }

    /// The overview: arrows move between the rows, Tab crosses to
    /// *Continue*, Enter opens a row's picker or goes on.
    fn on_the_overview(&mut self, key: SignInKey) -> Went {
        match key {
            SignInKey::Earlier => {
                self.ring = WhereTheRingIs::OnTheList;
                self.on = self.on.saturating_sub(1);
            }
            SignInKey::Later => {
                self.ring = WhereTheRingIs::OnTheList;
                self.on = self.on.saturating_add(1).min(Row::ALL.len() - 1);
            }
            SignInKey::OtherField => {
                self.ring = match self.ring {
                    WhereTheRingIs::OnTheList => WhereTheRingIs::OnTheWayOn,
                    WhereTheRingIs::OnTheWayOn => WhereTheRingIs::OnTheList,
                };
            }
            SignInKey::Enter => match self.ring {
                WhereTheRingIs::OnTheWayOn => return Went::Onward,
                WhereTheRingIs::OnTheList => self.open(self.row()),
            },
            // There is nowhere further back than the first sheet a machine
            // shows, which is `alo_access::leaving`'s reason for Escape here.
            SignInKey::Clear | SignInKey::Letter(_) | SignInKey::Erase | SignInKey::Nothing => {}
        }
        Went::Stayed
    }

    /// Open a row's picker.
    fn open(&mut self, row: Row) {
        self.sheet = match row {
            Row::Language => Sheet::Language(self.languages.clone()),
            Row::Region => Sheet::Region(TheRegionsOffered::named_in(
                self.the_language(),
                self.the_region().as_ref(),
            )),
            Row::Keyboard => {
                let offered = alo_strings::Language::written(self.the_language())
                    .map(|it| alo_keyboards::the_keyboards_offered(&it))
                    .unwrap_or_default();
                Sheet::Keyboard(TheKeyboardPicker::offering(
                    offered,
                    Some(&self.the_keyboard()),
                ))
            }
        };
    }

    /// The language sheet: the list as before, *Use this language* returns
    /// to the overview, Escape returns with the language as it was.
    fn on_the_language_sheet(&mut self, key: SignInKey, before: TheLanguagesOffered) {
        let offered = &mut self.languages;
        match key {
            SignInKey::Earlier => offered.up(),
            SignInKey::Later => offered.down(),
            SignInKey::OtherField => offered.ring_crosses(),
            SignInKey::Enter => match offered.ring() {
                WhereTheRingIs::OnTheWayOn => {
                    offered.ring_to_the_list();
                    return;
                }
                WhereTheRingIs::OnTheList => {
                    offered.choose();
                    offered.ring_to_the_way_on();
                }
            },
            SignInKey::Clear => {
                self.languages = before;
                return;
            }
            SignInKey::Letter(_) | SignInKey::Erase | SignInKey::Nothing => {}
        }
        self.sheet = Sheet::Language(before);
    }

    /// The region sheet: *Use this region* takes the region and returns;
    /// Escape goes back within the sheet, and then to the overview.
    fn on_the_region_sheet(&mut self, key: SignInKey, mut offered: TheRegionsOffered) {
        match key {
            SignInKey::Earlier => offered.up(),
            SignInKey::Later => offered.down(),
            SignInKey::OtherField => offered.ring_crosses(),
            SignInKey::Enter => match offered.ring() {
                WhereTheRingIs::OnTheWayOn => {
                    self.region = offered
                        .chosen()
                        .and_then(|it| RegionCode::named(it.code).ok());
                    return;
                }
                WhereTheRingIs::OnTheList => offered.enter_on_the_list(),
            },
            SignInKey::Clear => {
                if !offered.back() {
                    return;
                }
            }
            SignInKey::Letter(_) | SignInKey::Erase | SignInKey::Nothing => {}
        }
        self.sheet = Sheet::Region(offered);
    }

    /// The keyboard sheet: arrows choose among the keyboards, Tab reaches the
    /// test row and the way on, letters on the test row are the test.
    fn on_the_keyboard_sheet(&mut self, key: SignInKey, mut picker: TheKeyboardPicker) {
        match key {
            SignInKey::Earlier => picker.up(),
            SignInKey::Later => picker.down(),
            SignInKey::OtherField => picker.ring_crosses(),
            SignInKey::Enter => match picker.ring() {
                TheRing::OnTheWayOn => {
                    self.keyboard = picker.chosen().cloned();
                    return;
                }
                TheRing::OnTheKeyboards => picker.choose(),
                TheRing::OnTheTest => {}
            },
            SignInKey::Letter(letter) => picker.typed_a(letter),
            SignInKey::Erase => picker.erased(),
            SignInKey::Clear => return,
            SignInKey::Nothing => {}
        }
        self.sheet = Sheet::Keyboard(picker);
    }
}

#[cfg(test)]
#[path = "the_first_start_tests.rs"]
mod tests;
