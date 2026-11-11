//! Where a window's band is: the one geometry everything reads.
//!
//! `docs/design/the-window-controls.md` is the contract — the owner's
//! specification of 2026-11-09 — and every figure below is measured off the
//! design file `XbSmsBYix0TrcQ0Qzaup64`, frames `11:2` (at rest) and `11:24`
//! (revealed). The node trees are in
//! `docs/design/figma-snapshot/window-controls-XbSmsBYix0TrcQ0Qzaup64.xml`.
//!
//! # One calculation, because the contract asks for one
//!
//! [`edge_of`] answers drawing, pointer targets and reachability. A second
//! calculation anywhere — a painter laying out buttons from its own arithmetic,
//! a hit test rounding differently — is how a control comes to draw in one
//! place and answer in another.
//!
//! # Outside the content, and the content never moves
//!
//! The band occupies [`THE_REGION_IS_TALL`] logical units **above** the window,
//! and the application's content begins where the window begins. Nothing of
//! alo's is inside the application's own area, and revealing the controls costs
//! the application nothing: the band is there either way.
//!
//! # What changed on 2026-11-09
//!
//! The band was a 44-tall region with a 32-tall strip, four controls (Move,
//! Minimise, Maximise, Close), an application icon, and a compact layout below
//! 196 units that moved two of them into a menu. The owner's specification
//! replaced all of it: **the title and the application's name are always on the
//! band**, and **exactly three controls** — Window actions, Minimise, Close —
//! appear as the pointer approaches. Maximise, Left, Right and Full screen are
//! in the window panel Window actions opens (`crate::window_menu`). With three
//! controls at every width there is no second layout, so there is no
//! breakpoint either.
//!
//! # What is not decided here
//!
//! **Whether the application draws its own header.** Read from its protocol
//! state by the host and handed in as [`Decorations`], never guessed.
//!
//! **Whether the controls are shown.** The host owns pointer position, keyboard
//! focus and whether the panel is open; `revealed` is its answer.

use smithay::utils::{Logical, Physical, Point, Rectangle, Size};

/// How tall the band is: `11:31`, *Persistent title · only free-move handle*,
/// 54 tall in every frame of the specification.
pub const THE_REGION_IS_TALL: i32 = 54;

/// How tall what is drawn of the band is. **All of it**: the title is on the
/// band at all times, so there is no resting state with nothing drawn.
pub const THE_STRIP_IS_TALL: i32 = THE_REGION_IS_TALL;

/// Where the drawn band starts inside the region: at its top.
pub const THE_STRIP_STARTS_AT: i32 = THE_REGION_IS_TALL - THE_STRIP_IS_TALL;

/// How wide and tall one control's target is. `11:42`, `11:45`, `11:49`.
pub const A_TARGET_IS: i32 = 44;

/// Where the controls sit down the band: `11:41` at `y = 5`, which is the
/// band's height less a target's, halved — so they are centred, and derived
/// that way so the two cannot disagree.
pub const THE_CONTROLS_SIT_AT: i32 = (THE_REGION_IS_TALL - A_TARGET_IS) / 2;

/// The gap between control targets: `11:41` lays its three out with a gap of 4.
pub const BETWEEN_TARGETS: i32 = 4;

/// How far the last control ends from the band's trailing edge: `11:41` is 140
/// wide at `x = 898` in a 1056 band, so `1056 − (898 + 140) = 18`.
pub const THE_TRAILING_INSET: i32 = 18;

/// Where the title starts across the band: `11:32` at `x = 22`.
pub const THE_TITLE_STARTS_AT: i32 = 22;

/// Where the title's box starts down the band: `11:32` at `y = 18`.
pub const THE_TITLE_SITS_AT: i32 = 18;

/// How much room is left between the end of the title and the application's
/// name: `11:33` at `x = 160` after a title 114 wide at 22, so `160 − 136`.
pub const BEFORE_THE_APPLICATIONS_NAME: i32 = 24;

/// How far below the title's box the application's name sits: `11:33` at
/// `y = 20` against the title's 18 — the smaller type set on the same line.
pub const THE_APPLICATIONS_NAME_DROPS: i32 = 2;

/// The narrowest band there is, below which it stops shrinking: the title's
/// leading inset, the three controls with their two gaps, and the trailing
/// inset — `22 + 3 × 44 + 2 × 4 + 18 = 180`. A *window* may be narrower; then
/// the band overhangs it, centred, and the content is never resized to fit.
pub const THE_NARROWEST_EDGE_IS: i32 =
    THE_TITLE_STARTS_AT + 3 * A_TARGET_IS + 2 * BETWEEN_TARGETS + THE_TRAILING_INSET;

/// Who draws a window's header.
///
/// **Read from the application's protocol state and handed in**, never guessed
/// — the contract forbids identifying this from screenshots, application names
/// or how something looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decorations {
    /// The application draws its own header, with the window buttons it
    /// shows itself. alo's band adds **Window actions, and whichever of
    /// Minimise and Close the application does not show** — the owner,
    /// 2026-11-12: *No need to add controls if the app has, we can only add
    /// the ones that are not there.* Its own buttons stay.
    TheApplicationDraws(ItsOwnButtons),
    /// The application leaves it to the shell: the band carries the title, the
    /// application's name and all three controls.
    TheShellDraws,
    /// A task window, which is alo's own and carries its title inside it.
    ///
    /// **Minimise and Close**: a task window does not move (the owner, 2026-10-28),
    /// and minimising puts the task in the right-hand panel, still running, now
    /// that the panel holds tasks (design `23:2`, approved 2026-11-11).
    /// `crate::a_task_windows_edge` draws and presses it.
    ATaskWindow,
    /// One of alo's own windows that is an application in its own right — the
    /// terminal first (ADR 0145): **its title, and Close**. Nothing can put one
    /// of alo's windows aside or open a panel for it yet, and a control that
    /// reaches nothing looks live and is not. `crate::the_terminals_edge` draws
    /// and presses it.
    AlosOwnWindow,
}

/// Which of the band's controls an application that draws its own header
/// shows itself.
///
/// **Never read off the application's pixels or its accessibility tree**:
/// `crate::window_edge_who_draws` answers it from what the application
/// declared and a reviewed table of what each was measured to draw, and where
/// neither says, [`Self::NOT_KNOWN`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItsOwnButtons {
    /// The application draws a Minimise button of its own.
    pub minimise: bool,
    /// The application draws a Close button of its own.
    pub close: bool,
}

impl ItsOwnButtons {
    /// What alo assumes where it cannot know: that the application shows both,
    /// so alo adds **Window actions only** and never a duplicate. The cost — a
    /// window that in fact has neither reaches Minimise and Close through the
    /// keyboard alone — is named in
    /// `docs/quirks/an-application-that-draws-its-own-header-is-not-asked-which-buttons.md`.
    pub const NOT_KNOWN: Self = Self {
        minimise: true,
        close: true,
    };
}

/// What one control on the band does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnTheEdge {
    /// Open or close the window panel: placement, full screen and what comes
    /// after them. The layout mark and no word, tooltip and name *Window
    /// actions*.
    WindowActions,
    /// Put the window aside, through the mechanism that already does it.
    Minimise,
    /// Ask the application to close, with its own unsaved-work handling.
    Close,
}

/// One control: where it answers and is drawn.
///
/// **The target is the artwork's box.** The design exports each mark as a
/// 44-unit SVG the size of its target, with the mark at its own place inside
/// it (`crates/alo-shell/icons/the-window-controls/`), so there is no second
/// rectangle to keep in step with this one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeControl {
    /// What pressing it does.
    pub does: OnTheEdge,
    /// The 44 × 44 it answers in and is drawn in. Non-overlapping with every
    /// other.
    pub target: Rectangle<i32, Logical>,
}

/// The whole band of one window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowEdge {
    /// The band: where a pointer reveals the controls from, and keeps them
    /// revealed within.
    pub region: Rectangle<i32, Logical>,
    /// What is drawn of the band. The whole of it, always.
    pub strip: Rectangle<i32, Logical>,
    /// Where dragging moves the window. The band less the controls that are
    /// shown — so a press where a hidden control would be is a press on the
    /// band, because a control nobody can see is never pressed.
    pub drag: Rectangle<i32, Logical>,
    /// Where the title starts, on a window whose band carries one.
    pub title: Option<Point<i32, Logical>>,
    /// How wide the title and the application's name may run, from
    /// [`Self::title`]: to one gap before where the first control is **whether
    /// or not it is shown** — so revealing the controls never reflows the
    /// title, which the specification asks for in as many words.
    pub title_room: i32,
    /// The controls that are shown, left to right. Empty while concealed.
    pub controls: Vec<EdgeControl>,
}

impl WindowEdge {
    /// How much width the title has: [`Self::title_room`], or nothing on a
    /// band that carries no title.
    #[must_use]
    pub fn the_room_a_title_has(&self) -> i32 {
        if self.title.is_some() {
            self.title_room
        } else {
            0
        }
    }

    /// The control that does this, if it is shown.
    #[must_use]
    pub fn the_control(&self, does: OnTheEdge) -> Option<EdgeControl> {
        self.controls.iter().copied().find(|it| it.does == does)
    }

    /// How large one control's target appears at this zoom, in thousandths.
    ///
    /// For the contract's *at overview scales where targets become too small,
    /// use the canvas selection interaction*. **No threshold is set here** —
    /// none is named and inventing one would be deciding a design.
    #[must_use]
    pub fn how_big_a_target_looks(zoom_in_thousandths: u32) -> u32 {
        let target = u32::try_from(A_TARGET_IS).unwrap_or(0);
        target.saturating_mul(zoom_in_thousandths) / 1000
    }
}

/// The output the band is clamped inside, in the units it is laid out in.
///
/// **One conversion, in one place.** `scale` is in hundredths — 100 is
/// one-to-one, 200 a doubled display. A scale below one-to-one is read as
/// one-to-one: 0 would divide by zero, and a display under 100 hundredths is not
/// a thing this project has.
#[must_use]
pub fn the_usable_output(pixels: Size<i32, Physical>, scale_hundredths: u32) -> Size<i32, Logical> {
    let hundredths = i64::from(scale_hundredths.max(100));
    let logical = |it: i32| i32::try_from(i64::from(it) * 100 / hundredths).unwrap_or(i32::MAX);
    Size::from((logical(pixels.w), logical(pixels.h)))
}

/// Where the band's own rectangle is, which is **not** always the window's.
///
/// At [`THE_NARROWEST_EDGE_IS`] or wider it is exactly the window's width and
/// position. Narrower, it is that wide, centred over the window, then clamped
/// into the output — **the application's content is never moved or resized to
/// make it fit**, so the band hangs over both sides and the overhang is part of
/// it: a press there reaches what it looks like it should.
fn where_the_edge_goes(
    window: Rectangle<i32, Logical>,
    output: Size<i32, Logical>,
) -> Rectangle<i32, Logical> {
    let wide = window.size.w.max(THE_NARROWEST_EDGE_IS);
    let at = if window.size.w >= THE_NARROWEST_EDGE_IS {
        window.loc.x
    } else {
        // Integer division truncates toward zero, so an odd unit goes to the
        // trailing side.
        let centred = window.loc.x - (THE_NARROWEST_EDGE_IS - window.size.w) / 2;
        let furthest = (output.w - THE_NARROWEST_EDGE_IS).max(0);
        centred.clamp(0, furthest)
    };
    Rectangle::new(
        Point::from((at, window.loc.y - THE_REGION_IS_TALL)),
        Size::from((wide, THE_REGION_IS_TALL)),
    )
}

/// The band of this window: where everything on it is.
///
/// `window` is the application's own rectangle — what
/// `crate::where_a_window_is` answers. The band is placed **above** it, so the
/// content neither moves nor shrinks. `revealed` is the host's answer, from
/// pointer, keyboard, panel and the person's *Always show window controls*.
#[must_use]
pub fn edge_of(
    window: Rectangle<i32, Logical>,
    decorations: Decorations,
    revealed: bool,
    output: Size<i32, Logical>,
) -> WindowEdge {
    let region = where_the_edge_goes(window, output);

    // **Which controls a window has is asked of `window_edge_reading`**, one
    // list rather than two that could drift: whether a control exists is an
    // accessibility question before it is a drawing one.
    let has = crate::window_edge_reading::the_controls_of(decorations);
    let how_many = i32::try_from(has.len()).unwrap_or(0);

    // **Counted from the trailing edge**, because the specimen's 1056 is not a
    // window width: the last control ends 18 in, each 4 after the one before.
    let last_at = region.loc.x + region.size.w - THE_TRAILING_INSET - A_TARGET_IS;
    let first_at = last_at - (how_many - 1).max(0) * (A_TARGET_IS + BETWEEN_TARGETS);
    let laid_out: Vec<EdgeControl> = has
        .into_iter()
        .zip(0_i32..)
        .map(|(does, which)| EdgeControl {
            does,
            target: Rectangle::new(
                Point::from((
                    first_at + which * (A_TARGET_IS + BETWEEN_TARGETS),
                    region.loc.y + THE_CONTROLS_SIT_AT,
                )),
                Size::from((A_TARGET_IS, A_TARGET_IS)),
            ),
        })
        .collect();

    // **The title's room ends one gap before where the first control is,
    // shown or not**, so a title never runs under a control that appears.
    let controls_start = laid_out
        .first()
        .map_or(region.loc.x + region.size.w - THE_TRAILING_INSET, |it| {
            it.target.loc.x
        });
    let title_at = region.loc.x + THE_TITLE_STARTS_AT;
    let title_room = (controls_start - BETWEEN_TARGETS - title_at).max(0);

    let controls = if revealed { laid_out } else { Vec::new() };

    // **The band drags, less the controls that are shown.** The whole height,
    // so a pointer travelling along it never leaves the band.
    let drag_until = controls.first().map_or(region.loc.x + region.size.w, |it| {
        it.target.loc.x - BETWEEN_TARGETS
    });
    let drag = Rectangle::new(
        region.loc,
        Size::from(((drag_until - region.loc.x).max(0), THE_REGION_IS_TALL)),
    );

    // **Only where the shell carries the title**: an application with its
    // own header has its own, and a task window carries its title inside it.
    let carries_a_title = matches!(
        decorations,
        Decorations::TheShellDraws | Decorations::AlosOwnWindow
    );
    let title = carries_a_title.then(|| Point::from((title_at, region.loc.y + THE_TITLE_SITS_AT)));

    WindowEdge {
        region,
        strip: Rectangle::new(
            Point::from((region.loc.x, region.loc.y + THE_STRIP_STARTS_AT)),
            Size::from((region.size.w, THE_STRIP_IS_TALL)),
        ),
        drag,
        title,
        title_room,
        controls,
    }
}

#[cfg(test)]
#[path = "window_edge_tests.rs"]
mod tests;
