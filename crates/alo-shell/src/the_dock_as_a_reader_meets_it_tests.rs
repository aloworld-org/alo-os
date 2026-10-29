//! That a reader is told each control on the Dock by the words it is drawn
//! with, can press each one a click does something on, and that a press heard
//! on a Dock that has since changed reaches nothing.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported, and \
              a formatted panic names the edge it was on"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_access::{Surface, TurnedOn};
use alo_dock::a_dock_entry::ANativeAction;
use alo_dock::places::WhatIsHere;
use alo_dock::{AppId, Dock, Edge};
use alo_strings::{Language, Strings, Translation};

use super::{ItsName, as_read};
use crate::access_action::Pressed;
use crate::access_nodes::ROOT;
use crate::approval_testing::words;
use crate::where_the_dock_is::{TheDocksPlaces, where_the_dock_is};
use crate::{ReadAloudTree, Server, WhatAReaderPressCameTo};

/// A display of this test's own.
fn a_session(what: &str) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(runtime.path(), what).expect("a display");
    (runtime, server)
}

/// This many open applications, named `app-0` onwards, as `alo-dock` hands
/// them over.
fn on_the_dock(how_many: usize) -> Vec<alo_dock::OnTheDock> {
    let patch = alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 1, 1).expect("one by one is a patch");
    let mut windows = alo_dock::Windows::none();
    for number in 0..how_many {
        let named = format!("app-{number}");
        windows.opened(alo_dock::Window::of(
            alo_dock::WindowId::numbered(number as u64),
            Some(AppId::named(&named).expect("a named application")),
            &named,
            patch,
            alo_dock::HowItSits::OnTheCanvas,
        ));
    }
    alo_dock::Holding::nothing().showing(&windows)
}

/// The Dock on this edge of a 1920 × 1080 display, holding three applications.
fn on_the_edge(edge: Edge) -> TheDocksPlaces {
    let mut dock = Dock::shipped();
    dock.set_edge(edge);
    where_the_dock_is(
        &dock,
        (1920, 1080),
        &on_the_dock(3),
        crate::display_scale::DisplayScale::ONE,
    )
    .unwrap_or_else(|why| panic!("{edge:?} would not lay out: {why:?}"))
}

/// What is on that Dock, in the order it is drawn.
fn slots(dock: &TheDocksPlaces) -> Vec<WhatIsHere> {
    dock.places
        .each()
        .iter()
        .map(|place| place.what().clone())
        .collect()
}

/// The tree with the Dock up and this Dock's controls in it.
fn heard(strings: &Strings, dock: &TheDocksPlaces) -> ReadAloudTree {
    ReadAloudTree::of(strings, &[Surface::Dock], &TurnedOn::nothing())
        .with_the_docks_slots(strings, &slots(dock))
}

/// `dock.history` in German, and every other word as written.
fn in_german() -> Strings {
    let vocabulary = alo_saying::everything_this_machine_can_say().expect("the vocabulary");
    let language = Language::written("de").expect("German");
    let translation = vocabulary
        .check(
            Translation::into_language(language.clone())
                .says(alo_dock::words::HISTORY.key(), "Verlauf"),
        )
        .expect("a translation of History");
    let mut strings = Strings::of(vocabulary);
    strings.speaks(translation).expect("German is spoken");
    strings.prefers(&[language]);
    strings
}

/// **Every control on the Dock is named, on every edge, by the words a
/// sighted person is shown** — and the general description is gone.
#[test]
fn every_control_on_the_dock_is_named_on_every_edge() {
    let strings = words();
    for edge in Edge::EVERY {
        let dock = on_the_edge(edge);
        let tree = heard(&strings, &dock);
        let names = tree.the_dock_as_read();
        for expected in ["app-0", "app-1", "app-2", "Show more apps"] {
            assert!(
                names.contains(&expected),
                "{edge:?}: no {expected} in {names:?}"
            );
        }
        assert_eq!(
            names.contains(&"History"),
            edge != Edge::Bottom,
            "{edge:?}: History is on every Dock but the bottom one: {names:?}"
        );
        for gone in ["an application", "open something"] {
            assert!(
                !names.contains(&gone),
                "{edge:?}: the general description {gone:?} is still read: {names:?}"
            );
        }
        let controls = slots(&dock)
            .iter()
            .filter(|what| !matches!(what, WhatIsHere::TheDivider))
            .count();
        assert_eq!(
            names.len(),
            controls,
            "{edge:?}: one name per control, and none for the divider"
        );
    }
}

/// **History is said in the person's language**, on a side Dock.
#[test]
fn history_is_said_in_the_persons_language() {
    let strings = in_german();
    for edge in [Edge::Left, Edge::Right] {
        let names = heard(&strings, &on_the_edge(edge))
            .the_dock_as_read()
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        assert!(names.contains(&"Verlauf".to_owned()), "{edge:?}: {names:?}");
        assert!(
            !names.contains(&"History".to_owned()),
            "{edge:?}: {names:?}"
        );
    }
}

/// **A press is offered where a click does something, and nowhere else.**
///
/// History opens nothing yet, so it is named and not pressable; the divider is
/// not a control at all.
#[test]
fn a_press_is_offered_where_a_click_does_something_and_nowhere_else() {
    let app = AppId::named("files").expect("a named application");
    let every = [
        WhatIsHere::Application(app.clone()),
        WhatIsHere::ANativeAction(ANativeAction::Settings),
        WhatIsHere::ANativeAction(ANativeAction::Applications),
        WhatIsHere::TheOverflow,
        WhatIsHere::TheComposer,
        WhatIsHere::ShowAll,
    ];
    for what in &every {
        let slot = as_read(what).unwrap_or_else(|| panic!("{what:?} was not read"));
        assert_eq!(
            slot.press,
            Some(Pressed::OnTheDock(what.clone())),
            "{what:?}"
        );
    }
    assert_eq!(
        as_read(&WhatIsHere::Application(app)).map(|slot| slot.name),
        Some(ItsName::ItsOwn("files")),
        "an application is called by its own name"
    );
    let history = as_read(&WhatIsHere::History).expect("History is read");
    assert_eq!(history.name, ItsName::Said(alo_dock::words::HISTORY));
    assert_eq!(
        history.press, None,
        "History offered a press that does nothing"
    );
    assert_eq!(
        as_read(&WhatIsHere::TheDivider),
        None,
        "the divider was read"
    );
}

/// **Before a Dock is drawn, the description stays**, rather than an empty
/// list a reader would hear as a Dock with nothing on it.
#[test]
fn with_no_dock_drawn_the_description_stays() {
    let strings = words();
    let before = ReadAloudTree::of(&strings, &[Surface::Dock], &TurnedOn::nothing());
    let after = before.clone().with_the_docks_slots(&strings, &[]);
    assert_eq!(before, after);
    assert_eq!(
        after.the_dock_as_read(),
        ["an application", "open something"]
    );
}

/// **The tree still holds together after the description is taken away**:
/// each path is its position, each child names its parent, and the windows
/// open are still found and still hang under the desktop.
#[test]
fn the_tree_still_holds_together_after_the_dock_is_filled() {
    let strings = words();
    let frames = [crate::FrameName::Given("Letters".to_owned())];
    let tree = ReadAloudTree::of(&strings, &Surface::ALL, &TurnedOn::nothing())
        .with_the_docks_slots(&strings, &slots(&on_the_edge(Edge::Bottom)))
        .with_the_frames_open(&strings, &frames, &TurnedOn::nothing());
    for (at, node) in tree.nodes().iter().enumerate() {
        let path = if at == 0 {
            ROOT.to_owned()
        } else {
            format!("/org/a11y/atspi/accessible/{at}")
        };
        assert_eq!(node.path, path, "node {at} is not where its path says");
        for child in &node.children {
            assert_eq!(
                tree.nodes()[*child].parent,
                at,
                "{:?}",
                tree.nodes()[*child].name
            );
        }
    }
    assert_eq!(tree.the_windows_open_as_read(), ["Letters"]);
}

/// **The Dock's controls are shown exactly when the Dock is.**
#[test]
fn the_docks_controls_are_shown_when_the_dock_is_and_not_otherwise() {
    const SHOWING: u32 = 25;
    let strings = words();
    let dock = on_the_edge(Edge::Bottom);
    for (up, showing) in [(vec![Surface::Dock], true), (vec![Surface::Desktop], false)] {
        let tree = ReadAloudTree::of(&strings, &up, &TurnedOn::nothing())
            .with_the_docks_slots(&strings, &slots(&dock));
        let pressable: Vec<_> = tree
            .nodes()
            .iter()
            .filter(|node| matches!(node.press, Some(Pressed::OnTheDock(_))))
            .collect();
        assert!(!pressable.is_empty(), "the control: the Dock has controls");
        for node in pressable {
            assert_eq!(
                node.states[0] & (1 << SHOWING) != 0,
                showing,
                "{:?} with {up:?} up",
                node.name
            );
        }
    }
}

/// **The tree a desktop publishes carries the Dock as it was drawn**, and
/// each of its controls can be pressed.
#[test]
fn the_published_tree_carries_the_dock_as_drawn() {
    let (_runtime, mut server) = a_session("the-dock-a-reader-meets");
    let dock = on_the_edge(Edge::Left);
    let expected = slots(&dock);
    server.the_docks_slots_were_drawn("a-display", dock);
    let tree =
        server.read_aloud_with_the_frames_open(&words(), &[Surface::Dock], &TurnedOn::nothing());
    assert!(tree.the_dock_as_read().contains(&"History"));
    let pressed = tree.what_can_be_pressed();
    for what in expected
        .iter()
        .filter(|what| !matches!(what, WhatIsHere::TheDivider | WhatIsHere::History))
    {
        assert!(
            pressed.contains(&Pressed::OnTheDock(what.clone())),
            "{what:?} cannot be pressed"
        );
    }
}

/// **A reader pressing *Show more apps* opens the list, as a click does.**
#[test]
fn a_readers_press_on_show_more_apps_opens_the_list() {
    use crate::what_the_dock_has_open::WhatTheDockWasAskedFor;
    let (_runtime, mut server) = a_session("a-reader-presses-more-apps");
    server.the_docks_slots_were_drawn("a-display", on_the_edge(Edge::Bottom));
    assert_eq!(
        server.a_reader_pressed(Pressed::OnTheDock(WhatIsHere::TheOverflow)),
        WhatAReaderPressCameTo::TheDockDidIt
    );
    assert!(
        matches!(
            server.the_dock_was_asked_for,
            WhatTheDockWasAskedFor::MoreApps { .. }
        ),
        "the list did not open"
    );
}

/// **A press on something no Dock holds now reaches nothing**: an application
/// that has gone is not opened again by a press heard before it went, and no
/// Dock at all means no press is carried out.
#[test]
fn a_readers_press_on_what_the_dock_no_longer_holds_does_nothing() {
    use crate::what_the_dock_has_open::WhatTheDockWasAskedFor;
    let gone = WhatIsHere::Application(AppId::named("gone").expect("a named application"));

    let (_runtime, mut server) = a_session("a-reader-presses-nothing");
    assert_eq!(
        server.a_reader_pressed(Pressed::OnTheDock(WhatIsHere::TheOverflow)),
        WhatAReaderPressCameTo::NothingOnTheDock,
        "a press with no Dock drawn was carried out"
    );
    server.the_docks_slots_were_drawn("a-display", on_the_edge(Edge::Bottom));
    assert_eq!(
        server.a_reader_pressed(Pressed::OnTheDock(gone)),
        WhatAReaderPressCameTo::NothingOnTheDock
    );
    assert_eq!(
        server.a_reader_pressed(Pressed::OnTheDock(WhatIsHere::History)),
        WhatAReaderPressCameTo::NothingOnTheDock,
        "History is not on the bottom Dock and opens nothing anywhere"
    );
    assert!(
        matches!(
            server.the_dock_was_asked_for,
            WhatTheDockWasAskedFor::Nothing
        ),
        "a refused press changed what the Dock has open"
    );
}
