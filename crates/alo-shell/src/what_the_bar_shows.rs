//! **What the alo Bar shows**, as a frame hands it over: what is typed, the
//! line saying where the results came from, what was found, the row the
//! keyboard is on and the row that asks alo.
//!
//! It is drawn in one place: the compact list rising from the Dock's own field
//! (`crate::bar_matches_raster`), the owner's decision of 2026-11-11 — *"Clicking
//! 'Find, open, or ask…' on the Dock should not open the same panel as
//! Super+Space. It should allow user to text inside it"*, with Super+Space
//! *the same as clicking the field*.
//!
//! **The centred launcher that was drawn from this too is gone.** After that
//! decision it was left only for *Help with this window*, and that road moved
//! into the window's own panel the same day (#1182), so nothing opened it any
//! more. Its drawing, its placement, its press and the dimmed screen behind it
//! were removed rather than kept as a surface nobody can reach; its rows and
//! their measures live on in the list (`crate::the_launchers_rows`,
//! `crate::where_the_bars_matches_rise`).

use crate::the_bar_finds::AResult;
use smithay::utils::{Physical, Size};

/// The Bar's own words, already said in the person's language.
#[derive(Debug, Clone, Copy)]
pub struct TheLaunchersWords<'a> {
    /// *Open*, the hint on the chosen application row.
    pub open_it: &'a str,
}

/// The row that asks alo, already said.
#[derive(Debug, Clone, Copy)]
pub struct TheAskRow<'a> {
    /// *Ask alo “files”*.
    pub asking: &'a str,
    /// *a new task*.
    pub what_it_is: &'a str,
    /// Where an answer would come from, in `alo-models`' own sentence.
    pub from: &'a str,
}

/// What the Bar shows, as a frame hands it over.
#[derive(Debug, Clone, Copy)]
pub struct TheBarOnTheDisplay<'a> {
    /// What a person has typed.
    pub typed: &'a str,
    /// The line saying where the results came from — empty while nothing is
    /// typed, and then not drawn.
    pub origin: &'a str,
    /// Every result, in the order to read them.
    pub results: &'a [AResult],
    /// Which row the keyboard is on: an application's, or one past the last
    /// of them, the row that asks.
    pub chosen: Option<usize>,
    /// **The display**, so the list and its shadow are kept on it.
    pub display: Size<i32, Physical>,
    /// The Bar's own words.
    pub words: TheLaunchersWords<'a>,
    /// The row that asks alo, where alo may be asked — never while the person
    /// has set AI to *Not at all*, nor while nothing is chosen to answer.
    pub asks: Option<TheAskRow<'a>>,
    /// What each application draws, for its row's icon.
    pub artwork: Option<&'a alo_dock::artwork::TheArtwork>,
}
