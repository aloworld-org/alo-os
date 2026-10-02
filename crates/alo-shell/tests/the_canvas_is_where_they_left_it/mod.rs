//! **The canvas is where they left it** — task 9 of the canvas plan.
//!
//! *Frame positions and the camera survive a session ending and starting again.*
//! `alo_arranging`'s own tests hold the file: that a round trip is exact, and that
//! a hand-edited place no canvas has is refused rather than clamped. What is held
//! here is everything the file cannot know — that the arrangement is read off the
//! frames actually open, that a window comes back where it was, and that an
//! application which did not come back leaves nothing behind.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{Application, Fixture};
use alo_arranging::Arrangement;
use alo_shell::WhereTheyLeftIt;
use smithay::utils::Rectangle;

/// The viewport these tests render to.
const VIEWPORT: (i32, i32) = (1280, 720);

/// A dock along the bottom, as the desktop draws one.
fn a_dock() -> Rectangle<i32, smithay::utils::Physical> {
    Rectangle::new((0, VIEWPORT.1 - 64).into(), (VIEWPORT.0, 64).into())
}

/// A real display with a pointer and an output.
fn fixture() -> Fixture {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    f.render(VIEWPORT, false, 1).expect("an output to look at");
    f
}

/// Offer a window its remembered place, and hand the arrangement back.
///
/// `Fixture::backend` takes a `'static` closure, so what is still needed
/// afterwards travels in and out rather than being borrowed across it.
fn put_back(
    f: &Fixture,
    frame: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    waiting: WhereTheyLeftIt,
) -> (bool, WhereTheyLeftIt) {
    let frame = frame.clone();
    f.backend(move |s| {
        // **The controls are recorded first, as a draw would.**
        //  took the Dock as an argument until 2026-10-02
        // and checked a remembered place against it with the rule the owner
        // replaced. It now asks what a drag asks, of the controls the last draw
        // laid out — so a test that wants the check to apply has to say what
        // was drawn, exactly as the dragging tests do. Without this line the
        // recovery is unchecked, which is the real behaviour before a first
        // frame and would make this test pass for the wrong reason.
        s.the_fixed_controls_were_drawn(
            alo_shell::FixedControlsDrawn {
                dock_band: Some(a_dock()),
                panel_reserved: smithay::utils::Rectangle::default(),
                display_scale: 100,
            },
            alo_appearance::TextScale::ordinary(),
        );
        let mut waiting = waiting;
        let took = s.put_back_where_it_was(&frame, &mut waiting);
        (took, waiting)
    })
}

/// A mapped application that calls itself `app_id`. **Kept alive** by the caller:
/// dropping it disconnects the client and unmaps the frame.
fn mapped_as(f: &Fixture, app_id: &str) -> Application {
    let mut app = Application::new(f);
    app.toplevel.set_app_id(app_id.to_owned());
    app.configure();
    app.attach();
    app.sync();
    app
}

/// **What is written down is what is open**, read off the frames themselves.
///
/// There is no second record of where a window is: the arrangement is taken from
/// `the_frames_on_the_plane` and the camera actually looked through, so it cannot
/// drift from what `scene::trees` draws.
#[test]
fn the_arrangement_written_down_is_the_one_on_the_screen() {
    let f = fixture();
    let _notes = mapped_as(&f, "org.alo.Notes");
    let _ledger = mapped_as(&f, "org.alo.Ledger");

    let frame = f
        .backend(|s| s.mapped_surfaces().next().cloned())
        .expect("a frame is mapped");
    {
        let frame = frame.clone();
        assert!(f.backend(move |s| s.place_window(&frame, (300, 200)).is_ok()));
    }
    assert!(f.backend(|s| s.pan_the_canvas(-40, 25)).is_some());

    let arrangement = f.backend(|s| s.the_arrangement_now());
    assert_eq!(arrangement.how_many(), 2, "two windows are open");
    assert_eq!(
        arrangement
            .camera_on(alo_canvas::Place::FIRST)
            .expect("the Place was arranged"),
        f.backend(|s| s.the_camera()),
        "the arrangement remembered a different camera from the one being looked through"
    );
    let (at, _) = arrangement
        .where_it_was(alo_canvas::Place::FIRST, "org.alo.Notes")
        .expect("the first window names itself");
    assert_eq!(
        (at.x, at.y),
        (300, 200),
        "a window was written down somewhere else"
    );
}

/// **A window comes back where it was**, and only the first of its application does.
///
/// One place per `app_id` is what the file can hold, so a second window of the same
/// application is placed as any new window is rather than landing on top of the
/// first. Stated in `alo_arranging`'s header and asserted here, because it is the
/// kind of consequence that is discovered by somebody opening two terminals.
#[test]
fn a_window_comes_back_where_it_was_and_the_second_does_not_take_its_place() {
    let f = fixture();
    let mut left = Arrangement::fresh();
    left.window_was(
        alo_canvas::Place::FIRST,
        "org.alo.Notes",
        alo_canvas::At::checked(420, 260).expect("a place on the plane"),
        alo_canvas::Size::checked(800, 600).expect("a size a frame may be"),
    );
    let waiting = WhereTheyLeftIt::from(left);

    let _first = mapped_as(&f, "org.alo.Notes");
    let first = f
        .backend(|s| s.mapped_surfaces().next().cloned())
        .expect("a frame is mapped");
    let (took, waiting) = put_back(&f, &first, waiting);
    assert!(
        took,
        "a window whose application was remembered did not get its place back"
    );
    let arrangement = f.backend(|s| s.the_arrangement_now());
    let (at, _) = arrangement
        .where_it_was(alo_canvas::Place::FIRST, "org.alo.Notes")
        .expect("it is open");
    assert_eq!(
        (at.x, at.y),
        (420, 260),
        "it did not come back where it was left"
    );

    // A second window of the same application finds the place already claimed.
    let _second = mapped_as(&f, "org.alo.Notes");
    let second = f
        .backend(|s| s.mapped_surfaces().last().cloned())
        .expect("two frames are mapped");
    let (took_again, _) = put_back(&f, &second, waiting);
    assert!(
        !took_again,
        "a second window of one application took a place that was already claimed"
    );
}

/// **An application that did not come back is absent, not drawn empty.**
///
/// The second half of task 9's acceptance. The arrangement is a list of places
/// waiting to be claimed: nothing reserves room for a window that never opened,
/// nothing draws it, and the canvas holds only what is actually there.
#[test]
fn an_application_that_did_not_come_back_leaves_nothing_behind() {
    let f = fixture();
    let mut left = Arrangement::fresh();
    for (app_id, x) in [("org.alo.Notes", 100), ("org.alo.Gone", 900)] {
        left.window_was(
            alo_canvas::Place::FIRST,
            app_id,
            alo_canvas::At::checked(x, 100).expect("a place on the plane"),
            alo_canvas::Size::checked(400, 300).expect("a size a frame may be"),
        );
    }
    let waiting = WhereTheyLeftIt::from(left);

    // Only one of the two applications comes back.
    let _notes = mapped_as(&f, "org.alo.Notes");
    let notes = f
        .backend(|s| s.mapped_surfaces().next().cloned())
        .expect("a frame is mapped");
    let (took, _) = put_back(&f, &notes, waiting);
    assert!(
        took,
        "the one application that came back did not get its place"
    );

    let arrangement = f.backend(|s| s.the_arrangement_now());
    assert_eq!(
        arrangement.how_many(),
        1,
        "the canvas holds a window for an application that never came back"
    );
    assert_eq!(
        arrangement.where_it_was(alo_canvas::Place::FIRST, "org.alo.Gone"),
        None,
        "an application that did not come back was still written down"
    );
    assert_eq!(
        f.backend(|s| s.the_frames_on_the_plane().len()),
        1,
        "a frame was drawn for an application that never came back"
    );
}

/// **A window that names itself nothing is not remembered.**
///
/// A place is keyed by `app_id`, and an application that gives none has nothing to
/// be remembered by. It is placed as any new window is rather than being given
/// somebody else's place, which is what keying on a fallback would have done.
#[test]
fn a_window_with_no_name_of_its_own_is_not_remembered() {
    let f = fixture();
    let mut app = Application::new(&f);
    app.configure();
    app.attach();
    app.sync();

    let frame = f
        .backend(|s| s.mapped_surfaces().next().cloned())
        .expect("a frame is mapped");
    assert_eq!(
        f.backend(move |s| s.the_app_id_of(&frame)),
        None,
        "a window that set no app id was given one"
    );
    assert_eq!(
        f.backend(|s| s.the_arrangement_now().how_many()),
        0,
        "a window with no name of its own was written into the arrangement"
    );
}

/// **An application that claimed its place on one Place can still claim on
/// another.**
///
/// Offer a window a remembered place at `y`, and say whether it took it.
fn offered_a_place_at(y: i32) -> bool {
    let f = fixture();
    let mut left = Arrangement::fresh();
    left.window_was(
        alo_canvas::Place::FIRST,
        "org.alo.Notes",
        alo_canvas::At::checked(420, y).expect("a place on the plane"),
        alo_canvas::Size::checked(800, 600).expect("a size a frame may be"),
    );
    let _app = mapped_as(&f, "org.alo.Notes");
    let frame = f
        .backend(|s| s.mapped_surfaces().next().cloned())
        .expect("a frame is mapped");
    put_back(&f, &frame, WhereTheyLeftIt::from(left)).0
}

/// **Recovery is held to the rule that replaced the old one, and until
/// 2026-10-02 it was not.**
///
/// `put_back_where_it_was` asked `as_far_as_a_frame_may_be_dragged`, which asks
/// `the_name_would_be_under` — *is the name **entirely** inside this one
/// rectangle* — against the Dock's band alone. **That is the rule the owner
/// replaced on 2026-09-30**, for the reason `A_USABLE_HANDLE` records: *one
/// exposed pixel is technically reachable and practically lost.* Dragging moved
/// to the new rule and to the whole set of controls; recovery did not, so the
/// superseded rule still had a caller.
///
/// The Dock's band here starts at 656 and a name band is 48 tall, so a frame
/// whose top is at 690 leaves **14 pixels** of name clear. The old rule allows
/// that — 14 pixels is not *entirely* under anything. The new rule refuses it,
/// because 14 is less than the 24-pixel floor.
///
/// Asserted against a place that **is** allowed, because a test that only showed
/// a refusal would pass for a rule that refuses everything and for a recovery
/// that had stopped working at all.
#[test]
fn a_remembered_place_that_leaves_too_little_name_is_refused() {
    let barely_clear = offered_a_place_at(690);
    let plainly_clear = offered_a_place_at(600);

    assert!(
        plainly_clear,
        "a remembered place well clear of the Dock was refused, so recovery is \
         refusing more than the rule asks or has stopped working"
    );
    assert!(
        !barely_clear,
        "a remembered place leaving 14 pixels of name clear of the Dock was \
         accepted: the old entirely-under rule allows that and the rule the \
         owner replaced it with does not, so recovery is still asking the \
         superseded question"
    );
}

/// Task 5 of `the-canvas-and-its-places.md`: *position, size, camera and the
/// panel's own state, **per Place rather than per session***. This is the one
/// field that would have refused it quietly.
///
/// `WhereTheyLeftIt::claimed` was a `Vec<String>` of `app_id`s, which was right
/// while an arrangement held one map: one application had one remembered place,
/// so claiming it once was claiming it everywhere. With a place per Place the
/// same list means an application that came back on one surface is **refused on
/// every other** — and refused silently, because `put_back_where_it_was` answers
/// `false` for *already claimed* exactly as it does for *nothing remembered*.
///
/// So this asserts the pair, not the single: the same `app_id` is offered its
/// place on two Places and takes both.
#[test]
fn claiming_a_place_on_one_place_does_not_claim_it_on_another() {
    let f = fixture();
    let elsewhere = alo_canvas::Place::FIRST
        .next()
        .expect("there is a second place");

    let mut left = Arrangement::fresh();
    left.window_was(
        alo_canvas::Place::FIRST,
        "org.alo.Notes",
        alo_canvas::At::checked(100, 100).expect("a place on the plane"),
        alo_canvas::Size::checked(400, 300).expect("a size a frame may be"),
    );
    left.window_was(
        elsewhere,
        "org.alo.Notes",
        alo_canvas::At::checked(700, 500).expect("a place on the plane"),
        alo_canvas::Size::checked(400, 300).expect("a size a frame may be"),
    );
    let waiting = WhereTheyLeftIt::from(left);

    // The first window opens on the Place a fresh machine looks at.
    let _here = mapped_as(&f, "org.alo.Notes");
    let first = f
        .backend(|s| s.mapped_surfaces().last().cloned())
        .expect("one frame is mapped");
    let (took_here, waiting) = put_back(&f, &first, waiting);
    assert!(took_here, "a window did not take its remembered place");

    // A second window of the same application, moved to another Place before it
    // is offered one. Moving is the primitive, not the gesture: task 3 owns the
    // road a person takes and this test only needs the window to be elsewhere.
    let _there = mapped_as(&f, "org.alo.Notes");
    let second = f
        .backend(|s| s.mapped_surfaces().last().cloned())
        .expect("two frames are mapped");
    f.backend({
        let second = second.clone();
        move |s| s.move_the_window_to(&second, elsewhere)
    });

    let (took_there, _) = put_back(&f, &second, waiting);
    assert!(
        took_there,
        "an application that claimed its place on one Place was refused on another, \
         which is the per-session claim the per-Place arrangement replaced"
    );
}
