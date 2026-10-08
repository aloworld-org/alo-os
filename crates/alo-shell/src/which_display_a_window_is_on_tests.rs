//! Which display a window is on, including the cases a lookup cannot answer.

use super::*;
use crate::screens_testing::a_cold_evening;
use alo_displays::{Reported, Socket};

/// A screen of this many pixels, at this glass size, in this socket.
fn a_screen(socket: &str, pixels: (u32, u32), millimetres: (u32, u32)) -> Reported {
    Reported::of(
        Socket::named(socket).expect("a socket"),
        None,
        pixels,
        Some(millimetres),
    )
    .expect("a screen")
}

/// A server holding the arrangement these screens make.
fn server_showing(reported: Vec<Reported>) -> (tempfile::TempDir, Server) {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind(directory.path(), "which-display").expect("a bound socket");
    server.these_screens_are(Some(crate::screens_testing::the_screens(
        reported,
        &alo_displays::Changes::untouched(),
        &alo_appearance::Appearance::shipped(),
        &a_cold_evening(),
    )));
    (directory, server)
}

/// A window of this size at this corner.
fn a_window(at: (i32, i32), size: (i32, i32)) -> Rectangle<i32, Physical> {
    Rectangle::new(at.into(), size.into())
}

/// Two 1080p screens side by side, both one to one, so the desk is 3840 wide.
fn two_side_by_side() -> Vec<Reported> {
    vec![
        a_screen("DP-1", (1920, 1080), (597, 336)),
        a_screen("DP-2", (1920, 1080), (597, 336)),
    ]
}

/// **A window on the second display is answered as the second display**, not
/// as the main one it is merely measured from.
///
/// The case the whole file exists for: before this, every window was on every
/// display, so this question had no answer to give and 5b, 6 and 7 each
/// needed one.
#[test]
fn a_window_past_the_first_display_is_on_the_second() {
    let (_directory, server) = server_showing(two_side_by_side());
    let places: Vec<_> = server
        .the_screens()
        .expect("an arrangement")
        .each()
        .map(|place| (place.at().across(), place.room().across_and_along().0))
        .collect();
    // The arrangement decides where the second screen sits; this test asks
    // about a point inside it rather than assuming 1920.
    let (second_at, second_across) = places
        .iter()
        .copied()
        .find(|(at, _)| *at != 0)
        .expect("a second screen somewhere other than the origin");
    let well_inside = second_at + second_across / 2;

    let on = server
        .the_display_a_window_is_on(a_window((well_inside, 10), (100, 100)))
        .expect("a window inside a display is on it");

    assert_eq!(
        on.at().across(),
        second_at,
        "a window on the second display was answered as another"
    );
}

/// **A window entirely on the first display is on the first.**
#[test]
fn a_window_on_the_first_display_is_on_the_first() {
    let (_directory, server) = server_showing(two_side_by_side());
    let on = server
        .the_display_a_window_is_on(a_window((10, 10), (100, 100)))
        .expect("a window inside a display is on it");
    assert_eq!(on.at().across(), 0);
}

/// **A window straddling two displays belongs to the one showing most of it.**
///
/// Not the one it starts on, which is the answer a corner lookup would give
/// and the one a person looking at the screen would disagree with.
#[test]
fn a_straddling_window_belongs_where_most_of_it_is() {
    let (_directory, server) = server_showing(two_side_by_side());
    let boundary = server
        .the_screens()
        .expect("an arrangement")
        .each()
        .map(|place| place.at().across())
        .find(|across| *across != 0)
        .expect("a second screen");

    // Ten units on the first display, ninety on the second.
    let mostly_second = a_window((boundary - 10, 10), (100, 100));
    let on = server
        .the_display_a_window_is_on(mostly_second)
        .expect("a straddling window is still on a display");

    assert_eq!(
        on.at().across(),
        boundary,
        "a window mostly on the second display was given to the first"
    );
}

/// **A window on no display at all is no display**, rather than the main one.
///
/// A window dragged off the desk is somewhere the person put it. Answering
/// *the main screen* would move it in the only sense this question has.
#[test]
fn a_window_off_the_desk_is_on_no_display() {
    let (_directory, server) = server_showing(two_side_by_side());
    assert!(
        server
            .the_display_a_window_is_on(a_window((-5000, -5000), (10, 10)))
            .is_none(),
        "a window off the desk was given a display"
    );
}

/// **A window that only touches a display's edge is not on it.**
///
/// Zero area is nothing shown, and an off-by-one here would put a window on a
/// display that cannot see a pixel of it.
#[test]
fn a_window_touching_an_edge_shows_nothing_on_it() {
    let (_directory, server) = server_showing(two_side_by_side());
    let boundary = server
        .the_screens()
        .expect("an arrangement")
        .each()
        .map(|place| place.at().across())
        .find(|across| *across != 0)
        .expect("a second screen");

    // Ends exactly where the second display begins.
    let on = server
        .the_display_a_window_is_on(a_window((boundary - 100, 10), (100, 100)))
        .expect("a window inside the first display is on it");
    assert_eq!(
        on.at().across(),
        0,
        "a window merely touching the second display was placed on it"
    );
}

/// **A session with no arrangement answers nothing**, which is every session
/// that was never given a display model.
#[test]
fn no_arrangement_answers_no_display() {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(directory.path(), "no-screens").expect("a bound socket");
    assert!(server.the_screens().is_none());
    assert!(
        server
            .the_display_a_window_is_on(a_window((10, 10), (100, 100)))
            .is_none()
    );
}

/// A display that draws, and says which one it is.
///
/// Needed since 2026-10-08: the per-display controls safeguard asks how many
/// displays this session has **drawn**, not how many sets of controls were
/// recorded, so a test about two displays has to have two displays draw. The
/// earlier version of the test below recorded controls for two names without
/// either drawing, which is not a two-display session — and it passed only
/// because the safeguard was counting the wrong thing.
struct ADisplay {
    /// Its connector name, which is what the controls are keyed by.
    named: &'static str,
}

impl crate::FrameTarget for ADisplay {
    fn size(&self) -> smithay::utils::Size<i32, Physical> {
        (1920, 1080).into()
    }
    fn submit(
        &mut self,
        _: &[smithay::reexports::wayland_server::protocol::wl_surface::WlSurface],
    ) -> Result<
        Vec<smithay::reexports::wayland_server::protocol::wl_surface::WlSurface>,
        crate::RenderError,
    > {
        Ok(vec![])
    }
    fn metadata(&self) -> Result<crate::OutputMetadata, crate::RenderError> {
        Ok(crate::OutputMetadata {
            name: self.named.to_owned(),
            make: "alo".to_owned(),
            model: "a monitor".to_owned(),
            physical_size: (310, 170),
            refresh: 60_000,
        })
    }
}

/// Controls with a dock band of this height along the bottom of a display
/// that is `across` wide and `down` tall, with its corner at `at`.
fn a_dock_on(at: (i32, i32), across: i32, down: i32) -> crate::FixedControlsDrawn {
    crate::FixedControlsDrawn {
        dock_band: Some(Rectangle::new(
            (at.0, at.1 + down - 60).into(),
            (across, 60).into(),
        )),
        panel_reserved: Rectangle::new((0, 0).into(), (0, 0).into()),
        what_is_leaving: None,
        top_controls: None,
    }
}

/// **A frame is held to the controls of the display it is on**, which is task
/// 5b's acceptance and the reason the per-display store exists.
///
/// Before this, one store held whichever display drew last, so a frame on the
/// second display was kept clear of the *first* display's dock — a rectangle
/// nothing occupies where that frame is — and was free to sit under the dock
/// that is actually over it.
#[test]
fn a_frame_is_held_to_the_controls_of_its_own_display() {
    let (_directory, mut server) = server_showing(two_side_by_side());
    let text = alo_appearance::TextScale::ordinary();
    // **Both displays draw first**, so this is a session with two displays
    // rather than a session with two sets of controls. The safeguard asks the
    // first question.
    let mut first = ADisplay { named: "DP-1" };
    let mut second = ADisplay { named: "DP-2" };
    let drawn = server.render_each_display(&mut [&mut first, &mut second], 1);
    assert_eq!(
        drawn.drawn().len(),
        2,
        "both displays have to draw for this to be a two-display session"
    );
    let second_at = server
        .the_screens()
        .expect("an arrangement")
        .each()
        .map(|place| place.at().across())
        .find(|across| *across != 0)
        .expect("a second screen");

    server.the_fixed_controls_were_drawn("DP-1", a_dock_on((0, 0), 1920, 1080), text);
    server.the_fixed_controls_were_drawn("DP-2", a_dock_on((second_at, 0), 1920, 1080), text);

    let on_the_first = server.the_fixed_controls_on_the_display_for(a_window((10, 10), (100, 100)));
    let on_the_second =
        server.the_fixed_controls_on_the_display_for(a_window((second_at + 10, 10), (100, 100)));

    let first_dock = on_the_first.first().copied().expect("the first dock");
    let second_dock = on_the_second.first().copied().expect("the second dock");
    assert_eq!(first_dock.loc.x, 0, "the first display's dock moved");
    assert_eq!(
        second_dock.loc.x, second_at,
        "a frame on the second display was held to the first display's dock"
    );
    assert_ne!(
        first_dock, second_dock,
        "both displays answered with one set of controls"
    );
}

/// **One display answers exactly as it always did**, whatever the window.
///
/// The safeguard that keeps every machine this lane can actually test
/// unchanged: with one display there is no choice to make, so the single
/// entry is the answer even for a window the arrangement would place nowhere.
#[test]
fn one_display_answers_as_it_always_did() {
    let (_directory, mut server) = server_showing(vec![a_screen("DP-1", (1920, 1080), (597, 336))]);
    server.the_fixed_controls_were_drawn(
        "DP-1",
        a_dock_on((0, 0), 1920, 1080),
        alo_appearance::TextScale::ordinary(),
    );

    let far_away = server.the_fixed_controls_on_the_display_for(a_window((-9000, -9000), (10, 10)));

    assert_eq!(
        far_away.len(),
        1,
        "the one display stopped answering for a window off the desk"
    );
}

/// **A display that drew without recording its controls is held to nothing,
/// not to another display's furniture.**
///
/// This is the test that would have caught `more-than-one-display-plan.md`
/// task 9's defect, and it is written from it rather than from the fix.
///
/// `Desk::present` recorded only the display the draw loop held, so on a
/// two-display machine one set of controls existed and the safeguard —
/// `fixed_controls.len() <= 1`, meaning *one set was recorded* — handed the
/// first display's dock to every window on the second. A safeguard written to
/// protect a single display was doing the opposite, and it hid the defect for
/// two days.
///
/// The condition is now *how many displays have drawn*, which `presentations`
/// answers. So the unrecorded display gets `None` — the same answer this gives
/// before any frame, because nothing has been drawn **there** yet — and a
/// future change that forgets a display again fails to nothing rather than to
/// somewhere wrong.
#[test]
fn a_display_whose_controls_were_never_recorded_is_held_to_nothing() {
    let (_directory, mut server) = server_showing(two_side_by_side());
    let mut first = ADisplay { named: "DP-1" };
    let mut second = ADisplay { named: "DP-2" };
    assert_eq!(
        server
            .render_each_display(&mut [&mut first, &mut second], 1)
            .drawn()
            .len(),
        2
    );
    let second_at = server
        .the_screens()
        .expect("an arrangement")
        .each()
        .map(|place| place.at().across())
        .find(|across| *across != 0)
        .expect("a second screen");

    // Only the first display records anything — the defect, reproduced.
    server.the_fixed_controls_were_drawn(
        "DP-1",
        a_dock_on((0, 0), 1920, 1080),
        alo_appearance::TextScale::ordinary(),
    );

    let on_the_first = server.the_fixed_controls_on_the_display_for(a_window((10, 10), (100, 100)));
    let on_the_second =
        server.the_fixed_controls_on_the_display_for(a_window((second_at + 10, 10), (100, 100)));

    assert_eq!(
        on_the_first.len(),
        1,
        "the display that recorded its controls stopped answering with them"
    );
    assert!(
        on_the_second.is_empty(),
        "a window on the display that recorded nothing was held to the other \
         display's dock, which is the fault this test exists for"
    );
}
