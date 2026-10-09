//! Where alo's external window edge is: the one geometry everything reads.
//!
//! `docs/design/the-external-window-edge.md` is the contract, and every figure
//! below is measured off `Canvas / External window edge` (`398:26305`) rather
//! than read off a picture. The capture is
//! `docs/design/figma-snapshot/398-26305.xml`, authoritative for this
//! component while the page export is stale for it.
//!
//! # One calculation, because the contract asks for one
//!
//! > Use one authoritative geometry calculation for rendering, pointer targets
//! > and reachability.
//!
//! So [`edge_of`] answers all three. A second calculation anywhere — a painter
//! laying out buttons from its own arithmetic, a hit test rounding differently
//! — is the fault that rule names, and it shows up as a control that draws in
//! one place and answers in another.
//!
//! # Outside the content, and the content never moves
//!
//! The edge occupies 44 logical units **above** the window, and the
//! application's content begins where the window begins. So:
//!
//! - nothing of alo's is ever inside the application's own area;
//! - **revealing costs the application nothing.** The region is there whether
//!   the strip is drawn or not, so showing it cannot resize or shift anything
//!   — which is the contract's *application content starts below that region
//!   and stays stationary when the strip reveals*.
//!
//! # What is not decided here
//!
//! **Whether the application draws its own header.** That is read from its
//! protocol state by the host and handed in as [`Decorations`]. The contract
//! is explicit that it is never guessed from a screenshot, a name or how a
//! title bar looks.
//!
//! **When the edge is revealed.** The host owns pointer position, keyboard
//! focus and whether a menu is open or a drag is in progress; `revealed` is
//! its answer, not a question asked here.
//!
//! **The zoom at which controls stop being usable.** The contract says that at
//! overview scales the canvas selection interaction takes over *rather than
//! pretending tiny controls remain usable* — and names no threshold. One is
//! not invented here; [`Edge::how_big_a_target_looks`] gives the apparent size
//! so a host can apply a rule when the owner has set one.

use smithay::utils::{Logical, Point, Rectangle, Size};

/// How tall the region a pointer must be in for the edge to reveal.
///
/// The component is 44 high and every variant of it is (`398:26305`).
pub const THE_REGION_IS_TALL: i32 = 44;

/// How tall the strip that is actually drawn, inside that region.
///
/// `402:26313`, *32px visible edge / 44px interaction*.
pub const THE_STRIP_IS_TALL: i32 = 32;

/// Where the strip starts inside the region: the region's height less the
/// strip's.
///
/// Measured as `y = 12` and derived the same way, so the two cannot disagree.
pub const THE_STRIP_STARTS_AT: i32 = THE_REGION_IS_TALL - THE_STRIP_IS_TALL;

/// How wide and tall one control's target is. Non-overlapping.
pub const A_TARGET_IS: i32 = 44;

/// How wide and tall the artwork inside a control is.
pub const ARTWORK_IS: i32 = 14;

/// The gap between control targets, and between the last and the trailing
/// edge.
///
/// Derived from the measured positions rather than stated: shell-owned
/// controls sit at 456, 504 and 552 in a 600 specimen, so `504 - 456 - 44 = 4`
/// and `600 - (552 + 44) = 4`.
pub const BETWEEN_TARGETS: i32 = 4;

/// How far in from the leading edge the drag region starts. Measured: `x = 8`.
pub const THE_DRAG_STARTS_AT: i32 = 8;

/// The resting grip: 24 wide, 2 tall, centred.
pub const THE_GRIP_IS: (i32, i32) = (24, 2);

/// Where the resting grip sits in the region, measured from `398:26244`: a
/// drag region at `y = 28` holding it at `y = 5`.
const THE_GRIP_SITS_AT: i32 = 33;

/// At rest the drag region is a shallow band, measured from `398:26243`:
/// `y = 28`, 12 tall, holding only the grip.
///
/// **The region a pointer reveals from is still the whole 44.** This is where
/// the grip is drawn and where a press at rest lands, not where the edge
/// notices a pointer.
const THE_DRAG_RESTS_AT: i32 = 28;

/// How tall that resting band is.
const THE_DRAG_RESTS_TALL: i32 = 12;

/// Where the title starts **inside the drag region**, measured from
/// `398:26250`: `x = 30` of a drag region that itself starts at 8, so 38 of
/// the edge.
const THE_TITLE_STARTS_AT: i32 = 30;

/// Where the title's box starts down the region, measured from `398:26250`.
const THE_TITLE_SITS_AT: i32 = 19;

/// Where the revealed grip sits **inside the drag region**, measured from
/// `398:26248`: `x = 8` of a drag region starting at 8, so 16 of the edge.
const THE_GRIP_STARTS_AT: i32 = 8;

/// Who draws a window's header.
///
/// **Read from the application's protocol state and handed in**, never guessed
/// — the contract forbids identifying this from screenshots, application names
/// or how something looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decorations {
    /// The application draws its own header. alo adds movement and a menu, and
    /// **no duplicate window buttons**.
    TheApplicationDraws,
    /// The application leaves it to the shell. alo's edge carries the title,
    /// the movement and the controls — and no second band is drawn inside.
    TheShellDraws,
}

/// What one control on the edge does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnTheEdge {
    /// Put the window aside, through the mechanism that already does it.
    Minimise,
    /// Maximise or restore. **Not full screen** — `docs/features.md` separates
    /// those deliberately, having conflated them once.
    Maximise,
    /// Ask the application to close, with its own unsaved-work handling.
    Close,
    /// The window menu, which is how an application with its own header still
    /// reaches every shell action.
    Menu,
}

/// One control: where it answers, and where its artwork draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Control {
    /// What pressing it does.
    pub does: OnTheEdge,
    /// The 44 × 44 it answers in. Non-overlapping with every other.
    pub target: Rectangle<i32, Logical>,
    /// The 14 × 14 that is drawn, centred in the **visible strip** rather than
    /// in the target — which is why its top is 9 below the strip's and not 15
    /// below the target's.
    pub artwork: Rectangle<i32, Logical>,
}

/// The whole edge of one window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    /// The continuous region a pointer reveals the edge from, and stays
    /// revealed within. Always present, revealed or not.
    pub region: Rectangle<i32, Logical>,
    /// The strip that is drawn, when the edge is revealed.
    pub strip: Option<Rectangle<i32, Logical>>,
    /// Where dragging moves the window. Never under a control.
    pub drag: Rectangle<i32, Logical>,
    /// The grip: 24 × 2 at rest, the 14 × 14 mark when revealed.
    pub grip: Rectangle<i32, Logical>,
    /// Where the window's title is drawn, when the shell draws it and the edge
    /// is revealed.
    pub title: Option<Point<i32, Logical>>,
    /// The controls, in the order they are drawn, left to right.
    pub controls: Vec<Control>,
}

impl Edge {
    /// How large one control's target appears at this zoom, in thousandths.
    ///
    /// For the contract's *at overview scales where targets become too small,
    /// use the canvas selection interaction rather than pretending tiny
    /// controls remain usable*. **No threshold is set here** — the contract
    /// names none and inventing one would be deciding a design. This is the
    /// measurement a host applies a rule to once there is one.
    #[must_use]
    pub fn how_big_a_target_looks(zoom_in_thousandths: u32) -> u32 {
        // `try_from` rather than `as`: a negative target is impossible, and a
        // cast that silently wrapped one would give a target of four billion.
        let target = u32::try_from(A_TARGET_IS).unwrap_or(0);
        target.saturating_mul(zoom_in_thousandths) / 1000
    }
}

/// The edge of this window: where everything on it is.
///
/// `window` is the application's own rectangle — what
/// `crate::where_a_window_is` answers, which is the window and not the
/// surface. The edge is placed **above** it, so the content neither moves nor
/// shrinks.
///
/// `revealed` is the host's answer, from pointer, focus, menu and drag.
#[must_use]
pub fn edge_of(window: Rectangle<i32, Logical>, decorations: Decorations, revealed: bool) -> Edge {
    let region = Rectangle::new(
        Point::from((window.loc.x, window.loc.y - THE_REGION_IS_TALL)),
        Size::from((window.size.w, THE_REGION_IS_TALL)),
    );
    let strip = revealed.then(|| {
        Rectangle::new(
            Point::from((region.loc.x, region.loc.y + THE_STRIP_STARTS_AT)),
            Size::from((region.size.w, THE_STRIP_IS_TALL)),
        )
    });

    // **Controls at the trailing edge, counted from the right.** The design
    // positions them absolutely in a 600 specimen; deriving them from the
    // trailing edge instead is what makes the edge fit any window, which the
    // contract asks for in as many words — *600 is a specimen width, not a
    // fixed window width*.
    // At rest nothing is drawn to press, so nothing answers: the rest
    // variants hold no control node at all.
    let does = if !revealed {
        Vec::new()
    } else {
        match decorations {
            Decorations::TheShellDraws => {
                vec![OnTheEdge::Minimise, OnTheEdge::Maximise, OnTheEdge::Close]
            }
            // **No duplicate window buttons**, by the owner's ruling: an
            // application with its own header keeps it, and alo adds only movement
            // and the menu.
            Decorations::TheApplicationDraws => vec![OnTheEdge::Menu],
        }
    };
    let how_many = i32::try_from(does.len()).unwrap_or(0);
    let controls_take = how_many * (A_TARGET_IS + BETWEEN_TARGETS);
    let first_target_at =
        region.loc.x + region.size.w - BETWEEN_TARGETS - controls_take + BETWEEN_TARGETS;

    let controls = does
        .into_iter()
        .enumerate()
        .map(|(which, does)| {
            let at = first_target_at
                + i32::try_from(which).unwrap_or(0) * (A_TARGET_IS + BETWEEN_TARGETS);
            let target = Rectangle::new(
                Point::from((at, region.loc.y)),
                Size::from((A_TARGET_IS, A_TARGET_IS)),
            );
            Control {
                does,
                target,
                artwork: artwork_in(target, region),
            }
        })
        .collect();

    let drag_from = region.loc.x + THE_DRAG_STARTS_AT;
    let drag = if revealed {
        // Revealed, dragging runs from the leading inset to one gap before the
        // first control, and takes the region's whole height so a pointer
        // travelling along it never leaves the reveal.
        Rectangle::new(
            Point::from((drag_from, region.loc.y)),
            Size::from((
                (first_target_at - BETWEEN_TARGETS - drag_from).max(0),
                THE_REGION_IS_TALL,
            )),
        )
    } else {
        // **At rest the whole edge drags**, because no control is shown to
        // take any of it — `398:26243` is 588 of a 600 edge, which is the
        // leading 8 and the trailing 4 and nothing else. A shallow band, where
        // the grip is drawn.
        Rectangle::new(
            Point::from((drag_from, region.loc.y + THE_DRAG_RESTS_AT)),
            Size::from((
                (region.size.w - THE_DRAG_STARTS_AT - BETWEEN_TARGETS).max(0),
                THE_DRAG_RESTS_TALL,
            )),
        )
    };

    let grip = if revealed {
        // The 14 × 14 mark, inside the drag region, centred in the strip like
        // every other piece of artwork.
        Rectangle::new(
            Point::from((
                drag.loc.x + THE_GRIP_STARTS_AT,
                region.loc.y + THE_STRIP_STARTS_AT + (THE_STRIP_IS_TALL - ARTWORK_IS) / 2,
            )),
            Size::from((ARTWORK_IS, ARTWORK_IS)),
        )
    } else {
        // **Centred in the drag region, not in the edge.** Those differ by two
        // units, because the insets are 8 and 4 — and being wrong by two is
        // exactly the kind of thing nobody sees and the design does.
        Rectangle::new(
            Point::from((
                drag.loc.x + (drag.size.w - THE_GRIP_IS.0) / 2,
                region.loc.y + THE_GRIP_SITS_AT,
            )),
            Size::from(THE_GRIP_IS),
        )
    };

    // Inside the drag region, like the grip.
    let title = (revealed && decorations == Decorations::TheShellDraws).then(|| {
        Point::from((
            drag.loc.x + THE_TITLE_STARTS_AT,
            region.loc.y + THE_TITLE_SITS_AT,
        ))
    });

    Edge {
        region,
        strip,
        drag,
        grip,
        title,
        controls,
    }
}

/// A control's artwork: centred across its target, centred down the **visible
/// strip**.
fn artwork_in(
    target: Rectangle<i32, Logical>,
    region: Rectangle<i32, Logical>,
) -> Rectangle<i32, Logical> {
    Rectangle::new(
        Point::from((
            target.loc.x + (A_TARGET_IS - ARTWORK_IS) / 2,
            region.loc.y + THE_STRIP_STARTS_AT + (THE_STRIP_IS_TALL - ARTWORK_IS) / 2,
        )),
        Size::from((ARTWORK_IS, ARTWORK_IS)),
    )
}

#[cfg(test)]
#[path = "window_edge_tests.rs"]
mod tests;
