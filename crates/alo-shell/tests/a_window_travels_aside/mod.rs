//! **A window the person puts aside travels to its card in the panel**, on
//! real client connections, through the road the Minimise control, the keys
//! in the band and the Minimise chord all take: `the_person_asked_to_put_aside`
//! and then `put_aside_what_was_asked_for`, as the frame loop meets them.
//!
//! `docs/design/the-window-controls.md`: *Minimise — puts this window aside,
//! travelling toward the panel of put-aside windows on the right; restoring
//! returns it to its Place and its size.* Each case has its refusal beside it:
//! less motion, less motion asked for mid-travel, no panel, no card, a client
//! that minimised itself, and a window brought back mid-travel. The travel's
//! arithmetic — start, middle, end, eased, 300 ms however far — is
//! `a_window_travels_aside_tests`'; what is under test here is that the
//! person's road starts one, the frame's step ends one, and every refusal
//! leaves the window put aside exactly as before.
//!
//! **Not the pointer's half of a panel click.** Which card a press landed on
//! is `which_preview_the_pointer_is_on`'s, tested there against real draws;
//! the restore below goes on from the window that click names, by the calls
//! `bring_back_what_was_clicked` makes.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, Instant};

use super::support::{Application, Fixture};
use alo_put_aside::Panel;
use alo_shell::{DesktopLook, TheCards};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

/// The design's own frame, used as *a* display rather than *the* display.
const A_DISPLAY: (i32, i32) = (1440, 960);

/// How long a travel lasts: the view glide's 300 ms.
const IT_LASTS: Duration = Duration::from_millis(300);

/// A mapped application that names itself, as the other client tests make one.
fn mapped(f: &Fixture, title: &str) -> Application {
    let mut app = Application::new(f);
    app.toplevel.set_app_id("Docs".to_owned());
    app.toplevel.set_title(title.to_owned());
    app.configure();
    app.attach();
    app.sync();
    app
}

/// The ordinary desktop at midday, read left to right.
fn a_look() -> DesktopLook {
    DesktopLook::of(
        &alo_appearance::Appearance::shipped(),
        &alo_access::TurnedOn::nothing(),
        alo_appearance::TimeOfDay::checked(12, 0).expect("midday"),
        alo_strings::Direction::LeftToRight,
        &alo_appearance::DisplayId::named("a-display").expect("a display's name"),
    )
}

/// The person puts `root` aside by their own road, into `panel`. Answers how
/// many were put aside and how many began to travel.
fn the_person_puts_it_aside(f: &Fixture, root: &WlSurface, panel: Panel) -> (Panel, usize, usize) {
    let root = root.clone();
    f.backend(move |s| {
        let mut panel = panel;
        s.the_person_asked_to_put_aside(&root);
        let put = s.put_aside_what_was_asked_for(Some(&mut panel));
        (panel, put, s.how_many_windows_are_travelling_aside())
    })
}

/// Whether `root` is hidden — off the canvas, and so out of every hit test.
fn hidden(f: &Fixture, root: &WlSurface) -> bool {
    let root = root.clone();
    f.backend(move |s| {
        s.minimized_surfaces().any(|it| *it == root) && !s.mapped_surfaces().any(|it| *it == root)
    })
}

/// **The person's Minimise: the window is gone at once, an outline travels
/// from it toward its card, taking on its size, and the travel ends exactly on the card
/// and is then gone.**
#[test]
fn the_persons_minimise_travels_to_its_card_and_ends_there() {
    let f = Fixture::keyboard();
    let _app = mapped(&f, "Launch strategy");
    let root = f.root();
    let window = alo_shell::where_a_window_is(&root);
    let (panel, put, travelling) = the_person_puts_it_aside(&f, &root, Panel::new());
    let after = Instant::now();
    assert_eq!(put, 1, "the control: the window was put aside");
    assert_eq!(travelling, 1, "the window did not begin to travel");
    assert!(
        hidden(&f, &root),
        "the window is still on the canvas while it travels"
    );

    let cards = TheCards::as_revealed(&panel, a_look(), A_DISPLAY, 100);
    let id = panel.previews().first().map(alo_put_aside::Preview::window);
    let card = cards.of(id.expect("the panel holds it")).expect("its card");
    let (first, last, gone, left) = f.backend(move |s| {
        let first = s.the_travels_aside_move_on(Instant::now(), &cards);
        let last = s.the_travels_aside_move_on(after + IT_LASTS, &cards);
        let left = s.how_many_windows_are_travelling_aside();
        let gone = s.the_travels_aside_move_on(after + IT_LASTS + IT_LASTS / 2, &cards);
        (first, last, gone, left)
    });
    let first = first
        .first()
        .copied()
        .expect("the first frame drew an outline");
    // Between the two sizes, whichever is larger: the fixture's window is 16
    // square and its card 48, so this outline grows onto its card.
    let between = |it: i32, a: i32, b: i32| a.min(b) <= it && it <= a.max(b);
    assert!(
        between(first.size.w, window.size.w, card.size.w)
            && between(first.size.h, window.size.h, card.size.h),
        "the outline is not between the window's size and its card's: {first:?}"
    );
    assert_eq!(
        last,
        vec![card],
        "the travel did not end exactly on the card"
    );
    assert_eq!(left, 0, "the travel stayed after it arrived");
    assert!(
        gone.is_empty(),
        "an outline was drawn after the travel arrived"
    );
    assert_eq!(panel.holding(), 1, "the panel does not hold it");
}

/// **Several put aside in a row each travel**, each to its own card.
#[test]
fn several_put_aside_in_a_row_each_travel_to_their_own_card() {
    let f = Fixture::keyboard();
    let _first = mapped(&f, "One");
    let _second = mapped(&f, "Two");
    let roots: Vec<WlSurface> = f.backend(|s| s.mapped_surfaces().cloned().collect());
    assert_eq!(roots.len(), 2, "the control: two windows");
    let (panel, put, travelling) = f.backend(move |s| {
        let mut panel = Panel::new();
        for root in &roots {
            s.the_person_asked_to_put_aside(root);
        }
        let put = s.put_aside_what_was_asked_for(Some(&mut panel));
        (panel, put, s.how_many_windows_are_travelling_aside())
    });
    assert_eq!((put, travelling), (2, 2));
    let cards = TheCards::as_revealed(&panel, a_look(), A_DISPLAY, 100);
    let ends = f.backend(move |s| s.the_travels_aside_move_on(Instant::now() + IT_LASTS, &cards));
    assert_eq!(ends.len(), 2, "both did not land");
    assert_ne!(ends.first(), ends.get(1), "two windows landed on one card");
}

/// **The refusal: with less motion asked for, nothing travels** — the window
/// is simply put aside, as it always was.
#[test]
fn with_less_motion_nothing_travels_and_the_window_is_put_aside() {
    let f = Fixture::keyboard();
    let _app = mapped(&f, "Launch strategy");
    let root = f.root();
    f.backend(|s| s.the_motion_is_reduced(true));
    let (panel, put, travelling) = the_person_puts_it_aside(&f, &root, Panel::new());
    assert_eq!(travelling, 0, "it travelled with less motion asked for");
    assert_eq!((put, panel.holding()), (1, 1), "it was not put aside");
    assert!(hidden(&f, &root), "it is still on the canvas");
}

/// **The refusal: less motion asked for mid-travel ends the travel at once.**
#[test]
fn less_motion_asked_for_mid_travel_ends_it() {
    let f = Fixture::keyboard();
    let _app = mapped(&f, "Launch strategy");
    let root = f.root();
    let (panel, _, travelling) = the_person_puts_it_aside(&f, &root, Panel::new());
    assert_eq!(travelling, 1, "the control: it travels");
    let cards = TheCards::as_revealed(&panel, a_look(), A_DISPLAY, 100);
    let (left, drawn) = f.backend(move |s| {
        s.the_motion_is_reduced(true);
        (
            s.how_many_windows_are_travelling_aside(),
            s.the_travels_aside_move_on(Instant::now(), &cards),
        )
    });
    assert_eq!(left, 0, "it went on travelling");
    assert!(
        drawn.is_empty(),
        "an outline was drawn after less motion was asked for"
    );
    assert_eq!(panel.holding(), 1);
}

/// **The refusal: no panel, nothing put aside and nothing travels.**
#[test]
fn with_no_panel_nothing_travels() {
    let f = Fixture::keyboard();
    let _app = mapped(&f, "Launch strategy");
    let root = f.root();
    let (put, travelling) = f.backend(move |s| {
        s.the_person_asked_to_put_aside(&root);
        (
            s.put_aside_what_was_asked_for(None),
            s.how_many_windows_are_travelling_aside(),
        )
    });
    assert_eq!((put, travelling), (0, 0));
}

/// **The refusal: no card to travel to, nothing travels** — a display with no
/// room for the panel's column — and the window is still put aside.
#[test]
fn with_no_card_nothing_travels_and_the_window_is_still_put_aside() {
    let f = Fixture::keyboard();
    let _app = mapped(&f, "Launch strategy");
    let root = f.root();
    let (panel, put, _) = the_person_puts_it_aside(&f, &root, Panel::new());
    // 100 wide: the panel's column is 112, so it has no room and no cards.
    let cards = TheCards::as_revealed(&panel, a_look(), (100, 960), 100);
    let (drawn, left) = f.backend(move |s| {
        (
            s.the_travels_aside_move_on(Instant::now(), &cards),
            s.how_many_windows_are_travelling_aside(),
        )
    });
    assert!(drawn.is_empty(), "an outline was drawn with no card");
    assert_eq!(left, 0, "a travel with no card was kept");
    assert_eq!((put, panel.holding()), (1, 1), "it was not put aside");
    assert!(hidden(&f, &root));
}

/// **The refusal: a client that minimises itself makes no card and no travel.**
#[test]
fn a_client_minimising_itself_does_not_travel() {
    let f = Fixture::keyboard();
    let mut app = mapped(&f, "Launch strategy");
    let root = f.root();
    app.toplevel.set_minimized();
    app.sync();
    assert!(
        hidden(&f, &root),
        "the control: the client's request hid it"
    );
    assert_eq!(f.backend(|s| s.how_many_windows_are_travelling_aside()), 0);
}

/// **A window brought back mid-travel stops travelling.**
#[test]
fn a_window_brought_back_mid_travel_stops_travelling() {
    let f = Fixture::keyboard();
    let _app = mapped(&f, "Launch strategy");
    let root = f.root();
    let (panel, _, travelling) = the_person_puts_it_aside(&f, &root, Panel::new());
    assert_eq!(travelling, 1, "the control: it travels");
    let (back, left) = f.backend(move |s| {
        let mut panel = panel;
        let back = panel
            .previews()
            .first()
            .map(alo_put_aside::Preview::window)
            .and_then(|id| s.the_put_aside_window_numbered(id))
            .map(|surface| s.bring_this_window_back(&mut panel, &surface).is_ok());
        (back, s.how_many_windows_are_travelling_aside())
    });
    assert_eq!(back, Some(true), "the control: it came back");
    assert_eq!(left, 0, "it went on travelling after it came back");
}

/// **Restoring returns it to its Place and its size**: put aside by the
/// person's road into a real panel, brought back by the calls a click on its
/// card makes, and it is on the same Place, at the same origin, the same size,
/// and the panel no longer holds it.
#[test]
fn restoring_from_the_panel_returns_it_to_its_place_and_its_size() {
    let f = Fixture::keyboard();
    let _app = mapped(&f, "Launch strategy");
    let root = f.root();
    let was = {
        let root = root.clone();
        f.backend(move |s| {
            (
                s.the_place_of_the_window(&root),
                alo_shell::window_buffer_origin(&root),
                alo_shell::where_a_window_is(&root).size,
            )
        })
    };
    assert!(was.0.is_some(), "the control: the window is on a Place");
    let (panel, put, _) = the_person_puts_it_aside(&f, &root, Panel::new());
    assert_eq!(put, 1, "the control: it was put aside");
    let (came_back, is, holding) = {
        let root = root.clone();
        f.backend(move |s| {
            let mut panel = panel;
            let id = panel.previews().first().map(alo_put_aside::Preview::window);
            let came_back = id
                .and_then(|id| s.the_put_aside_window_numbered(id))
                .is_some_and(|surface| s.bring_this_window_back(&mut panel, &surface).is_ok());
            (
                came_back,
                (
                    s.the_place_of_the_window(&root),
                    alo_shell::window_buffer_origin(&root),
                    alo_shell::where_a_window_is(&root).size,
                ),
                panel.holding(),
            )
        })
    };
    assert!(came_back, "the window named by its card did not come back");
    assert_eq!(is, was, "it came back to another Place, origin or size");
    assert_eq!(holding, 0, "the panel still holds it");
    assert!(!hidden(&f, &root), "it is not on the canvas again");
}
