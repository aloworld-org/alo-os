//! Whether the panel is collapsed, remembered for each Place separately.
//!
//! Task 1's owed clause, and the one thing that kept that task open after everything else in
//! it was built. `docs/autonomy/putting-a-window-aside.md` said the collapse choice *persists
//! per Place*, and for a day it did not: the choice was held once for the whole panel, because
//! there was no canvas Place to key it by.
//!
//! **It was blocked on Place identity and the canvas owned it.** The owner ruled on 2026-09-30
//! that Places are not to be reduced to a minimal implementation, that panel behaviour
//! depending on them is not to be deferred, and that the canvas owns Place identity. So a
//! `PlaceId` invented here would have been the surface least entitled to define it doing so.
//! `alo_canvas::Place` landed on 2026-10-01, and this is that clause being paid rather than
//! moved.
//!
//! # Why this is its own file
//!
//! `panel.rs` changes when what is *put aside* changes. This changes when what is *remembered
//! about a surface* changes. Two reasons to change, so two files — the fourth law, and the
//! same split `Shown` was pulled out of `proposing.rs` for.
//!
//! # Absent means expanded, and that is the type's own answer rather than a choice made here
//!
//! A Place a person has never collapsed is expanded. That is not a default invented for a
//! missing entry: [`Chosen`] carries `#[default]` on `Expanded` and its own doc says *what a
//! panel does by default*, so the absent case and the fresh case give the same answer because
//! they **are** the same answer.
//!
//! This matters more than it looks. The alternative — storing `Expanded` for every Place the
//! moment it is seen — would make *never collapsed* and *collapsed then expanded again*
//! different states in the map while being the same state to a person, and anything that later
//! writes this to disk would carry a row per Place a person had merely looked at.
//!
//! # Nothing here is a `Camera`, and the crate still cannot reach one
//!
//! A [`Place`] is a number the canvas minted, not a position, so holding one is not this crate
//! learning where anything is. `tests/the_panel_never_reaches_the_camera.rs` goes on passing,
//! and it has to: the panel is a viewport surface, and `alo-canvas`'s rule is that nothing in
//! that layer may read the camera to correct itself.

use alo_canvas::Place;
use std::collections::BTreeMap;

use crate::showing::Chosen;

/// What the person chose on each Place they have chosen anything on.
///
/// **A map rather than a field, because the choice belongs to the surface and not to the
/// panel.** A person who collapses the rail on the Place they are writing on has said nothing
/// about the Place they keep their reading on, and a single field cannot tell those apart.
///
/// `BTreeMap` rather than `HashMap` so that iteration is in Place order, which matters for the
/// one thing that reads it other than a lookup: anything that writes this down wants a stable
/// order, or two identical states produce two different files.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TheCollapseChoice {
    /// Only the Places a person has actually chosen on. See the header: absent is expanded.
    chosen: BTreeMap<Place, Chosen>,
}

impl TheCollapseChoice {
    /// Nothing chosen anywhere, which is what a fresh session is.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// What the person chose on this Place.
    ///
    /// [`Chosen::Expanded`] for a Place they have never chosen on, which is the same answer a
    /// fresh panel gives and for the same reason — see the header.
    #[must_use]
    pub fn on(&self, place: Place) -> Chosen {
        self.chosen.get(&place).copied().unwrap_or_default()
    }

    /// Collapse the panel on this Place, leaving every other Place alone.
    pub fn collapse(&mut self, place: Place) {
        self.chosen.insert(place, Chosen::Collapsed);
    }

    /// Expand the panel on this Place, leaving every other Place alone.
    ///
    /// **Removes the entry rather than storing `Expanded`.** Expanded is what absent means, so
    /// writing it down would be recording *this Place is in the state every Place starts in* —
    /// a row that says nothing, in a map something will eventually persist. The header gives
    /// the sharper reason: it would make *never collapsed* and *expanded again* two states
    /// that a person cannot tell apart.
    pub fn expand(&mut self, place: Place) {
        self.chosen.remove(&place);
    }

    /// How many Places a person has collapsed.
    ///
    /// For a caller deciding whether there is anything worth writing down, and for a test
    /// asserting that expanding really does forget rather than merely overwrite.
    #[must_use]
    pub fn how_many_are_collapsed(&self) -> usize {
        self.chosen.len()
    }
}
