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

/// A window of `app`, numbered `number`, sitting however it sits.
fn window_sitting(number: u64, app: Option<&str>, sits: HowItSits) -> Window {
    let patch = Patch::of(Spot::at(0, 0), 1, 1).expect("one by one is a patch");
    Window::of(
        WindowId::numbered(number),
        app.map(|app| AppId::named(app).expect("a named application")),
        "",
        patch,
        sits,
    )
}

/// **A minimised window still counts as one of its application's windows.**
///
/// The assumption the builder rests on now that it reads `minimized_surfaces`
/// as well as `mapped_surfaces`. `OnTheDock::is_open` is `windows > 0` and
/// nothing subtracts the put-aside ones — which is what makes *open, including
/// minimised windows, gets a running indicator* true rather than hoped for.
#[test]
fn an_application_whose_windows_are_all_minimised_is_still_open() {
    let mut holding = Holding::nothing();
    holding.pin(AppId::named("an.app").expect("a named application"));
    let mut windows = Windows::none();
    windows.opened(window_sitting(0, Some("an.app"), HowItSits::PutAside));
    windows.opened(window_sitting(1, Some("an.app"), HowItSits::PutAside));

    let shown = holding.showing(&windows);
    let entry = shown.first().expect("the pinned application is shown");
    assert!(
        entry.is_open(),
        "an application put aside drew no indicator"
    );
    assert_eq!(entry.how_many_windows(), 2);
    assert_eq!(
        entry.how_many_put_aside(),
        2,
        "the put-aside count was zero, which is what it always was before the \
         builder read the minimised list"
    );
}

/// **And that count is what a screen reader is told.**
///
/// `alo_dock::announcing` reads `how_many_put_aside`, so the builder feeding it
/// a permanent zero was a person with three windows put away being told about
/// none of them. Asserted through `announcing` rather than through the count,
/// because the count being right is not the thing that was broken.
#[test]
fn the_put_aside_count_reaches_what_a_person_is_told() {
    let mut holding = Holding::nothing();
    holding.pin(AppId::named("an.app").expect("a named application"));
    let mut windows = Windows::none();
    windows.opened(window_sitting(0, Some("an.app"), HowItSits::OnTheCanvas));
    windows.opened(window_sitting(1, Some("an.app"), HowItSits::PutAside));

    let shown = holding.showing(&windows);
    let entry = shown.first().expect("the pinned application is shown");
    let said = alo_dock::announcing::Announced::of(entry, alo_dock::announcing::Focused::No);
    assert_eq!(
        said.which().key(),
        alo_dock::words::ANNOUNCED_SOME_PUT_ASIDE.key(),
        "a person was told the plain open sentence, which is the one chosen \
         when nothing is put aside"
    );
}

/// **An unpinned application with one window open and one put aside keeps its
/// icon**, and loses it when the second goes.
///
/// The owner's worked example, asserted here rather than only in `alo-dock`
/// because this is the file that decides which windows the rule ever sees. With
/// the minimised list unread, the second window simply was not there — the
/// right answer by exclusion rather than by decision, which stops being the
/// right answer the moment the rule changes.
#[test]
fn the_builders_two_lists_are_what_the_membership_rule_reads() {
    let holding = Holding::nothing();

    let mut one_of_each = Windows::none();
    one_of_each.opened(window_sitting(0, Some("an.app"), HowItSits::OnTheCanvas));
    one_of_each.opened(window_sitting(1, Some("an.app"), HowItSits::PutAside));
    assert_eq!(
        holding.showing(&one_of_each).len(),
        1,
        "an application with a window still on the canvas left the Dock"
    );

    let mut both_aside = Windows::none();
    both_aside.opened(window_sitting(0, Some("an.app"), HowItSits::PutAside));
    both_aside.opened(window_sitting(1, Some("an.app"), HowItSits::PutAside));
    assert!(
        holding.showing(&both_aside).is_empty(),
        "an unpinned application with every window put aside stayed on the Dock"
    );
}
