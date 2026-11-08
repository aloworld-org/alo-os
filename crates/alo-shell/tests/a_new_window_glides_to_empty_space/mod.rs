//! **A new window with no room in view: the view glides to empty space**, on
//! real client connections, through the production placing road.
//!
//! The owner's decision of 2026-11-07, `docs/design/where-a-new-window-opens.md`
//! amended: with the default choice, a window the person opened that has no
//! room in view opens in the nearest empty space on the Place, covering
//! nothing, with the focus, and the view glides there. Each case has its
//! refusal beside it: the other choice, less motion, a window nobody asked for,
//! and a desk with room.
//!
//! The view is small — 180 by 80 — so that one 16-pixel window, which is what
//! the fixture's buffer is, fills it **with its title strip**: a strip is 32
//! tall and, over a window that narrow, as wide as the narrowest strip, 148
//! (`crate::a_window_and_its_strip`), and the gap of 16 goes round both. It was
//! 48 by 48 until 2026-11-08, when the placers began counting the strip. What is under test is the decision and
//! the glide, not the arithmetic of a full desk, which
//! `the_nearest_empty_space_tests` walks position by position.

#![expect(
    clippy::panic,
    reason = "in a test, a panic names the fixture that did not hold"
)]

use std::time::{Duration, Instant};

use super::support::{Application, Fixture};
use alo_appearance::WhenThereIsNoRoom;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Physical, Rectangle, Size};
use wayland_client::Proxy as _;

/// A view one window and its strip fill, with the gap round them.
const A_SMALL_VIEW: (i32, i32) = (180, 80);

/// An application by its identifier.
fn app(named: &str) -> alo_dock::AppId {
    match alo_dock::AppId::named(named) {
        Ok(app) => app,
        Err(why) => panic!("{named} is not an application name: {why:?}"),
    }
}

/// The windows on the screen, front first.
fn front_to_back(f: &Fixture) -> Vec<WlSurface> {
    f.backend(|s| s.mapped_surfaces().cloned().collect())
}

/// A window of `app_id`, mapped on a connection of its own, and the
/// compositor's handle on it.
fn mapped(f: &Fixture, app_id: &str) -> (Application, WlSurface) {
    let before = front_to_back(f);
    let mut app = Application::new(f);
    app.toplevel.set_app_id(app_id.to_owned());
    app.configure();
    app.attach();
    app.sync();
    let new: Vec<WlSurface> = front_to_back(f)
        .into_iter()
        .filter(|surface| !before.contains(surface))
        .collect();
    match new.as_slice() {
        [one] => (app, one.clone()),
        other => panic!("expected one new window, found {}", other.len()),
    }
}

/// Place every window that just opened, as the frame loop does, in a view of
/// `viewport`.
fn placed(f: &Fixture, viewport: (i32, i32)) -> usize {
    f.backend(move |s| {
        s.place_every_window_that_just_opened(
            "",
            Size::<i32, Physical>::from(viewport),
            alo_strings::Direction::LeftToRight,
        )
    })
}

/// Where a window is on the plane.
fn where_it_is(f: &Fixture, surface: &WlSurface) -> Rectangle<i32, Logical> {
    let surface = surface.clone();
    f.backend(move |_| alo_shell::where_a_window_is(&surface))
}

/// Where the view's corner is.
fn the_view(f: &Fixture) -> (i32, i32) {
    f.backend(|s| {
        let at = s.the_camera().at();
        (at.x, at.y)
    })
}

/// A desk whose small view one editor fills, with the editor holding the
/// keyboard — and, where `asked`, the person having started Files by their
/// own road.
fn a_full_view(asked: bool) -> (Fixture, Application, WlSurface) {
    let f = Fixture::keyboard();
    let (editor, root) = mapped(&f, "org.example.Editor");
    assert_eq!(
        placed(&f, A_SMALL_VIEW),
        1,
        "the control: the editor was placed"
    );
    assert!(
        f.focus_surface(root.clone()).is_ok(),
        "the control: the editor has the keyboard"
    );
    if asked {
        let not_started = f.backend(|s| {
            s.the_person_asked_to_start(app("org.kde.dolphin"));
            s.start_what_was_asked_for(&|_, _| true)
        });
        assert!(not_started.is_empty(), "the control: the start was sent");
    }
    (f, editor, root)
}

/// The view a window opens into, as a rectangle on the plane.
fn view_at(corner: (i32, i32)) -> Rectangle<i32, Logical> {
    Rectangle::new(corner.into(), A_SMALL_VIEW.into())
}

/// **The person's window, no room: it opens in empty space, covering nothing,
/// the view glides there, and the window has the focus.**
#[test]
fn the_persons_window_with_no_room_glides_to_empty_space_with_the_focus() {
    let (f, _editor, editor) = a_full_view(true);
    let (mut files, theirs) = mapped(&f, "org.kde.dolphin");
    assert_eq!(placed(&f, A_SMALL_VIEW), 1);
    let (editor_is, files_is) = (where_it_is(&f, &editor), where_it_is(&f, &theirs));
    assert!(
        !editor_is.overlaps(files_is),
        "the new window covers the editor: {files_is:?} over {editor_is:?}"
    );
    assert!(
        f.backend(|s| s.the_view_is_gliding()),
        "the view is not gliding to the new window"
    );
    assert_eq!(the_view(&f), (0, 0), "the view jumped rather than glided");
    // Once it has ended — its timing is `the_view_glides_tests`' — the new
    // window is wholly in view.
    let began = Instant::now();
    assert!(!f.backend(move |s| s.the_glide_moves_on(began + Duration::from_secs(5))));
    let after = the_view(&f);
    assert_ne!(after, (0, 0), "the view did not move");
    assert!(
        view_at(after).contains_rect(files_is),
        "the view glided somewhere the new window is not: {after:?}, {files_is:?}"
    );
    assert_eq!(
        f.backend(|s| s.the_camera().zoom()),
        alo_canvas::Zoom::LIFE_SIZE,
        "the view zoomed"
    );
    assert_eq!(
        where_it_is(&f, &editor),
        editor_is,
        "the editor moved or resized"
    );
    // **The focus**, by rule 8's road for a window the person opened.
    f.backend(|s| s.what_the_person_opened_comes_in_front());
    files.sync();
    assert_eq!(
        files.events.keyboard.surfaces.last(),
        Some(&files.surface.id().protocol_id()),
        "the new window did not get the keyboard"
    );
}

/// **The other choice**: open in front, offset — today's rule 5 — and the view
/// stays where it is.
#[test]
fn with_open_in_front_chosen_the_view_stays_and_nothing_glides() {
    let (f, _editor, _root) = a_full_view(true);
    f.backend(|s| s.when_there_is_no_room_is(WhenThereIsNoRoom::OpenInFront));
    let (_files, theirs) = mapped(&f, "org.kde.dolphin");
    assert_eq!(placed(&f, A_SMALL_VIEW), 1);
    assert!(
        !f.backend(|s| s.the_view_is_gliding()),
        "it glides when the person chose not to"
    );
    assert_eq!(the_view(&f), (0, 0), "the view moved");
    // The control: the same desk with the default does glide, so this is the
    // choice speaking and not a desk with room.
    let (g, _editor, _root) = a_full_view(true);
    let (_again, glided) = mapped(&g, "org.kde.dolphin");
    assert_eq!(placed(&g, A_SMALL_VIEW), 1);
    assert!(
        g.backend(|s| s.the_view_is_gliding()),
        "the control: the default glides"
    );
    assert_ne!(
        where_it_is(&f, &theirs).loc,
        where_it_is(&g, &glided).loc,
        "the two choices put the window in the same place"
    );
}

/// **Less motion asked for: the view jumps**, to the same place it would
/// have glided to.
#[test]
fn with_reduced_motion_the_view_jumps_to_the_empty_space() {
    let (f, _editor, editor) = a_full_view(true);
    f.backend(|s| s.the_motion_is_reduced(true));
    let (_files, theirs) = mapped(&f, "org.kde.dolphin");
    assert_eq!(placed(&f, A_SMALL_VIEW), 1);
    assert!(
        !f.backend(|s| s.the_view_is_gliding()),
        "it glides with less motion asked for"
    );
    let files_is = where_it_is(&f, &theirs);
    assert!(!where_it_is(&f, &editor).overlaps(files_is));
    assert!(
        view_at(the_view(&f)).contains_rect(files_is),
        "the view did not jump to the new window"
    );
}

/// **A window nobody asked for never moves the view**, whatever the choice:
/// rule 8.
#[test]
fn a_window_opened_in_the_background_never_moves_the_view() {
    let (f, _editor, _root) = a_full_view(false);
    let (_files, _theirs) = mapped(&f, "org.kde.dolphin");
    assert_eq!(placed(&f, A_SMALL_VIEW), 1);
    assert!(
        !f.backend(|s| s.the_view_is_gliding()),
        "a window nobody asked for moved the view"
    );
    assert_eq!(the_view(&f), (0, 0));
    assert!(!f.backend(move |s| s.the_glide_moves_on(Instant::now() + Duration::from_secs(5))));
    assert_eq!(the_view(&f), (0, 0), "a later frame moved the view");
}

/// **Room in view: nothing changes** — rules 3 and 6 — and the view stays.
#[test]
fn with_room_in_view_nothing_glides() {
    let roomy = (800, 300);
    let f = Fixture::keyboard();
    let (_editor, editor) = mapped(&f, "org.example.Editor");
    assert_eq!(placed(&f, roomy), 1);
    assert!(f.focus_surface(editor.clone()).is_ok());
    let not_started = f.backend(|s| {
        s.the_person_asked_to_start(app("org.kde.dolphin"));
        s.start_what_was_asked_for(&|_, _| true)
    });
    assert!(not_started.is_empty());
    let (_files, theirs) = mapped(&f, "org.kde.dolphin");
    assert_eq!(placed(&f, roomy), 1);
    assert!(
        !f.backend(|s| s.the_view_is_gliding()),
        "it glided with room in view"
    );
    assert_eq!(the_view(&f), (0, 0));
    let (editor_is, files_is) = (where_it_is(&f, &editor), where_it_is(&f, &theirs));
    // **One gap between the two strips**, which over windows this narrow are
    // wider than the windows: the gap is kept between what is drawn.
    let strip = |window: Rectangle<i32, Logical>| {
        alo_shell::edge_of(
            window,
            alo_shell::Decorations::TheShellDraws,
            true,
            Size::from((1_000_000, 1_000_000)),
        )
        .strip
        .unwrap_or_else(|| panic!("a revealed edge has a strip"))
    };
    let (editors, files) = (strip(editor_is), strip(files_is));
    assert_eq!(
        files.loc.x,
        editors.loc.x + editors.size.w + 16,
        "rule 3: beside the active window, one gap to the right of its strip"
    );
    assert_eq!(files_is.loc.y, editor_is.loc.y, "and level with it");
}

/// A window of `app_id` whose buffer is larger than the window: `window` is its
/// window geometry and `margins` (left, top, right, bottom) the shadow a client
/// drawing its own decorations leaves round it — Dolphin's, on the walk of
/// 2026-11-08, was about 57 across and 25 above.
fn mapped_with_margins(
    f: &Fixture,
    app_id: &str,
    window: (i32, i32),
    (left, top, right, bottom): (i32, i32, i32, i32),
) -> (Application, WlSurface) {
    let before = front_to_back(f);
    let mut app = Application::new(f);
    app.toplevel.set_app_id(app_id.to_owned());
    app.configure();
    app.xdg.set_window_geometry(left, top, window.0, window.1);
    assert!(
        app.set_a_viewport_destination(window.0 + left + right, window.1 + top + bottom),
        "the control: this compositor offers a viewport"
    );
    app.attach();
    app.sync();
    let new: Vec<WlSurface> = front_to_back(f)
        .into_iter()
        .filter(|surface| !before.contains(surface))
        .collect();
    match new.as_slice() {
        [one] => (app, one.clone()),
        other => panic!("expected one new window, found {}", other.len()),
    }
}

/// What is drawn of a window at life size: the window and the title strip
/// `edge_of` lays out above it, which is the layout the painter draws.
fn drawn(window: Rectangle<i32, Logical>) -> Rectangle<i32, Logical> {
    let strip = alo_shell::edge_of(
        window,
        alo_shell::Decorations::TheShellDraws,
        true,
        Size::from((1_000_000, 1_000_000)),
    )
    .strip
    .unwrap_or_else(|| panic!("a revealed edge has a strip"));
    strip.merge(window)
}

/// How far apart two rectangles are: the larger of the gaps across and down,
/// negative where they overlap.
fn apart(one: Rectangle<i32, Logical>, other: Rectangle<i32, Logical>) -> i32 {
    let across =
        (other.loc.x - (one.loc.x + one.size.w)).max(one.loc.x - (other.loc.x + other.size.w));
    let down =
        (other.loc.y - (one.loc.y + one.size.h)).max(one.loc.y - (other.loc.y + other.size.h));
    across.max(down)
}

/// **The walk of 2026-11-08, on real connections**: a window filling most of a
/// 1280 × 800 view, and the person opening Files — a client whose buffer has
/// margins round its window geometry. The new window and its strip cover
/// nothing that is drawn of the first, and keep the gap from it.
///
/// What failed on the machine: the placers kept the gap to the window and not
/// to the title strip alo draws above it, so Files' strip was drawn 14 pixels
/// into the window above. And the window is placed by its geometry, not its
/// buffer, so the margins are the client's own and take no room — they are
/// checked here because a placer reading the buffer would pass the first half.
#[test]
fn a_window_with_margins_keeps_the_gap_from_what_is_drawn_of_the_window_before_it() {
    let view = (1280, 800);
    let f = Fixture::keyboard();
    let (_editor, editor) =
        mapped_with_margins(&f, "org.example.Editor", (924, 614), (20, 20, 20, 20));
    assert_eq!(
        placed(&f, view),
        1,
        "the control: the first window was placed"
    );
    assert!(f.focus_surface(editor.clone()).is_ok());
    let not_started = f.backend(|s| {
        s.the_person_asked_to_start(app("org.kde.dolphin"));
        s.start_what_was_asked_for(&|_, _| true)
    });
    assert!(not_started.is_empty(), "the control: the start was sent");
    let (_files, theirs) = mapped_with_margins(&f, "org.kde.dolphin", (760, 566), (57, 25, 57, 57));
    assert_eq!(placed(&f, view), 1, "the control: Files was placed");
    let (editor_is, files_is) = (where_it_is(&f, &editor), where_it_is(&f, &theirs));
    assert_eq!(
        (files_is.size.w, files_is.size.h),
        (760, 566),
        "the control: the window is its geometry, not its buffer"
    );
    let gap = alo_shell::A_SMALL_GAP;
    assert!(
        apart(drawn(files_is), drawn(editor_is)) >= gap,
        "Files and its strip, {:?}, are {} from what is drawn of the window before it, {:?}, \
         not the gap of {gap}",
        drawn(files_is),
        apart(drawn(files_is), drawn(editor_is)),
        drawn(editor_is)
    );
}
