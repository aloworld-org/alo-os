//! What `crate::dock_windows` assumes about `alo-dock`, held as tests.
//!
//! **These do not build a `Server`.** `the_windows_the_dock_sees` reads mapped
//! surfaces, which needs a real compositor and a real client; that road is the
//! nested fixtures', and `examples/a_real_application.rs` walks it on a machine
//! with a Wayland parent.
//!
//! What is held here is the thing a unit test can hold and the fixture cannot:
//! **the assumptions the builder rests on.** Each of these would compile, pass
//! every other test, and make the Dock quietly wrong if `alo-dock` changed
//! under it.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err or None is the failure being reported"
)]

use alo_dock::{AppId, Holding, HowItSits, Patch, Spot, Window, WindowId, Windows};

/// A window of `app`, numbered `number`.
fn window(number: u64, app: Option<&str>) -> Window {
    let patch = Patch::of(Spot::at(0, 0), 1, 1).expect("one by one is a patch");
    Window::of(
        WindowId::numbered(number),
        app.map(|app| AppId::named(app).expect("a named application")),
        "",
        patch,
        HowItSits::OnTheCanvas,
    )
}

/// **The Dock's own order is the order things opened**, which is the order
/// `mapped_surfaces` hands them over in, which is why the builder may replay
/// them straight through.
///
/// `windows.rs` says it in as many words — *where an icon sits is a fact a
/// person learns once, so it follows when the application arrived and never
/// what they have been using* — and `Holding::showing` iterates
/// `each_as_opened`. If that ever became the recency order, every icon would
/// move whenever somebody clicked something.
#[test]
fn the_dock_reads_windows_in_the_order_they_opened() {
    let mut windows = Windows::none();
    for number in 0..4 {
        windows.opened(window(number, Some("an.app")));
    }
    let numbers: Vec<u64> = windows
        .each_as_opened()
        .map(|window| window.id().number())
        .collect();
    assert_eq!(
        numbers,
        vec![0, 1, 2, 3],
        "the Dock's order is not creation order"
    );
}

/// **And the recency order is the reverse**, which is the thing the builder
/// cannot be right about and says so in its header.
///
/// Replaying mapped surfaces in creation order leaves the *newest* window at
/// the front of the most-recently-used list. Nothing in the shell records when
/// a person last used a window, so this is not a bug to fix here — it is a
/// false answer waiting for a caller, and this test is what makes it visible
/// rather than discovered by somebody with two windows open.
#[test]
fn most_recent_is_most_recently_opened_and_that_is_not_the_same_fact() {
    let mut windows = Windows::none();
    for number in 0..4 {
        windows.opened(window(number, Some("an.app")));
    }
    let app = AppId::named("an.app").expect("a named application");
    let recent = windows
        .most_recent_of(&app)
        .expect("the application has windows")
        .id()
        .number();
    assert_eq!(
        recent, 3,
        "replaying in creation order should leave the newest looking most recent"
    );
    // And `used` is the call that would make it true, with nothing to call it.
    assert!(windows.used(WindowId::numbered(1)), "window 1 is open");
    assert_eq!(
        windows
            .most_recent_of(&app)
            .expect("the application has windows")
            .id()
            .number(),
        1,
        "using a window should move it to the front"
    );
}

/// **Several windows of one application are one entry on the Dock.**
///
/// This is what `how_many_the_dock_holds` counts, and it is the reason
/// `the_application_of` reads the app id rather than the title: two windows of
/// one program have two titles and one application, and a count that grouped by
/// title would draw the same program twice.
#[test]
fn several_windows_of_one_application_are_one_entry() {
    let mut windows = Windows::none();
    windows.opened(window(0, Some("a.terminal")));
    windows.opened(window(1, Some("a.terminal")));
    windows.opened(window(2, Some("an.editor")));
    let showing = Holding::nothing().showing(&windows);
    assert_eq!(showing.len(), 2, "{showing:?}");
    let terminal = showing
        .iter()
        .find(|entry| entry.app().name() == "a.terminal")
        .expect("the terminal is on the Dock");
    assert_eq!(terminal.how_many_windows(), 2);
}

/// **A window whose application is unknown adds no entry, and is not an
/// error.**
///
/// The builder includes such a window deliberately — it is open and a person
/// can see it — and `Holding::showing` is what decides it has no icon, because
/// there is no application to put one under. This holds that the decision stays
/// there rather than becoming a filter in the shell.
#[test]
fn a_window_that_named_no_application_adds_no_icon() {
    let mut windows = Windows::none();
    windows.opened(window(0, None));
    assert_eq!(windows.how_many(), 1, "the window is open");
    assert!(
        Holding::nothing().showing(&windows).is_empty(),
        "a window with no application has nothing to appear under"
    );
}
