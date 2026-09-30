//! What the panel claims as its own, for whoever classifies a pointer.
//!
//! Part of task 6 of `docs/autonomy/putting-a-window-aside.md`. `alo_dock::Revealing` is told
//! *which region the pointer is in* and never where the pointer is — its own header says
//! *three places rather than a coordinate, because then two files would have opinions about
//! the same edge*. This is the panel's half of that: what those places **mean** for this
//! surface, so the caller that owns the coordinates has one thing to classify against.
//!
//! # No coordinates, and therefore no screen size
//!
//! Nothing here is a number. Not the strip's width, not the rail's, not the panel's height.
//! The standing rule is that **nothing is built to one screen size** — a figure taken from a
//! 1440x960 frame becomes a proportion or a named rule, never a constant — and this file
//! satisfies it the way rule 1 is satisfied: **by the value not being reachable**, rather than
//! by a conversion done correctly. There is no figure here to convert.
//!
//! That relocates every measurement into whoever draws, which is the only place that knows
//! the display's size and scale. One conversion, in one crate, instead of two that would
//! agree today.
//!
//! # The edge is data
//!
//! [`WhichEdge`] exists because the panel's edge may not always be the right one. The design
//! holds one mirrored shelf frame at the opposite edge among a hundred and fifty-six, which is
//! as consistent with a stray as with a right-to-left variant, and nobody has established
//! which. **The cost of taking the edge as data is nothing if it is a stray, and the
//! difference between a rename and a rewrite if it is not** — which is the correction
//! `alo-dock`'s own reveal machine already went through, from a module that named the bottom
//! edge everywhere to one that names no edge at all.
//!
//! # One continuous region, because the alternative flickers
//!
//! The previews, the panel's controls, its open menus and **the path between them** are one
//! region. Not three regions with gaps: a pointer crossing from a preview to a control passes
//! over whatever lies between, and a classifier that answered `Elsewhere` there would conceal
//! the panel under a pointer that had not left it. That is the fault `revealing` was built to
//! prevent, arriving through hit-testing instead of through state.
//!
//! A region is **not** confined to the panel's own outline either. The design has a peek
//! region of its own in the middle of the screen, nowhere near any edge, so a menu opening
//! away from the panel is ordinary rather than an exception needing a rule.
//!
//! # What this file will not decide
//!
//! **Which surface wins where two regions overlap.** The owner settled it on 2026-09-30 — the
//! right panel owns the shared corner, the top controls and the Dock stop before the reserved
//! area, and *one pointer position cannot reveal two surfaces*. That is recorded here as a
//! constraint the classifier must satisfy, and **not implemented here as an order**, because a
//! classifier that resolved overlap by the order its branches were written would have decided
//! geometry by accident. Whoever classifies applies the rule once, before any machine is told.

/// Which edge this panel belongs to.
///
/// Data rather than a constant. See this module's header: the panel is at the right edge
/// today, one frame in the design mirrors it, and nothing here needs to know which is true.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhichEdge {
    /// The right edge, which is where the panel lives in a left-to-right arrangement.
    Right,
    /// The left edge, mirrored.
    Left,
}

/// A part of the panel a pointer can be over.
///
/// Every one of these classifies as `alo_dock::ThePointer::OnTheSurface`. They are named
/// separately so a caller can say *which* part it found without inventing a second
/// vocabulary for the answer, and so that adding a part is a change to this list rather than
/// to a predicate somebody has to re-read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartOfThePanel {
    /// A preview's row, or its icon on the collapsed rail.
    APreview,
    /// The panel's own controls — its header, its count, its collapse handle.
    AControl,
    /// A menu belonging to the panel, wherever it opened.
    ///
    /// **Wherever it opened.** A menu is not required to lie inside the panel's outline, and
    /// the design already has a region in the middle of the screen for peeking.
    AMenu,
    /// The space between the parts above, inside the panel's own extent.
    ///
    /// **This is the one that is easy to leave out**, and leaving it out is the flicker: a
    /// pointer travelling from a preview to a control crosses it, and answering `Elsewhere`
    /// here conceals the panel under a pointer that never left it.
    ThePathBetweenThem,
}

/// Where a pointer is, as far as this panel is concerned.
///
/// The caller turns a coordinate into one of these. Nothing else in this crate takes a
/// position, and `alo_dock::Revealing` takes this shape rather than a point for the reason its
/// own header gives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhereThePointerIs {
    /// In the strip along the panel's edge that asks for it.
    InTheStrip,
    /// Over some part of the panel itself.
    OnThePanel(PartOfThePanel),
    /// Somewhere that is neither.
    Elsewhere,
}

impl WhereThePointerIs {
    /// What to tell `alo_dock::Revealing`.
    ///
    /// Returns the three cases that machine knows: in the asking strip, on the surface, or
    /// elsewhere. **Every part of the panel collapses to one answer here**, which is the whole
    /// point — the machine must not be able to tell a preview from the path beside it, or it
    /// would grow an opinion about hit-testing.
    ///
    /// Given as a pair of booleans rather than by naming `alo-dock`'s type, so that this crate
    /// states the contract without a second crate having to agree about it. The caller builds
    /// the variant.
    #[must_use]
    pub const fn holds_the_panel_open(self) -> bool {
        matches!(self, Self::InTheStrip | Self::OnThePanel(_))
    }

    /// Whether this is the asking strip rather than the panel itself.
    ///
    /// The distinction the machine needs: the strip **reveals** from concealed, the surface
    /// only **keeps** what is already revealed. A classifier that reported the panel's own
    /// area as the strip would reveal a concealed panel from a pointer resting where the panel
    /// would be if it were showing.
    #[must_use]
    pub const fn is_the_asking_strip(self) -> bool {
        matches!(self, Self::InTheStrip)
    }

    /// Which part was found, for a caller that wants to say.
    #[must_use]
    pub const fn part(self) -> Option<PartOfThePanel> {
        match self {
            Self::OnThePanel(part) => Some(part),
            Self::InTheStrip | Self::Elsewhere => None,
        }
    }
}

/// The constraint every classifier must satisfy, in the owner's words.
///
/// > Activation strips and pointer paths must follow those bounds too, so **one pointer
/// > position cannot reveal two surfaces.**
///
/// Held here as a statement a test can check rather than as an order this file imposes. A
/// classifier hands its answers for one point to [`at_most_one_surface_claims_it`], and a
/// surface that claims a point another has already claimed is the failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhatEachSurfaceSaid {
    /// Whether the top controls claim this point.
    pub the_top_controls: bool,
    /// Whether the Dock claims it.
    pub the_dock: bool,
    /// Whether this panel claims it.
    pub the_panel: bool,
}

/// Whether at most one surface claims this point.
///
/// **The invariant, and it is deliberately not a resolution.** It reports whether the rule
/// holds; it does not pick a winner, because picking one is what the owner already decided and
/// what a classifier must apply rather than discover. A version of this that returned *which
/// surface wins* would be the priority order this file refuses to contain.
///
/// Zero is allowed: most of the screen belongs to no edge surface, and the middle of a canvas
/// claiming nothing is correct rather than a gap.
#[must_use]
pub const fn at_most_one_surface_claims_it(said: WhatEachSurfaceSaid) -> bool {
    let how_many = said.the_top_controls as u8 + said.the_dock as u8 + said.the_panel as u8;
    how_many <= 1
}
