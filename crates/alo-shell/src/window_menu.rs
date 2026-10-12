//! The window menu: where it opens, what it holds, and what is unavailable.
//!
//! The compact edge keeps Close and moves the rest here, so **this file is
//! what makes the compact layout honest**. Without it, a window between 148
//! and 195 logical units would have Minimise and Maximise on no route at all —
//! the contract's *no action is silently dropped* failing while every test
//! passed, because nothing is laid out out of region and a reader correctly
//! still announces four things the window can do.
//!
//! `crate::window_edge` says where the edge and its controls are; this says
//! where the panel those controls open goes. Two files because the edge is
//! laid out from a window and the menu from a control, and because a design
//! change to one is not a change to the other.
//!
//! # Measured, from `414:27639` and `414:27653`
//!
//! | | |
//! |---|---|
//! | panel | 264 wide |
//! | side padding | 9 leading, 7 trailing — the design's own asymmetry, kept |
//! | title block | 248 × 56 at `y = 13`, selectable |
//! | action rows | 248 × 48, from `y = 69`, one after another |
//! | below the last row | 35 |
//!
//! The two masters are **identical but for one label**: `Window=Normal` says
//! *Maximise* where `Window=Maximised` says *Restore size*. So the menu has
//! one shape and one row that changes word, which is why
//! [`WhatARowDoes::Maximise`] and [`WhatARowDoes::Restore`] are two variants
//! of one row rather than two rows.
//!
//! **The 9 and the 7 are not a mistake of mine.** 264 less 248 is 16, and the
//! design puts the block at `x = 9`, leaving 7. Recorded as measured rather
//! than evened to 8 each side: a reader checking this against the file should
//! find what the file says.
//!
//! # Where it opens
//!
//! *Open 8px below the edge, anchor to Menu, clamp inside usable output and
//! away from other windows' controls. Flip above when needed if the edge
//! remains reachable.*
//!
//! So: aligned to the leading edge of the Menu control, 8 below the edge's
//! envelope, then clamped into the output, then flipped above if it would not
//! fit below — and **not flipped if flipping would put it off the top**,
//! because an edge a person cannot reach is worse than a menu that is clipped.
//!
//! # What is not here
//!
//! **Opening it.** A press on the Menu control is `crate::pointer`'s business
//! and the Mac lane's branch has the arm that will call this. This file
//! answers *where would it be and what would it hold*, which is what that arm
//! needs and what can be tested without a seat.
//!
//! **Two of its five rows have no word yet**, and that is the one thing
//! stopping this menu from being announced. `alo-access`' vocabulary holds an
//! identity — every word it declares is a name the tree says — so words for
//! Move and Resize must land in the same change that makes the tree say them.
//! Three of the five reuse sentences that already exist and already say they
//! have more than one home.
//!
//! **Two of its five actions have no [`Action`] behind them.** Move and Resize
//! are not in `alo_shortcuts::Action` — measured 2026-10-11 — and the design
//! forbids inventing one: *do not guess a keyboard chord: use the project's
//! assigned window-menu action.* They are rows with names and no action, and
//! [`MenuRow::what_it_does`] answers [`None`] for them, exactly as
//! `crate::window_edge_reading::what_it_does` does for the Menu control
//! itself. A row that cannot act is still announced, because a person being
//! told what a window can do is not the same as being able to do it from here.

use alo_shortcuts::Action;
use alo_strings::Word;
use smithay::utils::{Logical, Point, Rectangle, Size};

use crate::window_edge::{OnTheEdge, WindowEdge};

/// How wide the panel is. `414:27639`.
pub const THE_MENU_IS_WIDE: i32 = 264;

/// How tall one action row is, and how far apart they therefore sit.
pub const A_ROW_IS_TALL: i32 = 48;

/// How wide a row is inside the panel: 248 of 264.
pub const A_ROW_IS_WIDE: i32 = 248;

/// Where a row starts, across. The design's leading padding.
const ROWS_START_ACROSS: i32 = 9;

/// Where the title block starts, down.
const THE_TITLE_STARTS_DOWN: i32 = 13;

/// How tall the title block is when one line fits.
pub const THE_TITLE_BLOCK_IS_TALL: i32 = 56;

/// What sits below the last row: `344 - 13 - 56 - 5 × 48`.
const BELOW_THE_LAST_ROW: i32 = 35;

/// How far below the edge the panel opens. `space/2`, and the contract's *open
/// 8px below the edge*.
pub const BELOW_THE_EDGE: i32 = 8;

/// One thing the menu can do to the window.
///
/// Five rows in the design's order, which is the order a keyboard walks them
/// and the order a reader meets them — one order, so what a person hears and
/// what the arrow keys do cannot disagree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatARowDoes {
    /// Put the window aside.
    Minimise,
    /// Make it fill the screen. The same row as [`Self::Restore`], and only
    /// one of the two is ever present.
    Maximise,
    /// Put it back to the size it was. `414:27653`'s *Restore size*.
    Restore,
    /// Begin moving it.
    Move,
    /// Begin resizing it.
    Resize,
    /// Ask the application to close, with its own unsaved-work handling.
    Close,
}

impl WhatARowDoes {
    /// The action this row performs, where the project has one.
    ///
    /// [`None`] for [`Self::Move`] and [`Self::Resize`]: `alo_shortcuts` has
    /// no action for either, and the design forbids inventing one. Recorded
    /// rather than faked, which is `crate::window_edge_reading`'s rule for the
    /// Menu control and the same reason — a name a person is told and a thing
    /// the machine can do are two claims, and only one of them is true here.
    #[must_use]
    pub const fn what_it_does(self) -> Option<Action> {
        match self {
            Self::Minimise => Some(Action::MinimiseWindow),
            // One action for both, which is what its word already says:
            // *Maximise the window, or put it back*. The two rows differ in
            // the word a person reads, not in what runs.
            Self::Maximise | Self::Restore => Some(Action::MaximiseWindow),
            Self::Close => Some(Action::CloseWindow),
            Self::Move | Self::Resize => None,
        }
    }

    /// What a person reads, and what a reader speaks.
    ///
    /// Three of the five reuse a sentence that already exists, and each of
    /// those already carries a note saying it has two homes — a row in the
    /// list of shortcuts, and the name a reader speaks for a button drawn on
    /// every window (ADR 0089). The menu is a third home and they read
    /// naturally spoken in it.
    ///
    /// **Move and Resize are named from `alo_access`' own words**, not from an
    /// action's, because neither is an action: `alo_shortcuts` has no action
    /// for either and the design forbids inventing a chord. ADR 0089 names a
    /// control by what it does, and what these do has a sentence without
    /// having an action.
    ///
    /// They are in `alo-access` rather than here because that crate's
    /// vocabulary is collected into the one this machine can say, and
    /// `alo-shell`'s is not — a word declared here would be a string a person
    /// can see and no translator can reach. The same crate holds an identity
    /// between the words it declares and the names the tree says, so these two
    /// landed with `crate::window_edge_reading::what_a_reader_is_told`
    /// announcing them.
    ///
    /// **Maximise and Restore share one word on purpose.** It says *Maximise
    /// the window, or put it back* — one sentence covering both directions,
    /// which is one action's worth of meaning. The label a person reads is
    /// drawn from the window's state; what a reader speaks is the capability.
    #[must_use]
    pub const fn its_own_word(self) -> Word {
        match self {
            Self::Minimise => alo_shortcuts::words::MINIMISE_WINDOW,
            Self::Maximise | Self::Restore => alo_shortcuts::words::MAXIMISE_WINDOW,
            Self::Close => alo_shortcuts::words::CLOSE_WINDOW,
            Self::Move => alo_access::words::MOVE_THE_WINDOW,
            Self::Resize => alo_access::words::RESIZE_THE_WINDOW,
        }
    }

    /// What a reader is told it can do with this row.
    ///
    /// [`State::CanBeUsed`](alo_access::State::CanBeUsed) when the window
    /// allows it, and [`State::ReadOnly`](alo_access::State::ReadOnly) when it
    /// does not — *read and nothing more*, which is the true thing to say
    /// about a row that is there and will not act.
    ///
    /// **That is the contract's *explain unavailable actions accessibly*, and
    /// only half of it.** A reader is told the row cannot be used; it is not
    /// told *why* — that a window with one size cannot be resized. The reason
    /// would need a sentence of its own and is recorded here as owed rather
    /// than invented, because a reason given in English only would be worse
    /// than a state said in the person's own language.
    #[must_use]
    pub const fn how_a_reader_finds_it(available: bool) -> alo_access::State {
        if available {
            alo_access::State::CanBeUsed
        } else {
            alo_access::State::ReadOnly
        }
    }
}

/// What this window will let the menu do to it.
///
/// **Asked of the window rather than assumed**, which is the contract's
/// *respect application capabilities*. A window with one size cannot be
/// resized and a maximise row on it would be a button that does nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhatTheWindowAllows {
    /// Whether it can be put aside.
    pub minimise: bool,
    /// Whether it can be made to fill the screen, or put back.
    pub maximise: bool,
    /// Whether it can be moved.
    pub move_it: bool,
    /// Whether it can be resized — false for a window whose smallest and
    /// largest size are the same.
    pub resize: bool,
    /// Whether it can be asked to close. **Always true in practice**: closing
    /// is a request to the application and every application may be asked.
    pub close: bool,
}

impl WhatTheWindowAllows {
    /// Everything, which is what the shell advertises to clients today.
    ///
    /// `crate::surfaces` sends `WmCapabilities::{Maximize, Minimize,
    /// Fullscreen}`, so a window that said nothing about its own limits may be
    /// offered all of these.
    #[must_use]
    pub const fn everything() -> Self {
        Self {
            minimise: true,
            maximise: true,
            move_it: true,
            resize: true,
            close: true,
        }
    }

    /// Whether this row is available on this window.
    #[must_use]
    pub const fn allows(&self, does: WhatARowDoes) -> bool {
        match does {
            WhatARowDoes::Minimise => self.minimise,
            WhatARowDoes::Maximise | WhatARowDoes::Restore => self.maximise,
            WhatARowDoes::Move => self.move_it,
            WhatARowDoes::Resize => self.resize,
            WhatARowDoes::Close => self.close,
        }
    }
}

/// One row of the menu, where it is and whether it can be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuRow {
    /// What pressing it does.
    pub does: WhatARowDoes,
    /// The 248 × 48 it answers in.
    pub row: Rectangle<i32, Logical>,
    /// Whether this window allows it.
    ///
    /// **Present and unavailable rather than absent.** The contract: *explain
    /// unavailable actions accessibly*. A row that vanished would leave a
    /// person who knows the menu wondering whether they misremembered it, and
    /// would move every row below it between one window and the next.
    pub available: bool,
}

/// The whole menu, laid out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowMenu {
    /// The panel. 264 wide, as tall as its contents need and the output allows.
    pub panel: Rectangle<i32, Logical>,
    /// The full window title, selectable and copyable.
    pub title: Rectangle<i32, Logical>,
    /// The rows, in the design's order.
    pub rows: Vec<MenuRow>,
    /// Whether the panel had to be shortened to fit, so its contents scroll.
    ///
    /// *The menu grows only within available height; overflow scrolls.*
    pub scrolls: bool,
    /// Whether it opened above the edge rather than below it.
    pub above: bool,
}

impl WindowMenu {
    /// The rows a person can actually use, in order.
    #[must_use]
    pub fn what_can_be_used(&self) -> Vec<MenuRow> {
        self.rows
            .iter()
            .copied()
            .filter(|it| it.available)
            .collect()
    }

    /// Which row a position is on, if any.
    ///
    /// The menu's own hit test, and it is here rather than in
    /// `crate::window_edge_pointing` because a menu is not an edge: its rows
    /// are 248 × 48 and its panel is not laid out from a window.
    #[must_use]
    pub fn what_is_under(&self, at: Point<i32, Logical>) -> Option<MenuRow> {
        self.rows.iter().copied().find(|it| it.row.contains(at))
    }
}

/// Where this window's menu opens, and what it holds.
///
/// [`None`] when the edge has no Menu control, which is every window at
/// [`crate::window_edge::FULL_CONTROLS_NEED`] and wider: the full layout puts
/// its three actions on the edge and draws no menu target, so there is nothing
/// for a menu to anchor to.
///
/// **That leaves Move and Resize unreachable from a wide window's edge**, and
/// it is the design's own arrangement rather than an omission here — the
/// `State=Full controls` master has four targets and none is a menu. A
/// keyboard route to the menu would reach it at any width and is §6's.
///
/// `how_tall_the_title_is` is what the title needs once wrapped, which the
/// caller measures because it owns the fonts. Pass
/// [`THE_TITLE_BLOCK_IS_TALL`] for a title that fits on the design's two
/// lines.
#[must_use]
pub fn the_menu_of(
    edge: &WindowEdge,
    allows: WhatTheWindowAllows,
    maximised: bool,
    how_tall_the_title_is: i32,
    output: Size<i32, Logical>,
) -> Option<WindowMenu> {
    let anchor = edge.controls.iter().find(|it| it.does == OnTheEdge::Menu)?;

    let does = [
        WhatARowDoes::Minimise,
        if maximised {
            WhatARowDoes::Restore
        } else {
            WhatARowDoes::Maximise
        },
        WhatARowDoes::Move,
        WhatARowDoes::Resize,
        WhatARowDoes::Close,
    ];

    let title_tall = how_tall_the_title_is.max(THE_TITLE_BLOCK_IS_TALL);
    let rows_tall = A_ROW_IS_TALL * i32::try_from(does.len()).unwrap_or(0);
    let wants = THE_TITLE_STARTS_DOWN + title_tall + rows_tall + BELOW_THE_LAST_ROW;

    // Below the edge, aligned to the leading edge of the Menu control.
    let below_at = edge.region.loc.y + edge.region.size.h + BELOW_THE_EDGE;
    let room_below = output.h - below_at;
    let room_above = (edge.region.loc.y - BELOW_THE_EDGE).max(0);

    // **Flipped only if there is more room above and the edge stays
    // reachable.** An edge pushed off the top is worse than a menu that
    // scrolls, which is what *if the edge remains reachable* rules out.
    let above = wants > room_below && room_above > room_below;
    let tall = if above {
        wants.min(room_above)
    } else {
        wants.min(room_below.max(0))
    };
    let at_y = if above {
        (edge.region.loc.y - BELOW_THE_EDGE - tall).max(0)
    } else {
        below_at
    };

    let at_x = anchor
        .target
        .loc
        .x
        .clamp(0, (output.w - THE_MENU_IS_WIDE).max(0));
    let panel = Rectangle::new(
        Point::from((at_x, at_y)),
        Size::from((THE_MENU_IS_WIDE, tall)),
    );

    let title = Rectangle::new(
        Point::from((at_x + ROWS_START_ACROSS, at_y + THE_TITLE_STARTS_DOWN)),
        Size::from((A_ROW_IS_WIDE, title_tall)),
    );

    let rows_from = at_y + THE_TITLE_STARTS_DOWN + title_tall;
    let rows = does
        .into_iter()
        .enumerate()
        .map(|(which, does)| MenuRow {
            does,
            row: Rectangle::new(
                Point::from((
                    at_x + ROWS_START_ACROSS,
                    rows_from + i32::try_from(which).unwrap_or(0) * A_ROW_IS_TALL,
                )),
                Size::from((A_ROW_IS_WIDE, A_ROW_IS_TALL)),
            ),
            available: allows.allows(does),
        })
        .collect();

    Some(WindowMenu {
        panel,
        title,
        rows,
        scrolls: tall < wants,
        above,
    })
}

#[cfg(test)]
#[path = "window_menu_tests.rs"]
mod tests;
