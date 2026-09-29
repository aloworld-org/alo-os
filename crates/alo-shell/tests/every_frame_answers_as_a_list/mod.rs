//! **Every canvas answers as a list** — the canvas plan's task 7, which the plan
//! puts there so that *the first accessibility review does not undo the other
//! nine*.
//!
//! Its constraint is the important half: **this is not a second interface. It is
//! the same frames, said in order.** So there is nothing new here to reach a frame
//! with — the shipped `NextWindow` and `PreviousWindow` chords are the road, the
//! ring is `switch_window`'s own, and what these cases prove is that **a canvas
//! cannot change any of it**.
//!
//! # The clause a canvas can break, and what a stability test can actually catch
//!
//! *The order is stable across a pan and a zoom.* `switch_window`'s ring follows
//! **first observation**, so it is independent of position already, and the second
//! case below is how that stays true.
//!
//! But be exact about what that case proves, because the obvious claim is wrong: a
//! pan and a zoom are a translation and a positive scale, so **a plain sort by
//! where frames sit is monotonic under both** and would pass a stability test
//! untouched. What the case catches is the orderings that are not — banding by
//! screen rows into a reading order, where a zoom changes which frames share a
//! row, and anything measured from the viewport rather than from the plane, like
//! *nearest the middle first*.
//!
//! The failure a person actually meets is the third case: an enumeration that
//! reaches **only what is on the screen**. That is the tempting implementation of
//! *list the frames* on a canvas, it passes both cases above, and it loses a frame
//! the moment somebody pans away from it. Proved to bite — skip frames past the
//! viewport in `switch_window` and that case fails while the other two stay green,
//! which is exactly why it is written separately rather than folded in.
//!
//! Reachability is tested here too, with **no pointer enabled at all**, because
//! the acceptance says so and because a keyboard road that quietly depended on a
//! pointer having been somewhere is exactly the dependency a sighted author does
//! not notice.
//!
//! # What cannot be held yet: a frame has no name
//!
//! The acceptance asks that every frame be *reached, focused and named*. **This
//! compositor never reads a window's name.** `xdg_toplevel.set_title` is not read
//! anywhere in `alo-shell` — the only labels it draws are the three window
//! controls' own — so there is nothing to say about a frame except which one it is.
//!
//! That is a missing capability rather than a missing test, it is shared with task
//! 3 (ADR 0065: *the name is the handle*) and with ADR 0065's *frames carry a name,
//! shown when the canvas is far out*, and it is written up under task 7 in the plan.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{Application, Fixture, looking};
use alo_shell::WindowSwitchDirection;
use alo_shortcuts::{Action, Chord, Shortcuts};

/// Complete the real XDG handshake and attach the 16x16 buffer.
fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}

/// The chord a person actually presses for this action, as shipped.
fn chord(settings: &Shortcuts, action: Action) -> Chord {
    settings
        .chord_for(action)
        .expect("a shipped binding for this action")
}

/// Walk the whole ring by keyboard, and say which frames were **told** they had
/// focus, in the order the wire carried it.
///
/// The evidence is each client's own `wl_keyboard.enter`, not a question put back
/// to the compositor: *focused* means the application was told, and a compositor
/// that moved its own idea of focus without telling anybody is the failure a
/// screen reader meets first. Each step presses the person's own shipped chord.
///
/// Starts from whatever is focused, so the sequence is the ring's own rotation
/// rather than a list built here. Compared as a rotation below, because *stable*
/// means the same cycle, not the same starting point.
fn walked_by_keyboard(f: &Fixture, settings: &Shortcuts, apps: &mut [Application]) -> Vec<usize> {
    let next = chord(settings, Action::NextWindow);
    for app in apps.iter_mut() {
        app.sync();
        app.events.keyboard.surfaces.clear();
    }
    let mut reached = Vec::new();
    for _ in 0..apps.len() {
        let settings = settings.clone();
        let action = f
            .backend(move |s| s.dispatch_window_shortcut(&settings, next))
            .expect("the shipped next-window chord");
        assert_eq!(
            action,
            Some(Action::NextWindow),
            "the shipped chord for the next frame reached no action"
        );
        // **Which client**, not which surface id. A protocol id is per
        // connection, so three separate applications each call their own surface
        // object eight — the same collision that stopped `window_number` being
        // keyed by one. Whose socket carried the enter is the only identity that
        // distinguishes them.
        let mut told: Vec<usize> = Vec::new();
        for (which, app) in apps.iter_mut().enumerate() {
            app.sync();
            if !app.events.keyboard.surfaces.is_empty() {
                app.events.keyboard.surfaces.clear();
                told.push(which);
            }
        }
        assert_eq!(
            told.len(),
            1,
            "one press of the next-frame chord told {} frames they had focus",
            told.len()
        );
        reached.append(&mut told);
    }
    reached
}

/// Whether these two cycles are the same ring, allowing a different start.
fn the_same_ring(first: &[usize], again: &[usize]) -> bool {
    if first.len() != again.len() || first.is_empty() {
        return false;
    }
    (0..first.len()).any(|turn| {
        first
            .iter()
            .enumerate()
            .all(|(at, frame)| again.get((at + turn) % again.len()) == Some(frame))
    })
}

/// **Three frames are reached and focused by keyboard with no pointer at all.**
#[test]
fn every_frame_is_reached_and_focused_with_no_pointer_enabled() {
    let f = Fixture::keyboard();
    // Deliberately no `enable_pointer`: the acceptance says *with no pointer at
    // all*, and a keyboard road that needed one would pass any test written by
    // somebody who had moved the arrow first.
    let mut apps: Vec<Application> = (0..3).map(|_| mapped(&f)).collect();
    let settings = Shortcuts::shipped();

    let reached = walked_by_keyboard(&f, &settings, &mut apps);
    let mut distinct = reached.clone();
    distinct.sort_unstable();
    distinct.dedup();
    assert_eq!(
        distinct.len(),
        3,
        "three frames are open and three presses of the next-frame chord told \
         {} distinct frames they had focus: {reached:?}",
        distinct.len()
    );

    // And backwards, which is the other shipped chord.
    let previous = chord(&settings, Action::PreviousWindow);
    let settings_now = settings.clone();
    assert_eq!(
        f.backend(move |s| s.dispatch_window_shortcut(&settings_now, previous))
            .expect("the shipped previous-window chord"),
        Some(Action::PreviousWindow)
    );
}

/// **The order does not change when the plane moves under it.**
///
/// Three frames are placed far apart on the plane, the ring is walked, then the
/// plane is panned and zoomed and walked again — and it has to be the same ring. A
/// reader that announced *two of three* about a different frame each time a person
/// moved would be a reader nobody could follow.
///
/// This does **not** catch a plain sort by position, which is monotonic under a
/// translation and a positive scale; see this module's header for what it does
/// catch and which case catches the rest.
#[test]
fn the_order_is_the_same_ring_across_a_pan_and_a_zoom() {
    let f = Fixture::keyboard();
    let mut apps: Vec<Application> = (0..3).map(|_| mapped(&f)).collect();
    let roots = f.backend(|s| s.mapped_surfaces().cloned().collect::<Vec<_>>());
    assert_eq!(roots.len(), 3, "three clients did not map");
    // Far apart, and deliberately not in the order they mapped: an enumeration
    // that read positions would come back sorted, which is what this catches.
    for (root, place) in roots.iter().zip([(900, 40), (100, 700), (500, 300)]) {
        let root = root.clone();
        assert!(f.backend(move |s| s.place_window(&root, place)).is_ok());
    }
    let settings = Shortcuts::shipped();

    let first = walked_by_keyboard(&f, &settings, &mut apps);
    // Three distinct frames, or the check below would compare a stuck ring with
    // itself and call it stable.
    let mut distinct = first.clone();
    distinct.sort_unstable();
    distinct.dedup();
    assert_eq!(
        distinct.len(),
        3,
        "the ring did not reach three frames: {first:?}"
    );
    looking(&f, (-260, 180), 400);
    let zoomed_out = walked_by_keyboard(&f, &settings, &mut apps);
    assert!(
        the_same_ring(&first, &zoomed_out),
        "the frames answered in a different order once the plane was panned and \
         zoomed out"
    );
    looking(&f, (640, -90), 2500);
    let zoomed_in = walked_by_keyboard(&f, &settings, &mut apps);
    assert!(
        the_same_ring(&first, &zoomed_in),
        "the frames answered in a different order once the plane was zoomed in"
    );
}

/// **A frame is reachable wherever it sits, including off the viewport.**
///
/// A list that can only reach what is on the screen is a list that loses a frame
/// the moment somebody pans away from it — and it is the failure a person cannot
/// work around, because the way back to a frame is to be told it is there. The
/// plan's task 8 is *a frame is never lost*; this is the part of it a reader
/// depends on, and it is cheap to hold here.
#[test]
fn a_frame_far_outside_the_viewport_is_still_reached_and_focused() {
    let f = Fixture::keyboard();
    let mut apps: Vec<Application> = (0..2).map(|_| mapped(&f)).collect();
    let roots = f.backend(|s| s.mapped_surfaces().cloned().collect::<Vec<_>>());
    // `mapped_surfaces` is in mapping order and nothing has raised anything yet,
    // so the second root is the second application.
    let far = roots.last().cloned().expect("a second frame");
    let far_app = apps.len() - 1;
    {
        let far = far.clone();
        // Half a million units away, which no viewport this decade reaches.
        assert!(
            f.backend(move |s| s.place_window(&far, (500_000, 500_000)))
                .is_ok()
        );
    }
    looking(&f, (0, 0), 1000);

    let settings = Shortcuts::shipped();
    let reached = walked_by_keyboard(&f, &settings, &mut apps);
    assert!(
        reached.contains(&far_app),
        "a frame half a million units off the screen could not be reached by \
         keyboard, so a person who panned away from it has no way back"
    );
    assert!(
        f.backend(move |s| s.switch_window(WindowSwitchDirection::Forward).is_ok()),
        "the ring refused to turn with two frames open"
    );
}

/// What a reader is told the windows open are, in the tree's own order.
fn the_windows_a_reader_hears(f: &Fixture) -> Vec<String> {
    f.backend(|s| {
        let strings = alo_strings::Strings::of(
            alo_saying::everything_this_machine_can_say().expect("the assembled vocabulary"),
        );
        s.read_aloud_with_the_frames_open(&strings, &[alo_access::Surface::Desktop])
            .the_windows_open_as_read()
            .into_iter()
            .map(str::to_owned)
            .collect()
    })
}

/// The name a reader would hear for this frame, as the shell reads it.
fn named(name: alo_shell::FrameName) -> String {
    match name {
        alo_shell::FrameName::Given(name) | alo_shell::FrameName::ItsClass(name) => name,
        alo_shell::FrameName::AnApplication => "an application".to_owned(),
    }
}

/// **Every frame is named to a reader, by whatever the application calls itself.**
///
/// The last third of task 7's acceptance — *every frame can be reached, focused
/// and named* — and the third that was blocked on 2026-09-27 because nothing in
/// this shell read `xdg_toplevel.set_title`.
///
/// All three cases a name can arrive in are here: a title the application set, a
/// class where it set no title, and the machine's own words where it set neither.
/// The third is the only one a translator ever sees.
#[test]
fn every_frame_is_named_to_a_reader_by_whatever_it_calls_itself() {
    let f = Fixture::keyboard();
    let mut titled = mapped(&f);
    titled.toplevel.set_title("Ledger for March".to_owned());
    titled.sync();
    let mut classed = mapped(&f);
    classed.toplevel.set_app_id("org.alo.Notes".to_owned());
    classed.sync();
    let mut nameless = mapped(&f);
    nameless.sync();

    let heard = the_windows_a_reader_hears(&f);
    assert_eq!(
        heard.len(),
        3,
        "three windows are open, a reader heard {heard:?}"
    );
    assert!(
        heard.contains(&"Ledger for March".to_owned()),
        "an application's own title did not reach the reader: {heard:?}"
    );
    assert!(
        heard.contains(&"org.alo.Notes".to_owned()),
        "an application with no title was not named by its class: {heard:?}"
    );
    assert!(
        heard.contains(&"an application".to_owned()),
        "an application that named itself nothing got no words of ours: {heard:?}"
    );
}

/// **A reader hears the windows in the order the keyboard walks them.**
///
/// Task 7's constraint is that this is *not a second interface*. A list read in
/// one order while the keyboard moves in another would be exactly that: two
/// answers to *which window is next*, and a person using both told one thing and
/// shown another.
#[test]
fn the_order_a_reader_hears_is_the_order_the_keyboard_walks() {
    let f = Fixture::keyboard();
    let mut first = mapped(&f);
    first.toplevel.set_title("one".to_owned());
    first.sync();
    let mut second = mapped(&f);
    second.toplevel.set_title("two".to_owned());
    second.sync();
    let mut third = mapped(&f);
    third.toplevel.set_title("three".to_owned());
    third.sync();

    let heard = the_windows_a_reader_hears(&f);
    let walked: Vec<String> = f.backend(|s| {
        s.mapped_surfaces()
            .map(|frame| named(s.the_name_of(frame)))
            .collect()
    });
    assert_eq!(
        heard, walked,
        "a reader hears the windows in a different order from the one they are walked in"
    );
}

/// **A window that closes stops being read**, so a reader is never offered one
/// that is not there any more.
#[test]
fn a_window_that_closes_is_no_longer_read() {
    let f = Fixture::keyboard();
    let mut staying = mapped(&f);
    staying.toplevel.set_title("staying".to_owned());
    staying.sync();
    let mut going = mapped(&f);
    going.toplevel.set_title("going".to_owned());
    going.sync();
    assert_eq!(the_windows_a_reader_hears(&f).len(), 2);

    drop(going);
    f.wait_for((1, 1));
    assert_eq!(
        the_windows_a_reader_hears(&f),
        vec!["staying".to_owned()],
        "a reader was still told about a window that had gone"
    );
}
