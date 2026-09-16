//! The closed list, and what each setting changes.

use alo_appearance::{Scheme, TextScale};
use serde::{Deserialize, Serialize};

use crate::high_contrast::HighContrast;
use crate::key_filter::KeyFilter;
use crate::turned_on::{Magnification, TurnedOn};
use crate::words::{self, Word};

/// **Everything a person can turn on for access, and nothing else.**
///
/// Closed, like every other list in this repository that something acts on: a
/// setting that could be any string is a setting nothing can be tested against,
/// and a person is promised that each of these changes one thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Setting {
    /// Read the screen aloud, through the rented screen reader (ADR 0011).
    ScreenReader,
    /// Magnify what is under the pointer, by [`TurnedOn::magnification`].
    Magnifier,
    /// Draw with [`HighContrast`]'s palette rather than the shell's.
    HighContrast,
    /// Draw text larger, as a scale `alo-appearance` already understands.
    LargerText,
    /// Move nothing that does not have to move.
    ReducedMotion,
    /// A modifier held by pressing it, for somebody who cannot hold two keys.
    StickyKeys,
    /// A key counts when it has been held long enough to have been meant.
    SlowKeys,
    /// The same key twice in a moment is one press.
    BounceKeys,
    /// Where the keyboard is, always drawn, not only after a key is pressed.
    FocusAlwaysVisible,
}

impl Setting {
    /// All nine, in the order a person meets them.
    pub const ALL: [Self; 9] = [
        Self::ScreenReader,
        Self::Magnifier,
        Self::HighContrast,
        Self::LargerText,
        Self::ReducedMotion,
        Self::StickyKeys,
        Self::SlowKeys,
        Self::BounceKeys,
        Self::FocusAlwaysVisible,
    ];

    /// The sentence a person reads for this setting.
    #[must_use]
    pub fn word(self) -> Word {
        match self {
            Self::ScreenReader => words::SCREEN_READER,
            Self::Magnifier => words::MAGNIFIER,
            Self::HighContrast => words::HIGH_CONTRAST,
            Self::LargerText => words::LARGER_TEXT,
            Self::ReducedMotion => words::REDUCED_MOTION,
            Self::StickyKeys => words::STICKY_KEYS,
            Self::SlowKeys => words::SLOW_KEYS,
            Self::BounceKeys => words::BOUNCE_KEYS,
            Self::FocusAlwaysVisible => words::FOCUS_ALWAYS_VISIBLE,
        }
    }

    /// **What this setting changes, as the value another crate reads**, given
    /// everything the person has turned on and the scheme being drawn.
    ///
    /// [`None`] where the setting is off, so a caller cannot read a value for a
    /// setting nobody asked for — which is how an *accessibility mode* starts.
    #[must_use]
    pub fn what_it_changes(self, turned_on: &TurnedOn, scheme: Scheme) -> Option<WhatItChanges> {
        if !turned_on.has(self) {
            return None;
        }
        Some(match self {
            Self::ScreenReader => WhatItChanges::TheScreenIsRead,
            Self::Magnifier => {
                WhatItChanges::WhatIsUnderThePointerIsMagnified(turned_on.magnification())
            }
            Self::HighContrast => WhatItChanges::ThePalette(HighContrast::of(scheme)),
            Self::LargerText => WhatItChanges::TheTextScale(turned_on.larger_text()),
            Self::ReducedMotion => WhatItChanges::NothingMovesThatNeedNot,
            Self::StickyKeys | Self::SlowKeys | Self::BounceKeys => {
                WhatItChanges::TheKeyFilter(turned_on.key_filter())
            }
            Self::FocusAlwaysVisible => WhatItChanges::WhereTheKeyboardIsIsAlwaysDrawn,
        })
    }
}

/// **The value a setting changes**, in the type of whoever reads it.
///
/// A setting whose effect were a sentence in a document would be a setting two
/// crates implemented differently; each of these is a value handed over.
#[derive(Debug, Clone, PartialEq)]
pub enum WhatItChanges {
    /// `alo-appearance`'s palette, replaced by [`HighContrast`]'s.
    ThePalette(HighContrast),
    /// `alo-appearance`'s text scale.
    TheTextScale(TextScale),
    /// The keyboard's filter — sticky, slow and bounce keys together, because
    /// one keyboard reads one filter.
    TheKeyFilter(KeyFilter),
    /// What is under the pointer is drawn this many times larger.
    WhatIsUnderThePointerIsMagnified(Magnification),
    /// The rented screen reader runs and reads the tree the agent reads.
    TheScreenIsRead,
    /// Whatever would animate does not.
    NothingMovesThatNeedNot,
    /// Focus is drawn whether or not a key has been pressed.
    WhereTheKeyboardIsIsAlwaysDrawn,
}
