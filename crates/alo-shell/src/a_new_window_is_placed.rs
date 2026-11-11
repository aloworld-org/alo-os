//! Where a window alo opened goes, the first time it has an extent.
//!
//! [`crate::where_a_window_opens`] decides the position and had no caller.
//! This is the caller. Until it existed, `new_toplevel` gave a window a Place
//! and no position, so **every window opened at the plane's origin and the
//! second one opened exactly on top of the first** — alo burying a window with
//! nobody choosing it.
//!
//! That is not a missing feature. `docs/features.md`'s `[v0.01] ★ Every goal is
//! a canvas` promises *nothing is stacked — no window is buried behind another
//! where the person cannot find it*, and a machine that puts the second window
//! on the first breaks it on the second window. **This makes an existing
//! promise true rather than adding one**, which is the judgement `CLAUDE.md`
//! asks every change to state: no surface here is new, no setting is offered,
//! and the rules are the owner's in
//! `docs/design/where-a-new-window-opens.md`.
//!
//! # The two spaces, which is the whole difficulty
//!
//! A window's position is a point on the **plane** — the endless canvas a
//! person pans and zooms. A fixed control's bounds are on the **screen**: the
//! Dock is attached to the display's bottom edge and stays there however far
//! the canvas is zoomed out. [`crate::where_a_window_opens`] compares the two
//! against each other, so one of them has to be converted, and the conversion
//! is the camera's:
//!
//! ```text
//! screen = (plane - camera.at) * zoom        <- crate::scene
//! plane  = screen / zoom + camera.at         <- what this file does
//! ```
//!
//! **Everything handed to the placer is on the plane**, because that is the
//! space its answer is in — the point it returns goes to
//! [`crate::place_unless_already_placed`], which sets a plane position. A
//! control passed in screen pixels without this conversion would be in the
//! right place at zoom 1 with the camera at the origin, and wrong everywhere
//! else: a person zoomed out to half would have windows avoiding a Dock twice
//! its apparent size, in the wrong part of the plane.
//!
//! # One display's worth of view, because there is one camera
//!
//! **Found while wiring this and reported rather than left silent.** The
//! viewport and the control bounds handed in are the display the frame loop
//! itself holds — `Desk::present`'s own target. On a machine with a second
//! monitor, a window opening is placed against the first display's visible
//! area and the first display's Dock.
//!
//! That is not a shortcut taken here; it is the shape of what exists. There is
//! **one camera** in this shell — `crates/alo-shell/tests/the_camera_has_one_home.rs`
//! is a guard that there is exactly one — so there is one view, and *within the
//! current view* has one meaning. A camera per viewport is canvas task 9, and
//! when it lands this function wants the camera and viewport of the display the
//! window's Place is on rather than the loop's. Nothing here has to be undone
//! for that; one argument has to stop being the primary display's.
//!
//! The honest statement of today's behaviour: a window opens where the person
//! is looking, and on a two-monitor desk *where they are looking* is a question
//! this shell cannot yet answer per display.
//!
//! # One answer to *where is that window*
//!
//! This file had a private helper that added `geometry.loc` to
//! [`crate::window_buffer_origin`] and took `geometry.size`.
//! [`crate::where_a_window_is`] landed in #599 doing the same sum, so the
//! helper went: two copies of *where a window is* is the kind of pair that
//! agrees until one of them is corrected.
//!
//! **The surface's origin alone is not the window.** A client drawing its own
//! decorations starts its buffer inside its shadow margin, and `geometry.loc`
//! is what accounts for it — so either function is right and
//! `window_buffer_origin` **on its own** is not. The third PC measured that
//! gap at about fifty pixels on GNOME Calculator.
//!
//! The cost of switching is a rounding choice: the helper rounded a window's
//! occupied rectangle **outward**, so a window covering 100.4 units blocked
//! 101. The shared function rounds to nearest. That is given up on purpose —
//! placement and the reachability checks now read the same answer, and two
//! subsystems disagreeing about where a window is costs more than a sub-pixel
//! of overlap.
//!
//! **No second scale conversion happens here, deliberately.**
//! `crate::canvas_fixed_controls` records that the owner's ruling of
//! 2026-10-01 asks for exactly one, and that the bounds it stores are in the
//! pixels the display draws — which windows are drawn in too, because clients
//! are told scale one (`crate::their_displays`). Since 2026-10-28 the Dock's
//! band and the put-aside panel's column reach those pixels through the
//! display's scale, which
//! `desktop_raster_tests::the_dock_band_and_the_panel_column_move_with_the_displays_scale`
//! holds; either way each is the rectangle the display really covers.
//! So the camera is the only thing applied, and the `Physical` marker on those
//! rectangles is dropped rather than converted.
//!
//! # When there is no room, the view can glide to empty space — 2026-11-07
//!
//! The owner's amendment to rule 5: with the default choice, a window **the
//! person opened** that has no room in view goes to the nearest empty space on
//! the Place (`crate::the_nearest_empty_space`), covering nothing, and the view
//! glides there (`crate::the_view_glides`). Three things decide whether that
//! happens, and each is one line below:
//!
//! - **there was no room.** Rules 3 and 6 are asked first and unchanged; only
//!   an answer that would lie over another window is the case the choice is
//!   about. A window with room in view opens exactly where it did before.
//! - **the person chose it** — `Server::when_there_is_no_room`, *glide to empty
//!   space* unless they chose *open in front*, which is rule 5 unchanged.
//! - **the person opened it** — `Server::the_person_opened_this_one`. Rule 8:
//!   a window an application or alo opens in the background never moves the
//!   camera, so it keeps rule 5's answer and the view stays where it is.
//!
//! **Placed against where the view is going**, not where it is mid-glide: a
//! second window opened while the first glide is still moving is placed in
//! what the person will be looking at when it ends.

use crate::Server;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Physical, Point, Rectangle, Size};

/// A control's bounds moved from the screen onto the plane.
///
/// Empty rectangles are the caller's to filter, not this function's: an empty
/// panel column is *covers nothing* rather than a control at a position, and
/// `crate::canvas_fixed_controls` says so where the field is declared.
fn onto_the_plane(
    control: Rectangle<i32, Physical>,
    at: alo_canvas::At,
    zoom: f64,
) -> Option<Rectangle<i32, Logical>> {
    // A zoom of zero would divide the plane by nothing. It cannot arrive —
    // `Zoom` is built from thousandths above zero — and it is refused rather
    // than reasoned about, because law 3 makes no exception for a panic
    // somebody has argued is unreachable.
    if !zoom.is_finite() || zoom <= 0.0 {
        return None;
    }
    let left = f64::from(control.loc.x) / zoom + f64::from(at.x);
    let top = f64::from(control.loc.y) / zoom + f64::from(at.y);
    let width = f64::from(control.size.w) / zoom;
    let height = f64::from(control.size.h) / zoom;
    if ![left, top, width, height].iter().all(|it| it.is_finite()) {
        return None;
    }
    // **Outward, so a control is never smaller than what it covers.** A Dock
    // lying across 63.5 plane units counts as 64: `fixed` is tested with
    // *completely covered*, so rounding a control inwards would let a handle
    // be declared reachable under a control that in fact covers it. Rounded as
    // one rectangle rather than as a corner and a size, which is the same
    // reason — two independent roundings can shrink a rectangle from both
    // sides at once.
    Some(
        Rectangle::<f64, Logical>::new(Point::from((left, top)), Size::from((width, height)))
            .to_i32_up(),
    )
}

/// alo's own windows — its terminal and its task windows — moved onto the
/// plane, front first: the order they are drawn in, last opened in front.
///
/// **They are windows a new one must open beside, not on.** Until 2026-11-01
/// the placer was handed the applications' windows alone, so with alo's
/// terminal open and nothing else a new application opened in the middle of
/// the view — on top of the terminal — and the round-2 frames (`557:144`) draw
/// it beside.
///
/// **Both are already on the plane** since ADR 0145 put them there — the
/// terminal in stage 1, the task windows in stage 2 — so they are handed in as
/// they are, with no camera applied: a camera applied here would be applied
/// twice.
fn alos_own_windows(
    terminal: Option<Rectangle<i32, Logical>>,
    tasks_front_first: &[Rectangle<i32, Physical>],
) -> AlosOwnWindows {
    let terminal = terminal.filter(|window| window.size.w > 0 && window.size.h > 0);
    let tasks = tasks_front_first
        .iter()
        .filter(|window| window.size.w > 0 && window.size.h > 0)
        .map(|window| {
            Rectangle::new(
                (window.loc.x, window.loc.y).into(),
                (window.size.w, window.size.h).into(),
            )
        })
        .collect();
    (terminal, tasks)
}

/// alo's terminal, where one is open, and its task windows front first — on
/// the plane.
type AlosOwnWindows = (
    Option<Rectangle<i32, Logical>>,
    Vec<Rectangle<i32, Logical>>,
);

/// What every window placed on one frame is placed against: the view, the
/// fixed controls on the plane, the strip, the reading direction and the time.
///
/// Built once a frame by [`Server::place_every_window_that_just_opened`] and
/// handed to each placing — the first, and `crate::a_window_that_grows_is_placed_again`'s.
pub(crate) struct ThisFramesView {
    /// What the person is looking at, on the plane, where any glide ends.
    view: Rectangle<i32, Logical>,
    /// The fixed controls, on the plane.
    fixed: Vec<Rectangle<i32, Logical>>,
    /// The title strip above every window at this display's scale.
    pub(crate) strip: crate::a_window_and_its_strip::ItsStrip,
    /// The person's reading direction.
    reading: alo_strings::Direction,
    /// When this frame is.
    now: std::time::Instant,
}

/// **The size a window wants room for**, or [`None`] while it has no extent.
///
/// **Up, not rounded**, and this one stays mine: a window wanting 100.4 is
/// given 101 of room, because rounding a *wanted* size down would let the
/// placer call a gap big enough when the window overflows it by a fraction.
/// `where_a_window_is` answers where an **existing** window is, which is a
/// different question and rounds to nearest.
///
/// A window with no extent is not placed. It is the reason placing runs after
/// the buffer arrives rather than at `new_toplevel`: choosing a position for
/// something with no size is choosing a position for nothing, and rule 6 wants
/// the application's own requested size.
pub(crate) fn the_size_it_wants(surface: &WlSurface) -> Option<Size<i32, Logical>> {
    let wanted = crate::scene::geometry(surface).size.to_i32_ceil();
    (wanted.w > 0 && wanted.h > 0).then_some(wanted)
}

/// Every other window on a Place, and the one a new window opens beside.
pub(crate) type TheOthersAndTheActive = (
    Vec<Rectangle<i32, Logical>>,
    Option<Rectangle<i32, Logical>>,
);

impl Server {
    /// Place every window that has an extent and has never been placed, and
    /// place again any window alo placed that has since grown over another
    /// (`crate::a_window_that_grows_is_placed_again`).
    ///
    /// Returns how many were placed or placed again, which is zero on almost
    /// every frame and is the number the demonstration and the tests read.
    ///
    /// # Why this is safe to call on every frame
    ///
    /// [`crate::place_unless_already_placed`] is the door: it places a surface
    /// that has no position and refuses one that has. So a window is placed
    /// once, on the first frame after its buffer arrives, and a person who
    /// then drags it keeps where they put it — the contract's rule 1,
    /// *explicit placement wins*, enforced by the door rather than by this
    /// function remembering anything. The one exception is a window that grows
    /// over another before anybody has touched it, and it has its own file.
    ///
    /// An unmap calls `crate::window_placement::reset`, so a window that maps
    /// again is placed again. That is rule 2's business and not this
    /// function's: a window **restored** to a saved Place is put back by
    /// `Server::put_back_where_it_was` before this runs, and arrives here
    /// already placed.
    ///
    /// # One at a time, because of rule 7
    ///
    /// Two windows opening in one frame are placed in two passes, and the
    /// second sees the first among the windows already open. Placing them from
    /// one snapshot would put both in the same free space — which is the
    /// stacking this file exists to prevent, arriving by a different route.
    pub fn place_every_window_that_just_opened(
        &mut self,
        display: &str,
        viewport: Size<i32, Physical>,
        reading: alo_strings::Direction,
    ) -> usize {
        let Some(frame) = self.this_frames_view(display, viewport, reading) else {
            return 0;
        };
        // Front-to-back, so the first unplaced window is the one nearest the
        // front, and so the order two windows opening together are placed in
        // is the order they are stacked in.
        let waiting: Vec<WlSurface> = self
            .mapped_surfaces()
            .filter(|surface| !crate::has_been_placed(surface))
            .cloned()
            .collect();
        let mut placed = 0;
        for surface in waiting {
            let Some(wanted) = the_size_it_wants(&surface) else {
                continue;
            };
            // **Rule 5 as a choice** (amended 2026-11-07): the person's own
            // opening, with no room in view, glides to the nearest empty space
            // where they kept the default. Asked whatever the choice, because a
            // window is the person's once and the record of it is spent here.
            let theirs = self.the_person_opened_this_one(&surface);
            let point = self.where_this_window_goes(&surface, wanted, theirs, &frame);
            if crate::place_unless_already_placed(&surface, point) {
                crate::a_window_that_grows_is_placed_again::alo_placed_it(&surface, wanted, theirs);
                placed += 1;
            }
        }
        placed + self.place_again_every_window_that_grew(&frame)
    }

    /// This frame's view, controls and strip, or [`None`] where the camera
    /// cannot be divided by.
    fn this_frames_view(
        &self,
        display: &str,
        viewport: Size<i32, Physical>,
        reading: alo_strings::Direction,
    ) -> Option<ThisFramesView> {
        let camera = self.the_camera_after_the_glide();
        let zoom = crate::scene::drawn_at(camera);
        if !zoom.is_finite() || zoom <= 0.0 {
            return None;
        }
        // **Every window with the title strip above it** — the new one and each
        // already open — at this display's scale, which on the plane is the
        // same at every zoom because the strip zooms with its window
        // (`crate::a_window_and_its_strip`). Without it a window opened below
        // another had its strip drawn over the gap and into the one above.
        let strip = crate::a_window_and_its_strip::ItsStrip::at(
            self.the_display_scale_edges_are_measured_at(),
        );
        let at = camera.at();
        // **The visible area, not the output.** The person can only reach what
        // the camera is showing, so a window placed outside it is a window
        // placed where nobody is looking — and rule 3 says *within the current
        // view*. Zoomed out, this is larger than the display; zoomed in, it is
        // smaller.
        //
        // **Not reduced by the fixed controls.** The contract's §2 has them
        // floating over the workspace: a window may sit partly under the Dock,
        // and what it may not do is have its handle completely covered, which
        // is the `fixed` list's own test. Subtracting them here would forbid
        // what the contract allows, shrink every Place by the Dock's thickness
        // for no rule's sake, and leave nothing for that list to catch.
        let view = Rectangle::new(
            Point::<i32, Logical>::from((at.x, at.y)),
            Size::<f64, Logical>::from((
                f64::from(viewport.w) / zoom,
                f64::from(viewport.h) / zoom,
            ))
            .to_i32_round(),
        );
        // **The fixed controls only — the Dock, the panel's column, the top
        // controls and the egress indicator.** Not other windows' name bands:
        // a band belongs to a window, windows go in `open`, and the two lists
        // mean opposite things to the placer. `open` is *do not land on this*;
        // `fixed` is *do not let this completely cover the new handle*. A band
        // in `fixed` would ask the placer to protect a new window from
        // something rule 5 explicitly allows it to open in front of, and the
        // overlap case would refuse the placement it exists to produce.
        let fixed: Vec<_> = self
            .fixed_controls
            .get(display)
            .and_then(crate::canvas_fixed_controls::FixedControls::labelled)
            .into_iter()
            .flat_map(|drawn| {
                [
                    drawn.dock_band,
                    Some(drawn.panel_reserved),
                    drawn.what_is_leaving,
                    drawn.top_controls,
                ]
            })
            .flatten()
            // An empty rectangle covers nothing, so it is dropped rather than
            // handed over as a control at a position.
            .filter(|control| control.size.w > 0 && control.size.h > 0)
            .filter_map(|control| onto_the_plane(control, at, zoom))
            .collect();
        Some(ThisFramesView {
            view,
            fixed,
            strip,
            reading,
            now: std::time::Instant::now(),
        })
    }

    /// **Where one window of `wanted` size goes** on this frame: rules 3 to 7,
    /// and where there is no room and `theirs`, the person's choice.
    pub(crate) fn where_this_window_goes(
        &mut self,
        surface: &WlSurface,
        wanted: Size<i32, Logical>,
        theirs: bool,
        frame: &ThisFramesView,
    ) -> Point<i32, Logical> {
        let (open, active) = self.the_windows_a_new_one_opens_among(surface);
        let point = frame.strip.where_a_window_opens(
            frame.view,
            &open,
            active,
            wanted,
            frame.reading,
            &frame.fixed,
        );
        if !theirs {
            return point;
        }
        self.room_for_what_the_person_opened(
            point,
            &crate::room_in_a_full_view::AnOpening {
                view: frame.view,
                open: &open,
                wanted,
                fixed: &frame.fixed,
                reading: frame.reading,
                strip: frame.strip,
            },
            frame.now,
        )
    }

    /// **Every other window on `surface`'s Place, on the plane, and the one a
    /// new window opens beside** — read fresh each time, because the window
    /// placed a moment ago is now somewhere and the next one has to see it.
    ///
    /// The windows alone; the caller adds the strip above each.
    pub(crate) fn the_windows_a_new_one_opens_among(
        &self,
        surface: &WlSurface,
    ) -> TheOthersAndTheActive {
        let place = crate::canvas_place::the_place_of(surface);
        let applications: Vec<Rectangle<i32, Logical>> = self
            .mapped_surfaces()
            .filter(|other| *other != surface)
            .filter(|other| crate::canvas_place::the_place_of(other) == place)
            .map(crate::where_a_window_is)
            .collect();
        // **alo's own windows**, on the plane and on the Place being shown,
        // each in the way only while its Place is the one a new window opens
        // on (ADR 0145).
        let mut tasks: Vec<(crate::the_tasks_a_person_opened::WhichTask, _)> =
            self.the_task_windows_on_the_plane();
        tasks.reverse();
        let (terminal, task_windows) = alos_own_windows(
            self.where_the_terminal_is_on_the_plane(),
            &tasks.iter().map(|(_, window)| *window).collect::<Vec<_>>(),
        );
        // **And Settings**, a window on the canvas since ADR 0145's third stage.
        let settings = self.where_settings_is_on_the_plane();
        // **And the record**, since the fourth.
        let the_record = self.where_the_record_is_on_the_plane();
        // **The frontmost other window, not the focused one.** Rule 8 has a
        // person-initiated opening *activate* the new window, so by the time
        // this runs the focus may already be the window being placed — and
        // *beside the active window* would then mean *beside itself*.
        // Front-to-back stacking answers what a person means by the window
        // they were working in, and cannot name the new one.
        //
        // **And the frontmost of every kind, in the order the person put
        // them** (`crate::which_window_is_in_front`): a person working in
        // alo's terminal gets the new window beside the terminal.
        let active = self
            .which_is_in_front()
            .top_to_bottom()
            .into_iter()
            .find_map(|kind| match kind {
                crate::which_window_is_in_front::AKindOfWindow::TaskWindows => {
                    task_windows.first().copied()
                }
                crate::which_window_is_in_front::AKindOfWindow::TheTerminal => terminal,
                crate::which_window_is_in_front::AKindOfWindow::Settings => settings,
                crate::which_window_is_in_front::AKindOfWindow::TheRecord => the_record,
                crate::which_window_is_in_front::AKindOfWindow::Applications => {
                    applications.first().copied()
                }
            });
        let open = applications
            .iter()
            .copied()
            .chain(terminal)
            .chain(settings)
            .chain(the_record)
            .chain(task_windows)
            .collect();
        (open, active)
    }
}

#[cfg(test)]
#[path = "a_new_window_is_placed_tests.rs"]
mod tests;
