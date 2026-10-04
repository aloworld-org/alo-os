//! **A Place remembers time** — `docs/autonomy/the-canvas-and-its-places.md`
//! task 8, on the production path.
//!
//! The acceptance: *arrange windows on a Place, change it, and reach a named
//! earlier state through the production path — the layout that returns is the
//! one that was held, and the person is never shown a state they cannot get
//! back from.* Its own ruling says a test that round-trips a series in memory
//! is **progress, not completion**, so nothing here round-trips an
//! `Arrangement`: every test moves real frames on a real plane and reads back
//! where they ended up.
//!
//! # What *the production path* is, until the ribbon exists
//!
//! `docs/features.md:516`'s **time ribbon** — the strip at the bottom edge — is
//! `[v1.1]` and is not in this release. The capability at `:490` is, so the
//! road is the chord, which is what tasks 3 and 6 of this plan each did for the
//! same reason in their own words: *neither road is the only road*. These tests
//! press it the way the keyboard road reaches it, through
//! `dispatch_canvas_command`, and `tests/a_chord_reaches_its_action` is what
//! proves that road is entered by a key at all.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{Application, Fixture};
use alo_shortcuts::{Action, Shortcuts};
use std::time::{Duration, SystemTime};

/// The viewport every test here renders to.
const VIEWPORT: (i32, i32) = (1280, 720);

/// A moment a test chooses, so no assertion here depends on a clock.
fn moment(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000 + seconds)
}

/// A real display with an output and one window open.
///
/// **The window names itself, and that is load-bearing rather than tidy.** An
/// arrangement keys a remembered place by `app_id`, because that is the thing
/// that is the same tomorrow when a `wl_surface` is not — so a frame whose
/// application gives no name is not in any arrangement, and a test built on one
/// would assert that nothing came back and call it a passing restore.
fn a_desktop() -> (Fixture, Application) {
    let f = Fixture::keyboard();
    f.render(VIEWPORT, false, 1).expect("an output to look at");
    let mut app = Application::new(&f);
    app.toplevel.set_app_id("org.alo.Notes".to_owned());
    app.configure();
    app.attach();
    app.sync();
    (f, app)
}

/// Where the one open frame is on the plane.
fn the_frame_is_at(f: &Fixture) -> (i32, i32) {
    f.backend(|s| {
        let at = s
            .the_frames_on_the_plane()
            .into_iter()
            .next()
            .expect("a frame on the plane")
            .at();
        (at.x, at.y)
    })
}

/// Put the frame here, the way a drag does.
fn put_the_frame_at(f: &Fixture, (x, y): (i32, i32)) {
    let root = f.root();
    f.backend(move |s| s.place_window(&root, (x, y)))
        .expect("a frame may be placed on the plane");
}

/// Keep the layout as it stands, as a save does, and tell the seat what came
/// back.
///
/// **This is what the desktop lane does every time the layout changes** — the
/// shell reads the live canvas, the desktop keeps it and answers with what it
/// kept, and the seat is told. Written out here because a test has no desktop
/// loop; `crate::direct_desktop` is where production does it, in three lines
/// that say the same thing.
fn the_layout_is_kept(f: &Fixture, when: SystemTime) {
    f.backend(move |s| {
        let now = s.the_arrangement_now();
        let kept = now.following(s.what_these_places_remember(), when);
        s.these_places_remember(&kept);
    });
}

/// Press the chord this action ships with, through the keyboard road.
fn command(f: &Fixture, action: Action, now: SystemTime) -> Option<Action> {
    let settings = Shortcuts::shipped();
    let chord = settings
        .chord_for(action)
        .expect("every canvas action ships with a chord");
    f.backend(move |s| s.dispatch_canvas_command(&settings, chord, now))
        .expect("a shipped canvas chord is one this shell can carry out")
}

/// **Arrange, change, and go back: the layout that returns is the one that was
/// held.**
///
/// The acceptance's first clause, on real frames. The frame is put somewhere,
/// that state is kept, the frame is moved somewhere else, and the chord puts it
/// back — read off the plane rather than out of an `Arrangement`.
#[test]
fn a_place_goes_back_to_the_layout_that_was_held() {
    let (f, _app) = a_desktop();

    put_the_frame_at(&f, (400, 300));
    the_layout_is_kept(&f, moment(1));
    put_the_frame_at(&f, (900, 120));
    the_layout_is_kept(&f, moment(2));
    assert_eq!(the_frame_is_at(&f), (900, 120), "the frame did not move");

    assert_eq!(
        command(&f, Action::GoBackOnThisPlace, moment(3)),
        Some(Action::GoBackOnThisPlace)
    );
    assert_eq!(
        the_frame_is_at(&f),
        (400, 300),
        "going back did not return the layout that was held"
    );
}

/// **The person is never shown a state they cannot get back from.**
///
/// The acceptance's second clause, and the reason going back is itself a
/// rearrangement: the canvas being left joins the series rather than being
/// dropped. So pressing the chord twice walks backwards through two held states
/// instead of losing the middle one.
#[test]
fn the_canvas_a_person_leaves_is_itself_held() {
    let (f, _app) = a_desktop();

    put_the_frame_at(&f, (100, 100));
    the_layout_is_kept(&f, moment(1));
    put_the_frame_at(&f, (500, 500));
    the_layout_is_kept(&f, moment(2));
    put_the_frame_at(&f, (800, 200));
    the_layout_is_kept(&f, moment(3));

    command(&f, Action::GoBackOnThisPlace, moment(4));
    assert_eq!(the_frame_is_at(&f), (500, 500), "one step back");
    command(&f, Action::GoBackOnThisPlace, moment(5));
    assert_eq!(
        the_frame_is_at(&f),
        (100, 100),
        "a second step back lost the middle state instead of walking to it"
    );
}

/// **A Place with nothing held is not a fault.**
///
/// Pressing it on a canvas nobody has rearranged twice is an ordinary thing to
/// do, exactly as zooming in at the end of the ladder is: nothing moves, the
/// action is answered, and no refusal reaches the person.
#[test]
fn going_back_on_a_place_that_remembers_nothing_moves_nothing() {
    let (f, _app) = a_desktop();
    put_the_frame_at(&f, (260, 240));

    assert!(
        f.backend(|s| s.the_earlier_states_of_this_place().is_empty()),
        "a Place nobody has rearranged twice remembers something"
    );
    assert_eq!(
        command(&f, Action::GoBackOnThisPlace, moment(1)),
        Some(Action::GoBackOnThisPlace),
        "the chord was refused rather than doing nothing"
    );
    assert_eq!(the_frame_is_at(&f), (260, 240), "nothing should have moved");
}

/// **Going back on one Place leaves the others where they are.**
///
/// The shape question task 8 answered from its own title: *a Place remembers
/// time*, so a session-wide series — which would make going back move every
/// Place at once — is the thing this is not.
///
/// **Asked of the series by Place rather than of whichever is in front**, and
/// the first version of this test was wrong for assuming otherwise: it sent the
/// window to the next Place and then asserted about *this* Place, on the belief
/// that moving a window takes the person with it. It does not —
/// `MoveTheWindowToTheNextPlace` sends the window and leaves the person where
/// they are, which is the plan's own distinction between task 3 and task 4, and
/// the assertion was therefore about the Place it had just emptied.
#[test]
fn what_one_place_remembers_is_not_what_another_remembers() {
    let (f, _app) = a_desktop();
    let here = f.backend(|s| s.the_place_now());
    let next = here.next().expect("a canvas has more than one Place");

    put_the_frame_at(&f, (300, 300));
    the_layout_is_kept(&f, moment(1));
    put_the_frame_at(&f, (700, 700));
    the_layout_is_kept(&f, moment(2));

    let (mine, theirs) = f.backend(move |s| {
        let series = s.what_these_places_remember();
        (series.earlier_on(here).len(), series.earlier_on(next).len())
    });
    assert_eq!(mine, 1, "the Place being arranged remembers nothing");
    assert_eq!(
        theirs, 0,
        "a Place nobody has touched carries another Place's series"
    );

    // And the act itself only reaches the Place in front: the window moves back
    // and the other Place still has nothing to go back to.
    command(&f, Action::GoBackOnThisPlace, moment(3));
    assert_eq!(the_frame_is_at(&f), (300, 300));
    assert_eq!(
        f.backend(move |s| s.what_these_places_remember().earlier_on(next).len()),
        0,
        "going back on one Place wrote a series onto another"
    );
}
