//! Virtual desktops — each acceptance criterion of the task, walked through
//! this crate's public surface as the shell will reach it.
//!
//! The crate's own unit tests take each operation apart. This file is the other
//! half: one test per promise, written against nothing but what a compositor can
//! call, and holding every refusal to the sentence a person reads.
//!
//! It is not the hardware verification `CLAUDE.md` asks for. **Nothing here has
//! been swiped.** There is no compositor drawing a desktop yet; a person at a
//! certified machine is who says that moving between them feels like moving.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_desktops::{
    Desktop, DesktopChords, Desktops, DisplayId, Name, OnADisplay, Position, Promises, Refused,
    Switch, desktop_words,
};
use alo_dividing::{Area, Point, Side, Size, Window, WindowId};
use alo_shortcuts::{Action, Chord, Key, Modifier, Modifiers, Shortcuts};
use alo_strings::Strings;

/// A 1920 by 1080 display at the origin, in logical units.
fn area() -> Area {
    Area::of(Point::at(0, 0), Size::of(1920, 1080)).unwrap()
}

/// The window the compositor calls `id`.
fn window(id: u64) -> WindowId {
    WindowId::from_compositor(id)
}

/// A window stating no minimum size.
fn any(id: u64) -> Window {
    Window::any_size(window(id))
}

/// Three promise surfaces, numbered far away from any person's window.
fn promises(from: u64) -> Promises {
    Promises::of(window(from), window(from + 1), window(from + 2)).unwrap()
}

/// A person's desktops with one display plugged in, and that display's id.
fn one_display() -> (Desktops, DisplayId) {
    let laptop = DisplayId::from_compositor(1);
    let mut desktops = Desktops::default();
    desktops.plug_in(laptop, area(), promises(9001)).unwrap();
    (desktops, laptop)
}

/// A desktop of somebody else's display, which this one has never had.
fn a_desktop_elsewhere() -> alo_desktops::DesktopId {
    let other = DisplayId::from_compositor(99);
    let mut desktops = Desktops::default();
    desktops
        .plug_in(other, area(), promises(9901))
        .unwrap()
        .current()
}

/// A chord with Super held.
fn chord(key: Key) -> Chord {
    Chord::checked(Modifiers::just(Modifier::Super), key).unwrap()
}

/// This crate's own words, untranslated.
fn in_english() -> Strings {
    Strings::of(desktop_words().unwrap())
}

/// What each desktop of a display is called and which of the person's windows
/// are on it — the row a shell draws, read through nothing but the public
/// surface.
fn row(display: &OnADisplay, strings: &Strings) -> Vec<(u16, String, Vec<u64>)> {
    display
        .in_order()
        .map(|(at, desktop)| {
            (
                at.number(),
                desktop.shown(at, strings),
                desktop.windows().map(WindowId::to_compositor).collect(),
            )
        })
        .collect()
}

/// **`alo-desktops` holds a person's desktops per display — add, remove, name,
/// reorder — and which windows are on each.**
#[test]
fn a_person_adds_removes_names_and_reorders_their_desktops() {
    let strings = in_english();
    let (mut desktops, laptop) = one_display();
    let display = desktops.on_mut(laptop).unwrap();

    // One to begin with, unnamed, with nothing on it.
    let first = display.current();
    assert_eq!(
        row(display, &strings),
        vec![(1, "Desktop 1".to_owned(), vec![])]
    );

    // Two more, and a window on each of the three.
    let second = display.add().unwrap();
    let third = display.add().unwrap();
    for (desktop, id) in [(first, 1), (second, 2), (third, 3)] {
        display.put_on(desktop, window(id)).unwrap();
    }
    assert_eq!(
        row(display, &strings),
        vec![
            (1, "Desktop 1".to_owned(), vec![1]),
            (2, "Desktop 2".to_owned(), vec![2]),
            (3, "Desktop 3".to_owned(), vec![3]),
        ]
    );

    // Named, in the person's own words.
    display
        .call_it(second, Some(Name::given("Πόστα").unwrap()))
        .unwrap();
    assert_eq!(
        display
            .desktop(second)
            .unwrap()
            .shown(Position::numbered(2).unwrap(), &strings),
        "Πόστα"
    );

    // Reordered: the named one to the front. The windows go with their
    // desktops, and the numbers a person reads follow the row.
    display.reorder(second, Position::first()).unwrap();
    assert_eq!(
        row(display, &strings),
        vec![
            (1, "Πόστα".to_owned(), vec![2]),
            (2, "Desktop 2".to_owned(), vec![1]),
            (3, "Desktop 3".to_owned(), vec![3]),
        ]
    );

    // Removed, and the row closes up.
    display.remove(third).unwrap();
    assert_eq!(
        row(display, &strings),
        vec![
            (1, "Πόστα".to_owned(), vec![2]),
            (2, "Desktop 2".to_owned(), vec![1, 3]),
        ]
    );

    // And every way of asking for something that is not there is refused with a
    // sentence a person can act on, leaving the row exactly as it was.
    let before = row(display, &strings);
    let elsewhere = a_desktop_elsewhere();
    for refused in [
        display.remove(elsewhere).unwrap_err(),
        display.call_it(elsewhere, None).unwrap_err(),
        display.reorder(elsewhere, Position::first()).unwrap_err(),
    ] {
        assert_eq!(refused, Refused::NoSuchDesktop(elsewhere));
        let said = refused.said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        assert_eq!(refused.desktop(), Some(elsewhere));
    }
    assert_eq!(row(display, &strings), before);
}

/// **Removing a desktop moves its windows to a neighbour and never closes
/// one** — and the only desktop a display has cannot be removed at all.
#[test]
fn removing_a_desktop_moves_its_windows_to_a_neighbour_and_closes_none() {
    let (mut desktops, laptop) = one_display();
    let display = desktops.on_mut(laptop).unwrap();
    let first = display.current();
    let second = display.add().unwrap();
    let third = display.add().unwrap();
    display.put_on(second, window(1)).unwrap();
    display.put_on(second, window(2)).unwrap();
    display.put_on(third, window(3)).unwrap();

    // The middle one goes: its windows land on the desktop after it, and the
    // person, who was on it, lands there too.
    display
        .switch(Switch::To(Position::numbered(2).unwrap()))
        .unwrap();
    assert_eq!(display.current(), second);
    assert_eq!(display.remove(second), Ok(third));
    assert_eq!(display.current(), third, "the person follows their windows");
    assert_eq!(display.where_is(window(1)), Some(third));
    assert_eq!(display.where_is(window(2)), Some(third));
    assert_eq!(display.where_is(window(3)), Some(third));

    // The last one goes: its windows land on the desktop *before* it, because
    // there is nothing after it.
    assert_eq!(display.remove(third), Ok(first));
    assert_eq!(display.where_is(window(1)), Some(first));
    assert_eq!(display.where_is(window(3)), Some(first));
    assert_eq!(display.how_many(), 1);

    // And the only desktop left stays, with every window still on it.
    let refused = display.remove(first).unwrap_err();
    assert_eq!(refused, Refused::TheLastDesktop);
    assert!(!refused.said(&in_english()).is_a_bug());
    assert_eq!(display.how_many(), 1);
    for id in [1, 2, 3] {
        assert_eq!(display.where_is(window(id)), Some(first));
    }
}

/// **A window can be on every desktop**, and comes back onto one.
#[test]
fn a_window_can_be_on_every_desktop() {
    let (mut desktops, laptop) = one_display();
    let display = desktops.on_mut(laptop).unwrap();
    let first = display.current();
    let second = display.add().unwrap();
    display.put_on(first, window(1)).unwrap();
    display.put_on(first, window(2)).unwrap();

    display.on_every_desktop(window(1)).unwrap();
    assert!(display.is_everywhere(window(1)));
    for desktop in [first, second] {
        assert!(display.windows_on(desktop).unwrap().contains(&window(1)));
    }
    // It belongs to no one desktop's own list any more.
    assert_eq!(display.where_is(window(1)), None);
    assert!(!display.desktop(first).unwrap().holds(window(1)));
    // The window beside it did not move.
    assert!(!display.windows_on(second).unwrap().contains(&window(2)));

    // And back onto one desktop, off every other.
    display.put_on(second, window(1)).unwrap();
    assert!(!display.is_everywhere(window(1)));
    assert_eq!(display.where_is(window(1)), Some(second));
    assert!(!display.windows_on(first).unwrap().contains(&window(1)));
}

/// **Switching is a gesture or a shortcut**, and both hand the same request to
/// the same road.
#[test]
fn switching_is_a_gesture_or_a_shortcut() {
    let strings = in_english();
    let mut shortcuts = Shortcuts::shipped();
    let (mut desktops, laptop) = one_display();
    let first = desktops.on(laptop).unwrap().current();
    let second = desktops.on_mut(laptop).unwrap().add().unwrap();

    // The shortcut road: the person's own chords say what a chord asks for, and
    // this crate's own `switch` is what carries it out.
    let pressed = chord(Key::PageDown);
    let asked = desktops.chords().switch_for(&shortcuts, pressed);
    assert_eq!(asked, Some(Switch::Next));
    assert_eq!(
        desktops.on_mut(laptop).unwrap().switch(asked.unwrap()),
        Ok(second)
    );

    // The gesture road (task 5): the same value, and the same road back.
    assert_eq!(
        desktops.on_mut(laptop).unwrap().switch(Switch::Previous),
        Ok(first)
    );

    // A chord for a desktop by number, read as the desktop it reaches.
    let numbered = desktops
        .chords()
        .switch_for(&shortcuts, chord(Key::Digit2))
        .unwrap();
    assert_eq!(numbered.said(&strings).text(), "Desktop 2");
    assert_eq!(
        desktops.on_mut(laptop).unwrap().switch(numbered),
        Ok(second)
    );

    // **A chord a system shortcut holds is not a desktop chord.** The person
    // moves the launcher onto it, and the desktop chord stops answering rather
    // than shadowing a shortcut the compositor takes first.
    shortcuts.bind(Action::Launcher, pressed).unwrap();
    assert_eq!(desktops.chords().switch_for(&shortcuts, pressed), None);
    let refused = desktops
        .chords_mut()
        .bind(&shortcuts, pressed, Switch::Next)
        .unwrap_err();
    assert_eq!(refused, Refused::ChordIsTaken(Action::Launcher));
    assert!(!refused.said(&strings).is_a_bug());

    // A chord nobody bound is not a switch, and neither is any chord at all
    // once the person clears their desktop chords.
    assert_eq!(
        desktops.chords().switch_for(&shortcuts, chord(Key::Slash)),
        None
    );
    let mut cleared = Desktops::with(DesktopChords::none());
    cleared.plug_in(laptop, area(), promises(9101)).unwrap();
    assert_eq!(
        cleared.chords().switch_for(&shortcuts, chord(Key::PageUp)),
        None
    );

    // **And switching does not wrap**, at either end of the row.
    assert_eq!(
        desktops.on_mut(laptop).unwrap().switch(Switch::Next),
        Err(Refused::NoDesktopThatWay(Switch::Next))
    );
    assert_eq!(desktops.on(laptop).unwrap().current(), second);
}

/// **Each desktop keeps its own divisions.** A pair of windows split into halves
/// on one desktop stays split there while another desktop holds a single window,
/// and switching re-lays-out nothing.
#[test]
fn each_desktop_keeps_its_own_divisions() {
    let (mut desktops, laptop) = one_display();
    let display = desktops.on_mut(laptop).unwrap();
    let first = display.current();
    let second = display.add().unwrap();

    // Two windows on the first desktop, divided into halves.
    display.put_on(first, window(1)).unwrap();
    display.put_on(first, window(2)).unwrap();
    let division = display.desktop_mut(first).unwrap().division_mut();
    division
        .divide_with_next(any(1), Some(any(2)), Side::Left)
        .unwrap();
    assert_eq!(
        division.share_of(window(1)).map(|share| share.width()),
        Some(960)
    );

    // Two more on the second desktop, divided the other way: one above the
    // other, which the first desktop's division knows nothing about.
    display.put_on(second, window(3)).unwrap();
    display.put_on(second, window(4)).unwrap();
    display
        .desktop_mut(second)
        .unwrap()
        .division_mut()
        .divide_with_next(any(3), Some(any(4)), Side::Top)
        .unwrap();
    assert_eq!(
        display
            .desktop(second)
            .unwrap()
            .division()
            .share_of(window(3))
            .map(|share| (share.width(), share.height())),
        Some((1920, 540))
    );

    // Switching between them changes neither division.
    display.switch(Switch::Next).unwrap();
    display.switch(Switch::Previous).unwrap();
    assert_eq!(
        display
            .desktop(first)
            .unwrap()
            .division()
            .share_of(window(1))
            .map(|share| share.width()),
        Some(960)
    );
    assert_eq!(
        display.desktop(second).unwrap().division().shares().len(),
        2
    );

    // And a window moved to another desktop leaves its share behind: the
    // neighbour on the desktop it left takes the space, which is
    // `alo-dividing`'s rule and not re-decided here.
    display.put_on(second, window(1)).unwrap();
    assert_eq!(
        display
            .desktop(first)
            .unwrap()
            .division()
            .share_of(window(1)),
        None
    );
    assert_eq!(
        display
            .desktop(first)
            .unwrap()
            .division()
            .share_of(window(2))
            .map(|share| share.width()),
        Some(1920)
    );
    // It arrives on the second desktop floating: that desktop's division is an
    // arrangement somebody made, and nothing here rearranges it.
    assert_eq!(display.where_is(window(1)), Some(second));
    assert_eq!(
        display
            .desktop(second)
            .unwrap()
            .division()
            .share_of(window(1)),
        None
    );
}

/// **Desktops are per display**, and two displays never move together.
#[test]
fn desktops_are_per_display() {
    let laptop = DisplayId::from_compositor(1);
    let external = DisplayId::from_compositor(2);
    let mut desktops = Desktops::default();
    desktops.plug_in(laptop, area(), promises(9001)).unwrap();
    desktops.plug_in(external, area(), promises(9101)).unwrap();

    // Three desktops on the laptop, one on the external screen.
    desktops.on_mut(laptop).unwrap().add().unwrap();
    desktops.on_mut(laptop).unwrap().add().unwrap();
    assert_eq!(desktops.on(laptop).unwrap().how_many(), 3);
    assert_eq!(desktops.on(external).unwrap().how_many(), 1);

    // Switching on the laptop leaves the external screen where it was.
    let external_before = desktops.on(external).unwrap().current();
    desktops
        .on_mut(laptop)
        .unwrap()
        .switch(Switch::Next)
        .unwrap();
    assert_eq!(desktops.on(external).unwrap().current(), external_before);
    assert_eq!(
        desktops.on_mut(external).unwrap().switch(Switch::Next),
        Err(Refused::NoDesktopThatWay(Switch::Next))
    );

    // A window is on one display's desktops and is unknown to the other.
    let on_laptop = desktops.on(laptop).unwrap().current();
    desktops
        .on_mut(laptop)
        .unwrap()
        .put_on(on_laptop, window(1))
        .unwrap();
    assert_eq!(
        desktops.on(laptop).unwrap().where_is(window(1)),
        Some(on_laptop)
    );
    assert_eq!(desktops.on(external).unwrap().where_is(window(1)), None);
    assert_eq!(
        desktops.on_mut(external).unwrap().close(window(1)),
        Err(Refused::NoSuchWindow(window(1)))
    );

    // The external screen goes, and its desktops are handed back whole rather
    // than dropped: nothing on the laptop notices.
    let had = desktops.unplug(external).unwrap();
    assert_eq!(had.how_many(), 1);
    assert_eq!(desktops.on(laptop).unwrap().how_many(), 3);
    assert_eq!(desktops.displays().collect::<Vec<_>>(), vec![laptop]);
}

/// Every desktop a person can read is named, and every sentence this crate can
/// say about one is said — the vocabulary is whole.
#[test]
fn every_sentence_about_a_desktop_is_declared() {
    let strings = in_english();
    let (mut desktops, laptop) = one_display();
    let display = desktops.on_mut(laptop).unwrap();
    let first = display.current();

    // Filling a display to the cap, and reading every row.
    while display.how_many() < alo_desktops::MOST_DESKTOPS {
        display.add().unwrap();
    }
    for (at, desktop) in display.in_order() {
        let said = Desktop::numbered(at, &strings);
        assert!(!said.is_a_bug(), "desktop {}", at.number());
        assert!(said.unfilled().is_empty(), "desktop {}", at.number());
        assert_eq!(desktop.shown(at, &strings), said.text());
    }

    // The cap says what it is.
    let refused = display.add().unwrap_err();
    assert_eq!(refused, Refused::TooManyDesktops);
    let said = refused.said(&strings);
    assert!(said.text().contains("16"), "{said}");
    assert!(said.unfilled().is_empty(), "{said}");

    // And a name that is not a name says which of the three ways it is not.
    for typed in ["", "  ", "a\nb"] {
        let refused = Name::given(typed).unwrap_err();
        let said = refused.said(&strings);
        assert!(!said.is_a_bug(), "{typed:?}");
        assert!(said.unfilled().is_empty(), "{typed:?}: {said}");
    }
    assert!(
        display
            .call_it(first, Some(Name::given("Post").unwrap()))
            .is_ok()
    );
}
