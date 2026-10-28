//! The ordinary desktop for one display: the dock, and the two windows a person
//! may have open over the room the dock leaves.
//!
//! The windows are laid out in what is left of the display once the dock has
//! taken the bottom, so no window covers the dock. With one window open it has
//! that room; with both, the room is shared along its longer side, and the window
//! of what is running takes the half a person starts reading from. Each window's
//! rows are `crate::running_rows` and `crate::filling_rows`, set out by
//! `crate::desktop_list`.
//!
//! # The clock, the battery, the network and the volume are not drawn
//!
//! They were laid out inside the dock's status area, and [ADR
//! 0076](../../../docs/decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md)
//! took the status area off the Dock: a clock is not something a person opens or
//! brings into focus. **The promise is not withdrawn** — `docs/features.md` still
//! carries it at v0.5 — but it has no location now, and
//! `docs/autonomy/evidence-a-person-can-work-on-it-all-day.md` records that it is owed one before it is owed
//! an implementation. Drawing them somewhere else would be this file picking that
//! location, which is the shell plan's to pick.
//!
//! `crate::status_items` is untouched and still decides what they *say*: it is the
//! half that was never about where they go, and `alo-desktop` reads it.
//!
//! The egress indicator did not go with them. It has a corner of its own
//! (`crate::egress_status_place`) and sits exactly where it sat.

use alo_dock::{Dock, OnTheDock, Showing};
use alo_put_aside::the_region_the_panel_claims::WhichEdge;
use alo_strings::{Direction, Strings};
use cosmic_text::FontSystem;
use smithay::utils::{Physical, Rectangle};

use crate::desktop_list::{ListLook, ListPicture, ListShows};
use crate::dock_raster::DockPicture;
use crate::{DesktopLook, FillingShows, FillingWindow, RenderError, RunningShows, RunningWindow};

/// The desktop for one display, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DesktopPicture {
    /// The alo Bar's panel, when somebody has it open.
    ///
    /// **Drawn above everything but the task windows**, because it is what the
    /// keyboard is in: a panel a person is typing into, covered by anything,
    /// is a panel they cannot read what they typed in.
    pub(crate) the_bar: Option<crate::bar_raster::BarPicture>,
    /// The display size it was laid out for.
    pub(crate) size: (i32, i32),
    /// The dock, or nothing when a window needs the room it sits in and the
    /// person asked for it to give way.
    ///
    /// **[`Option`] rather than a flag on the picture**, so that painting a
    /// dock which should not be showing is not something a caller can do by
    /// forgetting: there is no `solids` to reach for. The band is laid out
    /// either way and still decides where the two desktop panels go — see
    /// [`crate::dock_room`] for why the work area must not depend on the
    /// person's choice.
    pub(crate) dock: Option<DockPicture>,
    /// The panel of put-aside windows at its edge.
    ///
    /// Not an [`Option`]: unlike the dock, the panel has no *give way* setting
    /// and a person who has put nothing aside gets a rail of no height rather
    /// than an absent picture. The reserved column it carries exists either
    /// way, because the panel owns its edge whether or not anything is in it.
    pub(crate) panel: crate::panel_raster::PanelPicture,
    /// The room the top controls reserve, or [`None`] when a window fills the
    /// screen and they give way. **Reserved rather than painted** — canvas task
    /// 6a's contents are not laid out yet, and the panel's own reserved column is
    /// the precedent: a region exists because a surface owns its edge, whatever is
    /// drawn in it.
    pub(crate) top_controls: Option<smithay::utils::Rectangle<i32, smithay::utils::Physical>>,
    /// Each open task window's furniture.
    ///
    /// **A window that could not be rasterised is left out rather than
    /// refusing the frame.** A task window too small for its own layout is one
    /// window's problem; refusing the frame would take the whole desktop down
    /// with it, including the Dock the person needs in order to do anything
    /// about it. `crate::task_raster::furniture` says what it refuses.
    pub(crate) tasks: Vec<crate::task_raster::TaskPicture>,
    /// The window of what a person can install, where one is open and could be
    /// drawn.
    ///
    /// **A window that refuses is left out rather than refusing the frame**,
    /// for the reason the task windows above give: one window too small for
    /// its own layout is one window's problem, and taking the desktop down
    /// with it would take the Dock down too.
    pub(crate) market: Option<crate::market_raster::MarketPicture>,
    /// The window of what is running.
    pub(crate) running: ListPicture,
    /// The window of what is filling the disk.
    pub(crate) filling: ListPicture,
    /// How this display is divided, and what a drop would do.
    pub(crate) division: crate::division_raster::DivisionPicture,
    /// The colour the plane's surface is, for this frame's display.
    ///
    /// Carried rather than drawn here: the surface is behind **every** element,
    /// including a client's window, so it is the frame's clear colour and not
    /// one more picture in a list that composites above it.
    pub(crate) surface: alo_appearance::Colour,
}

/// What a desktop has on it, as the crates that decide each said it.
///
/// One value rather than four arguments, because they are one thing: what is on
/// this display now. A raster that took them loose would grow a fifth the next
/// time a surface is added, and the order of four references is a mistake
/// waiting to be made.
///
/// **It held the status items until ADR 0076**, and losing one is what a value
/// like this is for: the field went and every caller was named by the compiler
/// rather than found by reading.
#[derive(Clone, Copy)]
pub(crate) struct Shown<'a> {
    /// The window of what is running.
    pub(crate) running: &'a ListShows,
    /// The window of what is filling the disk.
    pub(crate) filling: &'a ListShows,
    /// The task windows open on this display, already placed.
    pub(crate) tasks: &'a [crate::task_raster::ATaskOnTheDisplay<'a>],
    /// The window of what a person can install, where one is open.
    pub(crate) market: Option<crate::market_raster::AMarketOnTheDisplay<'a>>,
    /// How this display is divided.
    pub(crate) division: &'a alo_dividing::Division,
    /// What letting go of a dragged window would do.
    pub(crate) offer: &'a alo_dividing::Offer,
    /// Every mapped window on this display, in this display's physical
    /// pixels, so that the dock can be asked whether one needs the room it
    /// sits in.
    ///
    /// Handed in for the same reason the readings are: only the caller knows
    /// which display's windows these are and what scale puts them in the
    /// band's own space. An empty slice is the true answer on a desktop with
    /// nothing open, not a placeholder — see [`crate::dock_room`].
    pub(crate) windows: &'a [smithay::utils::Rectangle<i32, smithay::utils::Physical>],
    /// Whether the list of applications the Dock had no room for is open.
    ///
    /// Handed in like the rest, and for the sharpest version of the same
    /// reason: a person opened it with a press, presses reach the `Server`, and
    /// this crate's drawing half has no way to know.
    pub(crate) the_dock_has_open: &'a crate::WhatTheDockHasOpen,
    /// What each application draws on the Dock, where it draws anything.
    pub(crate) what_each_application_draws: &'a alo_dock::artwork::TheArtwork,
    /// Everything on the Dock that a person reads, already said by the lane.
    pub(crate) the_docks_words: &'a crate::the_docks_words::TheDocksWords,
    /// The windows a person put aside, which the panel at the edge shows.
    ///
    /// Handed in like the rest: `alo-put-aside` decides what is in the panel
    /// and holds no geometry at all, and [`crate::panel_raster`] is where that
    /// becomes rectangles on this display.
    pub(crate) put_aside: &'a alo_put_aside::Panel,
    /// Whether a window on this display is filling the screen.
    ///
    /// **This is the whole of what separates full screen from maximised as far
    /// as a person can see.** Both are the same area; a window filling the
    /// screen is the one the Dock and the panel give way to, and
    /// `docs/design/the-alo-dock.md` says so: *true full screen covers the
    /// Dock*. Asked of the shell by the caller, because only the caller knows
    /// which display's windows these are — the seam `windows` already takes.
    pub(crate) filling_the_screen: bool,
    /// Whether the put-aside panel is on the screen at this moment.
    ///
    /// **Asked of the desktop, because `alo_dock::Revealing` is session state.** That machine
    /// decides it from the edge, the surface, the keyboard, a drag and an open menu, and the
    /// crate on the other side of `crate::TheDesktop` holds it — the compositor holds none.
    ///
    /// A desktop that never advances the machine answers `false` for ever, which draws a panel
    /// that keeps its column and never its rail. That is the honest picture for a desktop with
    /// no pointer rather than a panel stuck shut by a bug.
    pub(crate) panel_is_revealed: bool,
    /// How many physical pixels this display draws for one logical one, in
    /// hundredths. 100 is one to one; 200 is a dense screen.
    ///
    /// **The display's conversion, applied once, here, at the rendering
    /// boundary** — which is what `alo_appearance::targets` says of the target
    /// floors and what nothing was actually doing until 2026-10-02. See
    /// [`crate::nested_desktop::DesktopFrame::display_scale`] for what was wrong and in
    /// which direction it failed.
    pub(crate) display_scale: u16,
    /// The applications the Dock is holding, in the order it shows them.
    ///
    /// Carried from [`crate::nested_desktop::DesktopFrame::on_the_dock`], whose
    /// note says why the desktop builds this list and this crate does not — and
    /// why a literal `0` stood here until 2026-10-09 and was the right number
    /// for the wrong reason.
    ///
    /// **It was a count until 2026-10-10.** The count is `.len()`, and the Dock
    /// needs the entries themselves to draw an icon for each — so the field
    /// carrying only the count was a projection of the answer standing in for
    /// it, and `Server::how_many_the_dock_holds` built the list and threw it
    /// away on the way here.
    pub(crate) on_the_dock: &'a [OnTheDock],
    /// What *More apps* lists, from
    /// [`crate::nested_desktop::DesktopFrame::in_more_apps`].
    pub(crate) in_more_apps: &'a [OnTheDock],
    /// Which slot on the bar the keyboard is on, from
    /// [`crate::nested_desktop::DesktopFrame::the_keyboard_is_on_the_bar`].
    pub(crate) the_keyboard_is_on_the_bar: Option<usize>,
    /// The alo Bar's panel, from
    /// [`crate::nested_desktop::DesktopFrame::the_bar`].
    pub(crate) the_bar: Option<crate::bar_raster::TheBarOnTheDisplay<'a>>,
}

/// What the running window shows, as a panel.
pub(crate) fn running_shows(window: &RunningWindow, strings: &Strings) -> ListShows {
    match window.shows() {
        RunningShows::Nothing => ListShows::Nothing,
        RunningShows::Refusal(why) => ListShows::Refusal(why.said(strings).into_text()),
        RunningShows::Running {
            running,
            moved_past,
        } => ListShows::Rows {
            remarks: crate::running_rows::remarks(running, strings),
            rows: crate::running_rows::rows(running, strings),
            first: moved_past,
            selected: None,
        },
    }
}

/// What the filling window shows, as a panel.
pub(crate) fn filling_shows(window: &FillingWindow, strings: &Strings) -> ListShows {
    match window.shows() {
        FillingShows::Nothing => ListShows::Nothing,
        FillingShows::Refusal(why) => ListShows::Refusal(why.said(strings).into_text()),
        FillingShows::Holding {
            holding,
            opened,
            selected,
        } => ListShows::Rows {
            remarks: crate::filling_rows::remarks(holding, strings),
            rows: crate::filling_rows::rows(holding, opened, strings),
            first: 0,
            selected: Some(selected),
        },
    }
}

/// Lay the desktop out on a display of `size` and rasterise it.
///
/// # Errors
/// Every refusal the dock makes, and [`RenderError::DesktopScene`] when an open
/// window cannot be held whole in the room the dock leaves.
pub(crate) fn picture(
    dock: &Dock,
    look: DesktopLook,
    shown: Shown<'_>,
    fonts: &mut FontSystem,
    size: (i32, i32),
) -> Result<DesktopPicture, RenderError> {
    let Shown {
        tasks,
        market,
        running,
        filling,
        division,
        offer,
        windows,
        put_aside,
        filling_the_screen,
        panel_is_revealed,
        display_scale,
        on_the_dock,
        in_more_apps,
        the_dock_has_open,
        the_docks_words,
        the_keyboard_is_on_the_bar,
        the_bar,
        what_each_application_draws,
    } = shown;
    // **The count the desktop handed over, not a constant.** This passed a
    // literal `0` until 2026-10-09, with a comment saying a bar holding nothing
    // is narrow and that this is true rather than a placeholder — which it was.
    // What it could not do was become anything else once something started
    // populating an `alo_dock::Holding`. `DesktopFrame::on_the_dock` carries the
    // number and its own note says why the desktop counts and this crate does
    // not.
    let dock_picture = crate::dock_raster::picture(
        dock,
        look,
        size,
        on_the_dock,
        in_more_apps,
        (the_dock_has_open, the_docks_words),
        the_keyboard_is_on_the_bar,
        what_each_application_draws,
        fonts,
    )?;

    // **Laid out first, shown or not.** The band has to exist before anything
    // can ask whether a window is over it, and `room_beside` below uses it
    // either way so that the two desktop panels get the same room whichever
    // the person chose. That is what keeps this from oscillating: the band is
    // a constant with respect to the question being asked of it.
    // **A window filling the screen covers the Dock, whatever the person chose
    // about it giving way.** Those are two different questions and only one of
    // them is a setting: `alo_dock::Hiding` is whether the Dock yields to a
    // window that needs its room, and this is a window that has taken the
    // whole screen rather than needing part of it. A
    // dock drawn over a full-screen video would be the thing the edge reveal
    // exists to make unnecessary.
    let showing = if filling_the_screen {
        Showing::Hidden
    } else {
        dock.showing(crate::dock_room::the_room(dock_picture.band, windows))
    };
    let palette = look.palette().map_err(|_| RenderError::AccentRefused)?;
    let measure = look.measure();
    let list = ListLook {
        measure,
        palette,
        right_to_left: look.reading() == Direction::RightToLeft,
    };
    let room = room_beside(&dock_picture, measure.px(16));
    let both = !matches!(running, ListShows::Nothing) && !matches!(filling, ListShows::Nothing);
    let (running_room, filling_room) = if both {
        shared(room, look.reading())
    } else {
        (room, room)
    };
    let running = crate::desktop_list::picture(running, fonts, size, running_room, list)?;
    let filling = crate::desktop_list::picture(filling, fonts, size, filling_room, list)?;
    // **A division is in logical units, so the display's conversion happens
    // here, once, at the boundary.** That sentence stood over a literal `1`
    // until 2026-10-02: *one is the scale this display is laid out at*, which is
    // true of a one-to-one screen and of nothing else. `picture` had no scale to
    // pass, so the seam was short by an argument rather than wrong — and a
    // division on a two-times display was drawn at half the room it owns.
    //
    // Hundredths straight through, because that is what `in_pixels` takes and
    // the conversion belongs where the multiplication is. The first version of
    // this line divided by a hundred here and floored 150 to 1 — the fault the
    // comment above was written to warn about, committed two edits later.
    let division = crate::division_raster::picture(
        division,
        offer,
        i32::from(display_scale),
        palette.ink,
        palette.accent,
    );

    // **Which edge the panel belongs to is read from the direction a person
    // reads in**, not assumed and not stored. The owner's ruling of 2026-09-30
    // is that the default panel is on the right and a right-to-left layout
    // mirrors it to the left *together with the Dock and the top-control
    // exclusion regions* — so the one fact this needs is already here, in the
    // look, and deriving it is why nothing names an edge.
    let panel = crate::panel_raster::picture(
        put_aside,
        look,
        size,
        match look.reading() {
            Direction::LeftToRight => WhichEdge::Right,
            Direction::RightToLeft => WhichEdge::Left,
        },
        if panel_is_revealed {
            crate::panel_raster::WhetherRevealed::Revealed
        } else {
            crate::panel_raster::WhetherRevealed::Concealed
        },
    )?;

    // **After the panel, because the band's right edge is the panel's left one.**
    // The owner's ruling of 2026-09-30: the top controls span the screen up to the
    // reserved right-panel area, and the reserved width follows the panel's current
    // width. Taking it from the column means the two cannot overlap by construction
    // rather than by two numbers agreeing — which is the same ruling's *one pointer
    // position cannot reveal two surfaces*.
    let top_controls =
        crate::top_controls_region::reserved(size, panel.reserved, filling_the_screen);

    // **Each task window's furniture, and a window that refuses is left out.**
    // `furniture` refuses a window too small for its own layout, and that is one
    // window's problem: refusing the whole frame would take the desktop down with
    // it — including the Dock a person needs in order to close the window that
    // caused it.
    let tasks = tasks
        .iter()
        .filter_map(|task| {
            crate::task_raster::furniture(
                task.at,
                look,
                task.how_many_files,
                &task.words,
                &task.said,
                task.scroll,
                fonts,
            )
            .ok()
            .map(|picture| crate::task_raster::TaskPicture {
                place: task.place,
                ..picture
            })
        })
        .collect();

    // **The market window, left out if it refuses** — the same rule as the
    // task windows above and for the same reason.
    let market = market.and_then(|open| {
        crate::market_raster::picture(
            open.at,
            open.showing,
            open.words,
            crate::market_raster::MarketLook {
                scheme: look.scheme(),
                scale: look.scale(),
            },
            fonts,
        )
    });

    // **The alo Bar, left out if it refuses** — the same rule as the market
    // and the task windows above. A panel that could not be rasterised is a
    // panel not drawn, never a half-drawn one.
    let the_bar = the_bar.and_then(|open| crate::bar_raster::picture(&open, palette, fonts));

    Ok(DesktopPicture {
        the_bar,
        tasks,
        market,
        top_controls,
        size,
        dock: match showing {
            Showing::Shown => Some(dock_picture),
            Showing::Hidden => None,
        },
        panel,
        running,
        filling,
        division,
        surface: look.surface(),
    })
}

/// The room the dock leaves above it, inset by `margin` on every side.
fn room_beside(dock: &DockPicture, margin: i32) -> Rectangle<i32, Physical> {
    let (width, height) = dock.size;
    let thick = dock.band.size.h;
    Rectangle::new(
        (margin, margin).into(),
        (
            (width - 2 * margin).max(0),
            (height - thick - 2 * margin).max(0),
        )
            .into(),
    )
}

/// One room shared by two windows along its longer side: the first for the
/// window of what is running, on the side a person starts reading from.
fn shared(
    room: Rectangle<i32, Physical>,
    reading: Direction,
) -> (Rectangle<i32, Physical>, Rectangle<i32, Physical>) {
    let gap = (room.size.w.min(room.size.h) / 64).max(1);
    if room.size.w >= room.size.h {
        let half = (room.size.w - gap) / 2;
        let left = Rectangle::new(room.loc, (half, room.size.h).into());
        let right = Rectangle::new(
            (room.loc.x + room.size.w - half, room.loc.y).into(),
            (half, room.size.h).into(),
        );
        match reading {
            Direction::LeftToRight => (left, right),
            Direction::RightToLeft => (right, left),
        }
    } else {
        let half = (room.size.h - gap) / 2;
        (
            Rectangle::new(room.loc, (room.size.w, half).into()),
            Rectangle::new(
                (room.loc.x, room.loc.y + room.size.h - half).into(),
                (room.size.w, half).into(),
            ),
        )
    }
}

#[cfg(test)]
#[path = "desktop_raster_tests.rs"]
mod tests;
