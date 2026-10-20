//! The eight keyboard layouts the install environment offers.
//!
//! Eight, decided 2026-10-20 and recorded in
//! `docs/autonomy/the-environment-in-full-plan.md` §3b: enough to cover the EU
//! languages this product promises with the fewest tables, plus the two that
//! differ most from each other. A ninth card, *Something else*, is deliberately
//! undrawn until anybody is known to reach it.
//!
//! **The cost of a layout is not the layout.** It is the generator and the
//! engine behind it, and once those exist the marginal cost of all 138 layouts
//! `xkbcommon` carries over these eight is close to nothing. Eight is taken
//! anyway, and the reason is not size: these are eight somebody can check by
//! typing on them, where 138 would be 130 tables nothing had ever verified.

/// A keyboard layout the install environment offers.
///
/// The names are the ones `xkbcommon` knows these by, which is what the
/// generator passes to it, so a variant here and a table row cannot drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TheLayout {
    /// Belgian — AZERTY, and the one this repository's owner types on.
    Be,
    /// French — AZERTY, and the layout `be` is mostly an override of.
    Fr,
    /// German — QWERTZ.
    De,
    /// Spanish.
    Es,
    /// Italian.
    It,
    /// Dutch.
    Nl,
    /// British English.
    Gb,
    /// United States English — the one with no third level at all.
    Us,
}

impl TheLayout {
    /// Every layout the environment offers, in the order the screen shows them.
    pub const ALL: [Self; 8] = [
        Self::Be,
        Self::Fr,
        Self::De,
        Self::Es,
        Self::It,
        Self::Nl,
        Self::Gb,
        Self::Us,
    ];

    /// What `xkbcommon` knows this layout by.
    ///
    /// This is what the generator passes as `--layout`, and what the table's
    /// rows are keyed by. A name that `xkbcommon` does not recognise makes the
    /// generator fail rather than quietly produce a table of the default
    /// layout, which is the fault
    /// [`crate::the_table_xkbcommon_gave`] exists to have already caught.
    #[must_use]
    pub const fn as_xkb_knows_it(self) -> &'static str {
        match self {
            Self::Be => "be",
            Self::Fr => "fr",
            Self::De => "de",
            Self::Es => "es",
            Self::It => "it",
            Self::Nl => "nl",
            Self::Gb => "gb",
            Self::Us => "us",
        }
    }

    /// The layout `xkbcommon` calls by this name, where this crate offers it.
    ///
    /// `None` for any other name, including the 130 layouts `xkbcommon` carries
    /// that no table here covers — a person who needs one of those is behind
    /// the *Something else* card, and is told so rather than given `us`.
    #[must_use]
    pub fn the_one_called(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|layout| layout.as_xkb_knows_it() == name)
    }

    /// Which number the generated table files this layout's rows under.
    ///
    /// Public because the generator writes the table and its test reads it
    /// back, and both of those live outside this crate. It is the table's own
    /// numbering and means nothing else — a caller that wants a name wants
    /// [`Self::as_xkb_knows_it`].
    #[must_use]
    pub const fn as_the_table_numbers_it(self) -> u8 {
        match self {
            Self::Be => 0,
            Self::Fr => 1,
            Self::De => 2,
            Self::Es => 3,
            Self::It => 4,
            Self::Nl => 5,
            Self::Gb => 6,
            Self::Us => 7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TheLayout;

    #[test]
    fn every_layout_has_a_name_xkb_would_recognise() {
        for layout in TheLayout::ALL {
            let name = layout.as_xkb_knows_it();
            assert!(
                name.len() == 2 && name.bytes().all(|byte| byte.is_ascii_lowercase()),
                "{layout:?} is named {name:?}, which is not the shape of an xkb layout name"
            );
        }
    }

    #[test]
    fn a_name_round_trips_and_an_unoffered_one_is_refused() {
        for layout in TheLayout::ALL {
            assert_eq!(
                TheLayout::the_one_called(layout.as_xkb_knows_it()),
                Some(layout)
            );
        }
        // `ch` is a real xkb layout this crate does not carry. It must come
        // back as nothing rather than as the nearest thing.
        assert_eq!(TheLayout::the_one_called("ch"), None);
        assert_eq!(TheLayout::the_one_called(""), None);
        assert_eq!(TheLayout::the_one_called("US"), None);
    }

    #[test]
    fn the_table_numbers_them_once_each() {
        let mut seen = [false; 8];
        for layout in TheLayout::ALL {
            let at = usize::from(layout.as_the_table_numbers_it());
            assert!(!seen[at], "two layouts are numbered {at}");
            seen[at] = true;
        }
        assert!(seen.into_iter().all(|one| one), "a number is unused");
    }
}
