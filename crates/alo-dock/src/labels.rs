//! Where the name of a thing in the dock is shown.
//!
//! **One state, and it used to be three.** `docs/features.md` promised that
//! *labels give way to icons where the short edge demands it*, and this file
//! was the answer to that clause: a name under an icon, a name beside one, and
//! a name that had given way because the bar had no room for it.
//!
//! The owner removed the row of names from the bar on 2026-10-10, after the
//! design was measured and found to have none:
//!
//! > No permanent application names beneath icons. Show names on hover and
//! > keyboard focus, outside the bar without changing its height. Screen
//! > readers always receive the application name.
//!
//! So there is nothing for a short edge to demand. What is left is where the
//! name appears when somebody asks for it, which is [`Labels`]'s one state.
//!
//! # Nothing was taken away, and that is stronger than it was
//!
//! EN 301 549 carries WCAG's requirement that text resize to 200% **without
//! loss of content or function**. The old answer was that a dock which dropped
//! names as the text grew did not lose them — they were still announced, still
//! shown on hover — and the reassurance was carried inside the string a person
//! read, so a translator was handed it and could not silently drop it.
//!
//! **The new answer needs no reassurance**: a name cannot be lost to a text
//! size that cannot reach it. The bar is a constant, the name is a tooltip, and
//! a screen reader is given the name whatever happens. `dock.labels.gave-way`
//! was the sentence a person read when the names left, and it is gone from the
//! vocabulary because there is no longer an occasion for it.

use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// Where the name of a thing in the dock appears.
///
/// **One state since 2026-10-10**, and it was three. `Under` put a name below
/// each icon and thickened the bar by a line of text; `GaveWay` was what
/// happened when that line no longer fitted. The owner removed the row:
///
/// > No permanent application names beneath icons. Show names on hover and
/// > keyboard focus, outside the bar without changing its height. Screen
/// > readers always receive the application name.
///
/// So neither state can arise. A variant nothing can construct is the shape
/// this repository keeps finding, so they are gone rather than left to read as
/// options.
///
/// **It stays an enum.** *The dock's size* is `[v0.5]` in `docs/features.md`
/// and a placement may come back with it; a single-variant enum gains variants
/// additively where a `bool` or a bare struct would have to be replaced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Labels {
    /// Beside the icon, in a tooltip that opens toward the canvas — and above
    /// it on a dock that runs across the screen. **Outside the bar either
    /// way.**
    ///
    /// **It does not thicken the dock, and that is the owner's rule rather than
    /// an economy.** Their words of 2026-10-04, about a dock down a side:
    /// *keep icons and click targets unchanged; labels do not permanently widen
    /// the Dock* — extended to every edge on 2026-10-10, when the measured
    /// design turned out to have no names in the bar at all.
    ///
    /// So a dock is `Room::a_dock_of_icons` thick whatever the text size is,
    /// and the name is a transient surface over the canvas rather than part of
    /// the bar.
    ///
    /// That is why this carries no measurement: the width a name gets is
    /// `crate::measures::A_NAME_BESIDE_AN_ICON` and does not vary with how much
    /// room the dock has. What the tooltip must do — appear on hover **and
    /// keyboard focus**, wrap to two lines then ellipsize, give the whole name
    /// to assistive technology, and stay inside the viewport on a small display
    /// — belongs to whatever draws it, and `alo-dock` draws nothing.
    Beside,
}

impl Labels {
    /// Whether a person can read the names.
    ///
    /// **Always, since 2026-10-10.** It answered `false` for `GaveWay`, which
    /// was a state where the name was not on the screen at all until somebody
    /// rested on the icon. There is no such state now: a name is shown on hover
    /// and on keyboard focus from every edge, and a screen reader is given it
    /// whatever happens.
    ///
    /// Kept rather than deleted because callers ask it to decide whether to
    /// make room, and *no* is the answer they need — the name never takes room
    /// in the bar. Its meaning has not changed; the set of answers has.
    #[must_use]
    pub const fn are_shown(self) -> bool {
        match self {
            Self::Beside => true,
        }
    }

    /// The string this crate declares for this state.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Beside => words::NAMES_BESIDE,
        }
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        // **Nothing to fill.** The sentence is about *where* the name appears,
        // and the only number this type ever carried belonged to `GaveWay` —
        // the text size at which the names left the bar, which is a thing that
        // no longer happens.
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// **A name is always readable**, which is the question a settings panel
    /// and a compositor both ask first.
    ///
    /// This asserted two answers until 2026-10-10 — a placement that drew names
    /// and a state where they had gone. The owner removed the row of names from
    /// the bar, so the second cannot arise, and the honest claim is that there
    /// is no state in which a person cannot get at the name.
    #[test]
    fn a_name_is_always_readable() {
        assert!(Labels::Beside.are_shown());
    }

    /// The one state says something of its own, and a settings panel can show
    /// it.
    ///
    /// **Two sentences stood here**, and they were two rather than one with a
    /// word swapped in because a language that inflects the placement needs the
    /// whole sentence — `alo-egress`' rule about its indicator line, met here.
    /// That rule still holds; there is one sentence to hold it for.
    #[test]
    fn the_placement_says_something_of_its_own() {
        let strings = in_english();
        let said = Labels::Beside.said(&strings);
        assert_eq!(
            said.text(),
            "each icon shows its name beside it when you point at it or reach it by keyboard"
        );
        assert!(said.unfilled().is_empty());
    }

    /// **And it is a translator's sentence rather than a built one.**
    ///
    /// The test this replaces checked that a `{percent}` filling landed where a
    /// German translator put the percent sign. That sentence belonged to
    /// `GaveWay` and went with it; what survives is the rule it was an example
    /// of — the whole sentence is the translator's, and nothing here assembles
    /// one from parts.
    #[test]
    fn the_whole_sentence_is_the_translators() {
        let strings = translated(&[(
            words::NAMES_BESIDE,
            "jedes Symbol zeigt seinen Namen daneben, wenn Sie darauf zeigen oder es mit der \
             Tastatur erreichen",
        )]);
        let said = Labels::Beside.said(&strings);
        assert_eq!(
            said.text(),
            "jedes Symbol zeigt seinen Namen daneben, wenn Sie darauf zeigen oder es mit der \
             Tastatur erreichen"
        );
        assert!(said.is_translated());
        assert!(said.unfilled().is_empty());
    }
}
