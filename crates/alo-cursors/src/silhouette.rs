//! **Which arrow the pointer is**, in one place.
//!
//! The owner chose on 2026-11-06, in one word: *"B"*. So the precision kite is
//! what a person sees, and the familiar arrow is kept beside it as **Classic**
//! for anybody who prefers it. Both are on the same 32-unit box as every other
//! shape in this crate, with the hotspot at the tip.
//!
//! **Choosing between them is one value**, [`Silhouette`], handed to
//! [`crate::draw`]. Nothing in Settings offers the choice yet; when it does, the
//! setting is that value and nothing else in this crate changes.

/// One of the two arrows a person's pointer can be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Silhouette {
    /// The precision kite, chosen by the owner on 2026-11-06.
    #[default]
    Kite,
    /// The refined arrow everybody already knows, kept as a choice.
    Classic,
}

impl Silhouette {
    /// Both, the shipped one first.
    pub const ALL: [Self; 2] = [Self::Kite, Self::Classic];

    /// The outline as SVG path data on the 32-unit box.
    #[must_use]
    pub const fn path(self) -> &'static str {
        match self {
            Self::Kite => {
                "M6 5 L25 12.6 Q26.6 13.3 25 14.1 L16.6 17.1 L13.6 25.5 Q12.9 27.1 12.2 25.5 Z"
            }
            Self::Classic => "M6 4 L6 24 L11.2 19.3 L14.9 27.2 L18.4 25.7 L14.8 17.9 L21.8 17.9 Z",
        }
    }

    /// The tip, which is the hotspot: the point that clicks.
    ///
    /// The first point of [`Self::path`], and a test holds the two together.
    #[must_use]
    pub const fn tip(self) -> (f32, f32) {
        match self {
            Self::Kite => (6.0, 5.0),
            Self::Classic => (6.0, 4.0),
        }
    }
}
