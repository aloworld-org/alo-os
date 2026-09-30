//! What the panel shows, which is not the same as what the person chose.
//!
//! # Two things, because a choice and a state are different
//!
//! A person chooses between **expanded** and **collapsed**, and that choice is theirs
//! and persists. What is drawn has a **third** case: a panel holding nothing shows a
//! small handle at the edge rather than an empty column, because an empty column is a
//! thing a person has to look at and learn to ignore.
//!
//! **The third case is not a choice and must not be stored as one.** If it were, a
//! panel could be *chosen empty* while holding three windows, which is a state with no
//! meaning that something would eventually have to decide what to do about. So
//! [`Chosen`] has two cases, [`HowItShows`] has three, and the third is computed from
//! whether anything is there.
//!
//! # Why a count is not enough to answer this
//!
//! It is tempting to let the drawing side ask *how many previews* and treat zero as the
//! handle. That is the same fault this repository has spent a week finding: **a zero
//! cannot distinguish its two cases.** *The panel is holding nothing* and *there is no
//! panel here* are different things, and a length is the same number for both — so the
//! empty case is named rather than inferred, and a surface that has no panel at all
//! never produces a [`HowItShows`] to begin with.

/// What the person chose, and what persists.
///
/// **Two cases, because a person can only pick between two things.** Collapsing is a
/// change to the panel and never to the windows — it is a presentation, and the test
/// that matters is that nothing else moves when it changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Chosen {
    /// Named previews, one per window. What a panel does by default.
    #[default]
    Expanded,
    /// A slim rail of window-specific icons.
    ///
    /// **Still one icon per window**, not per application: collapsing changes how much
    /// is shown about each window, never how many there are. A rail that merged three
    /// Browser windows into one icon would be the Dock, which is the duplicate the
    /// owner refused.
    Collapsed,
}

/// What is actually drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HowItShows {
    /// Small named previews.
    NamedPreviews,
    /// A slim rail of icons, one per window.
    ARailOfIcons,
    /// A small handle at the edge, because nothing is put aside.
    ///
    /// **Not an empty list drawn narrow.** The person put nothing away, so there is
    /// nothing to look at, and the handle is what says the panel exists and is empty
    /// rather than absent.
    AnEdgeHandle,
}

impl HowItShows {
    /// What a panel holding this many previews, chosen this way, shows.
    ///
    /// **Emptiness wins over the choice, and does not overwrite it.** A person who
    /// collapsed the panel and then restored their last window sees the handle; when
    /// they put something aside again they get the rail back, because the choice was
    /// never changed — only what could be drawn with it.
    #[must_use]
    pub const fn of(holding: usize, chosen: Chosen) -> Self {
        if holding == 0 {
            return Self::AnEdgeHandle;
        }
        match chosen {
            Chosen::Expanded => Self::NamedPreviews,
            Chosen::Collapsed => Self::ARailOfIcons,
        }
    }
}
