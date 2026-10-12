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
//! not invented here; [`WindowEdge::how_big_a_target_looks`] gives the apparent size
//! so a host can apply a rule when the owner has set one.

use smithay::utils::{Logical, Physical, Point, Rectangle, Size};

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

/// The highlight drawn behind a hovered control: its size, and its offset
/// inside the 44 x 44 target.
///
/// **Smaller than the target, measured from `402:26315`** — 32 x 28 at
/// `left 6, top 14`, `bg/cool`. The target is what answers a press; this is
/// what is drawn. Filling the whole 44 would be visibly wrong and is what the
/// contract's *subtle highlight only on the hovered control* rules out.
pub const A_HIGHLIGHT_IS: (i32, i32) = (32, 28);

/// Where that highlight sits inside its target.
const THE_HIGHLIGHT_SITS_AT: (i32, i32) = (6, 14);

/// How much the edge insets its outermost targets, measured `414:27583` and
/// `414:27626`: both put the leading target at `x = 4`, and `space/1` is 4.
///
/// **This is 4 and `THE_DRAG_STARTS_AT` below is 8**, which is not a
/// contradiction but a design change: the older capture `398:26243` put the
/// drag region's leading inset at 8 when nothing was drawn there. The compact
/// section of 2026-10-11 puts a 44 × 44 *Move* target at 4, and that target is
/// now what the leading inset is measured to.
pub const THE_PADDING_IS: i32 = 4;

/// The narrowest edge that holds all three window actions and a drag target.
///
/// Measured from the design's own arithmetic, which it states: *44 drag +
/// 3×44 actions + 3×4 gaps + 2×4 padding* — 44 + 132 + 12 + 8 = **196**. The
/// `State=Full controls` master is 196 wide, which is the same number drawn.
///
/// **Not 144.** This lane found controls laid out outside the region below 144
/// and the design answers it in as many words: *the previous 144px fault is a
/// reproduction boundary, not evidence that all full controls fit at 144.*
pub const FULL_CONTROLS_NEED: i32 = 196;

/// The narrowest edge there is, below which the edge stops shrinking.
///
/// Three 44 targets, two 4 gaps and 4 padding at each end: 132 + 8 + 8 =
/// **148**, and the four compact masters are each 148 wide. An edge is never
/// narrower than this; a *window* may be, and then the edge overhangs it.
pub const THE_COMPACT_EDGE_IS: i32 = 148;

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
    /// Take hold of the window and move it.
    ///
    /// **A control rather than a bare region**, which the compact design
    /// settled: `414:27583` and `414:27626` both draw *Move window / 44×44*
    /// with a 14px glyph, at the leading end of the edge. It is the same
    /// affordance the grip was, given a target so that a pointer, a finger and
    /// a keyboard can all find it in the same place.
    ///
    /// Its drawn mark is `crate::window_edge::WindowEdge::grip` and not an
    /// artwork of its own, so that one rectangle is drawn once.
    Move,
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
pub struct EdgeControl {
    /// What pressing it does.
    pub does: OnTheEdge,
    /// The 44 × 44 it answers in. Non-overlapping with every other.
    pub target: Rectangle<i32, Logical>,
    /// The 14 × 14 that is drawn, centred in the **visible strip** rather than
    /// in the target — which is why its top is 9 below the strip's and not 15
    /// below the target's.
    pub artwork: Rectangle<i32, Logical>,
    /// What is filled behind this control while it is hovered: 32 × 28 inside
    /// the 44 × 44, never the whole target.
    pub highlight: Rectangle<i32, Logical>,
}

/// The whole edge of one window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowEdge {
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
    pub controls: Vec<EdgeControl>,
}

impl WindowEdge {
    /// How much width the title has, from where it starts to the first action.
    ///
    /// **Derived here rather than computed by each caller.** It used to be
    /// `drag.size.w - THE_GRIP_IS.0` wherever it was needed, which was right
    /// while the title began at the drag region's leading edge. The compact
    /// design puts a 44 × 44 Move target there and the title starts after it,
    /// so that expression became an over-estimate — measured on a 320-wide
    /// window, where it let a title run to 225 against a first action at 176.
    ///
    /// Zero when there is no title, and zero rather than negative when the
    /// controls leave nothing: a narrow window has no room for a title at all,
    /// and a negative width would be a subtraction nobody checked.
    #[must_use]
    pub fn the_room_a_title_has(&self) -> i32 {
        let Some(title) = self.title else {
            return 0;
        };
        let first = self
            .controls
            .iter()
            .filter(|it| it.does != OnTheEdge::Move)
            .map(|it| it.target.loc.x)
            .min();
        let until = first.map_or_else(
            || self.region.loc.x + self.region.size.w - THE_PADDING_IS,
            |at| at - BETWEEN_TARGETS,
        );
        (until - title.x).max(0)
    }

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

/// The output the edge is clamped inside, in the units the edge is laid out
/// in.
///
/// **One conversion, in one place.** The edge is laid out in logical units and
/// every backend knows its framebuffer in physical pixels, so without this
/// each caller would divide by the scale itself and they would disagree the
/// first time one of them was wrong.
///
/// `scale` is in hundredths, as `EdgePicture::of` takes it — 100 is
/// one-to-one, 200 is a doubled display. **Today every caller passes 100**,
/// and that is a known gap rather than a simplification here: the real
/// per-display scale has no source yet, which `crate::the_pointer_in_pixels`
/// records at length for the panel's sake. When it arrives this function is
/// where it lands.
///
/// A scale below one-to-one is read as one-to-one: 0 would divide by zero, and
/// a display reporting a scale under 100 hundredths is not a thing this
/// project has. Refusing here would mean an edge that cannot be laid out at
/// all, which is worse than treating an impossible scale as the ordinary one.
#[must_use]
pub fn the_usable_output(pixels: Size<i32, Physical>, scale_hundredths: u32) -> Size<i32, Logical> {
    // **100 and not 1.** A scale of nothing means one-to-one, and one-to-one
    // is a hundred hundredths - `max(1)` guarded the division and multiplied
    // every dimension by a hundred, which the scale test caught.
    let hundredths = i64::from(scale_hundredths.max(100));
    let logical = |it: i32| i32::try_from(i64::from(it) * 100 / hundredths).unwrap_or(i32::MAX);
    Size::from((logical(pixels.w), logical(pixels.h)))
}

/// Which layout a window of this width gets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HowItFits {
    /// Minimise, Maximise and Close, each its own target, from
    /// [`FULL_CONTROLS_NEED`] upward.
    Full,
    /// Move, Menu and Close, below that. The actions Full shows directly are
    /// reached through the menu, so **nothing is dropped** — which is the
    /// whole reason this layout exists rather than a shorter list of controls.
    Compact,
}

impl HowItFits {
    /// Which layout a content width of `wide` gets.
    #[must_use]
    pub const fn at(wide: i32) -> Self {
        if wide >= FULL_CONTROLS_NEED {
            Self::Full
        } else {
            Self::Compact
        }
    }
}

/// Where the edge's own rectangle is, which is **not** the window's.
///
/// Three cases, and the third is the one the design was redrawn for:
///
/// | content width | the edge |
/// |---|---|
/// | [`THE_COMPACT_EDGE_IS`] or wider | exactly the window's width and position |
/// | narrower | [`THE_COMPACT_EDGE_IS`] wide, centred over the window, then clamped into the output |
///
/// **The application's content is never moved or resized to make the edge
/// fit**, which the contract says in as many words. So below 148 the edge is
/// wider than the window it belongs to and hangs over on both sides, and that
/// overhang is part of the edge: `crate::window_edge_pointing` bounds a press
/// by this rectangle, so a press on the overhang reaches the control it looks
/// like it should.
///
/// **Clamped last, after centring**, so an edge over a window at the very left
/// of a display slides right rather than off. When the output itself is
/// narrower than 148 there is nowhere to clamp to and the edge starts at 0,
/// overhanging the right — recorded rather than refused, because a display
/// narrower than 148 logical units is not a case this project has.
fn where_the_edge_goes(
    window: Rectangle<i32, Logical>,
    output: Size<i32, Logical>,
) -> Rectangle<i32, Logical> {
    let wide = window.size.w.max(THE_COMPACT_EDGE_IS);
    let at = if window.size.w >= THE_COMPACT_EDGE_IS {
        window.loc.x
    } else {
        // Centred over the window: half the difference to the left of it.
        // Integer division truncates toward zero, which puts the odd unit on
        // the trailing side — stated because a reader checking 143 against the
        // design will find 2 on the left and 3 on the right.
        let centred = window.loc.x - (THE_COMPACT_EDGE_IS - window.size.w) / 2;
        let furthest = (output.w - THE_COMPACT_EDGE_IS).max(0);
        centred.clamp(0, furthest)
    };
    Rectangle::new(
        Point::from((at, window.loc.y - THE_REGION_IS_TALL)),
        Size::from((wide, THE_REGION_IS_TALL)),
    )
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
pub fn edge_of(
    window: Rectangle<i32, Logical>,
    decorations: Decorations,
    revealed: bool,
    output: Size<i32, Logical>,
) -> WindowEdge {
    let region = where_the_edge_goes(window, output);
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
    //
    // **Which controls a window has is asked of `window_edge_reading`**, which
    // is one list rather than two that could drift. It lives there because
    // whether a control exists is an accessibility question before it is a
    // drawing one - a reader reaches every one of them while this edge is
    // concealed and draws none - and that file includes the owner's ruling
    // that an application with its own header keeps its buttons, so alo adds
    // only movement and the menu.
    let fits = HowItFits::at(window.size.w);
    let does = if revealed {
        crate::window_edge_reading::the_controls_of(decorations, fits)
    } else {
        Vec::new()
    };

    // **The actions are anchored to the trailing edge and Move to the leading
    // one**, which is what both masters draw. At 196 and at 148 — each the
    // exact minimum for its layout — the two anchors meet: the trailing walk
    // lands the first action one gap after a 44 target at `x = 4`, and that
    // target is Move. Above the minimum the two pull apart and the slack
    // between them is drag.
    //
    // Derived from the trailing edge rather than from absolute positions, as
    // before, because *600 is a specimen width, not a fixed window width* —
    // and now neither is 196.
    let actions = does
        .len()
        .saturating_sub(usize::from(does.contains(&OnTheEdge::Move)));
    let how_many = i32::try_from(actions).unwrap_or(0);
    let last_target_at = region.loc.x + region.size.w - THE_PADDING_IS - A_TARGET_IS;
    let first_target_at = last_target_at - (how_many - 1).max(0) * (A_TARGET_IS + BETWEEN_TARGETS);

    let mut action = 0_i32;
    let controls: Vec<EdgeControl> = does
        .into_iter()
        .map(|does| {
            // Move sits at the leading padding and takes no place in the
            // trailing walk; every other control steps along it in order.
            let at = if does == OnTheEdge::Move {
                region.loc.x + THE_PADDING_IS
            } else {
                let at = first_target_at + action * (A_TARGET_IS + BETWEEN_TARGETS);
                action += 1;
                at
            };
            let target = Rectangle::new(
                Point::from((at, region.loc.y)),
                Size::from((A_TARGET_IS, A_TARGET_IS)),
            );
            EdgeControl {
                does,
                target,
                artwork: artwork_in(target, region),
                highlight: Rectangle::new(
                    Point::from((
                        target.loc.x + THE_HIGHLIGHT_SITS_AT.0,
                        target.loc.y + THE_HIGHLIGHT_SITS_AT.1,
                    )),
                    Size::from(A_HIGHLIGHT_IS),
                ),
            }
        })
        .collect();

    let drag = if revealed {
        // **Dragging begins at the Move target and runs to one gap before the
        // first action**, so the drag area is never smaller than that target:
        // 44 × 44 at the minimum width, which is the contract's *the available
        // drag area is at least 44×44*, and wider as the window is.
        //
        // From `THE_PADDING_IS` and not `THE_DRAG_STARTS_AT`: the leading
        // inset is 4 in the compact section, where a target is drawn, and was
        // 8 in the older capture where nothing was.
        //
        // The region's whole height, so a pointer travelling along it never
        // leaves the reveal.
        let from = region.loc.x + THE_PADDING_IS;
        Rectangle::new(
            Point::from((from, region.loc.y)),
            Size::from((
                (first_target_at - BETWEEN_TARGETS - from).max(A_TARGET_IS),
                THE_REGION_IS_TALL,
            )),
        )
    } else {
        let drag_from = region.loc.x + THE_DRAG_STARTS_AT;
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
        // **Move's own artwork, not a second computation of it.** This read
        // `drag.loc.x + THE_GRIP_STARTS_AT` and gave 12 where Move's artwork
        // gave 19, because the drag's leading inset moved from 8 to 4 and the
        // grip's offset was measured against the old one. Two rectangles for
        // one mark is how they come to disagree, and they did.
        //
        // The design draws exactly one: *Move window / 14px glyph* at `x = 15`
        // inside a 44 target at `x = 4`, which is where `artwork_in` puts it.
        controls
            .iter()
            .find(|it| it.does == OnTheEdge::Move)
            .map_or_else(
                || {
                    // No Move laid out means no mark. Reachable only if the
                    // control list ever stops carrying it, which every width
                    // test asserts it does not.
                    Rectangle::new(
                        Point::from((drag.loc.x, region.loc.y + THE_STRIP_STARTS_AT)),
                        Size::from((0, 0)),
                    )
                },
                |it| it.artwork,
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

    // **After the Move target, not at the drag region's left edge.** The title
    // used to start at the drag's leading inset, which had nothing in it. The
    // compact design puts a 44 × 44 Move target there, so a title starting at
    // the same place would be drawn over its glyph — found by
    // `a_fitted_title_never_reaches_the_controls`, which reported the title
    // ending at 177 and *the first control* starting at 4.
    //
    // Only where the shell draws the decorations, as before: an application
    // with its own header has its own title.
    let title = (revealed && decorations == Decorations::TheShellDraws).then(|| {
        Point::from((
            drag.loc.x + A_TARGET_IS + BETWEEN_TARGETS + THE_TITLE_STARTS_AT,
            region.loc.y + THE_TITLE_SITS_AT,
        ))
    });

    WindowEdge {
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

#[cfg(test)]
#[path = "window_edge_breakpoints_tests.rs"]
mod breakpoints_tests;
