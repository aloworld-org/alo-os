//! What a Dock is filled with, and the two empties that are not the same.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_dock::AppId;
use alo_dock::pinning::{ARole, Discovery, ThePins};
use alo_dock::{ADockEntry, ANativeAction};

use crate::Server;

/// A server on a socket of its own.
fn server(named: &str) -> (tempfile::TempDir, Server) {
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let bound = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, bound)
}

/// An application the Dock will take.
fn app(name: &str) -> AppId {
    AppId::named(name).expect("a plain identifier is an application the Dock takes")
}

/// A completed reading filling both roles a scan can answer.
fn both_roles() -> Discovery {
    Discovery::Completed(vec![
        (ARole::Files, app("org.kde.dolphin")),
        (ARole::Browser, app("org.mozilla.firefox")),
    ])
}

/// **A fresh machine's Dock is pinned to what discovery found.**
///
/// The assertion is on the identifiers and their order, not on a count: a
/// filling that pinned two of something would satisfy a count and is exactly
/// the kind of wrong this is here to catch.
#[test]
fn a_completed_reading_fills_the_dock_with_what_it_found() {
    let (_directory, mut server) = server("a-completed-reading");

    server.its_dock_is_filled_from(&both_roles());

    let pinned: Vec<&str> = server
        .what_the_dock_is_pinned_to()
        .each_pinned()
        .filter_map(alo_dock::ADockEntry::application)
        .map(AppId::name)
        .collect();
    assert_eq!(
        pinned,
        vec!["org.kde.dolphin", "org.mozilla.firefox"],
        "the Dock was not pinned to the applications discovery found, in the roles' order"
    );
}

/// **A failed reading pins nothing — and that is not the same as a machine
/// with nothing.**
///
/// Both leave an empty Dock this boot and the difference is what gets written
/// down, which nothing writes yet. Asserted here anyway, because the day
/// persistence lands this is the test that says which of the two may be
/// stored, and a test written then would be written against the code.
#[test]
fn a_failed_reading_pins_nothing() {
    let (_directory, mut server) = server("a-failed-reading");
    server.its_dock_is_filled_from(&both_roles());

    server.its_dock_is_filled_from(&Discovery::Failed);

    assert_eq!(
        server.what_the_dock_is_pinned_to().each_pinned().count(),
        0,
        "a failed reading left pins on the Dock"
    );
    assert!(
        !ThePins::NeverSet
            .what_to_pin(&Discovery::Failed)
            .should_be_written_down(),
        "a failed reading asked to be written down, which would empty a person's Dock for good"
    );
    assert!(
        ThePins::NeverSet
            .what_to_pin(&Discovery::Completed(Vec::new()))
            .should_be_written_down(),
        "a machine that really has nothing installed was not written down, so it would be \
         rediscovered every boot"
    );
}

/// **A reading that has not finished pins nothing either**, and for the other
/// reason: it is waited for rather than reported.
#[test]
fn a_reading_still_running_pins_nothing() {
    let (_directory, mut server) = server("still-running");
    server.its_dock_is_filled_from(&both_roles());

    server.its_dock_is_filled_from(&Discovery::StillLooking);

    assert_eq!(
        server.what_the_dock_is_pinned_to().each_pinned().count(),
        0,
        "a reading that had not finished left pins on the Dock"
    );
}

/// **A machine that really has nothing installed gets Settings and nothing
/// else**, which is a real answer and not a failure.
///
/// This asserted an empty Dock until Settings became a native entry. It is not
/// installed — there is nothing about it a scan could fail to find — so a
/// machine before `alo-shipping` has run shows the one entry that needs no
/// application. The fact being tested is unchanged: *completed and found
/// nothing* is a different state from *has not finished*, and the test below
/// is its pair.
#[test]
fn a_machine_with_nothing_installed_is_pinned_to_settings_and_nothing_else() {
    let (_directory, mut server) = server("nothing-installed");

    server.its_dock_is_filled_from(&Discovery::Completed(Vec::new()));

    let pinned: Vec<&ADockEntry> = server.what_the_dock_is_pinned_to().each_pinned().collect();
    assert_eq!(
        pinned,
        vec![
            &ADockEntry::Native(ANativeAction::Settings),
            &ADockEntry::Native(ANativeAction::Applications)
        ],
        "a machine with no applications pinned an application, or lost the one entry that needs \
         none"
    );
}

/// **The Dock draws the pins, and the keyboard walks the same list.**
///
/// This is the assertion the change needed and did not have. While the two
/// Dock lists were only reachable by passing a `Holding`, three call sites
/// across two files each passed `Holding::nothing()` — and putting the
/// constant back at the draw's seam left **854 of 854 tests passing**. The
/// producer was tested, the consumer was tested, and nothing tested that they
/// were joined.
///
/// A pinned application **with no window** is what makes it checkable: it is on
/// the Dock only because it was pinned, so a list built from a constant does
/// not contain it, and the count below is 0 instead of 1.
#[test]
fn what_is_pinned_reaches_both_the_draw_and_the_keyboard() {
    use crate::what_the_dock_has_open::WhatTheDockWasAskedFor as Asked;
    let (_directory, mut server) = server("pins-reach-both");

    server.its_dock_is_filled_from(&Discovery::Completed(vec![(
        ARole::Browser,
        app("org.mozilla.firefox"),
    )]));

    let drawn = server.what_this_persons_dock_holds();
    let on_the_dock: Vec<&str> = drawn
        .iter()
        .filter_map(|on| on.app().map(AppId::name))
        .collect();
    assert_eq!(
        on_the_dock,
        vec!["org.mozilla.firefox"],
        "an application this person pinned is not on the Dock that is drawn"
    );

    let grid = server.what_this_persons_more_apps_holds();
    let in_more_apps: Vec<&str> = grid
        .iter()
        .filter_map(|on| on.app().map(AppId::name))
        .collect();
    assert_eq!(
        in_more_apps,
        vec!["org.mozilla.firefox"],
        "an application this person pinned is not in the grid that is drawn"
    );

    server.the_dock_was_asked_for = Asked::MoreApps { focused: None };
    // **Against the whole grid, not the applications in it.** `in_more_apps`
    // above filters to applications so it can name them; the keyboard walks
    // every row, and a native entry is a row. Comparing against the filtered
    // list would assert that the arrows skip Settings, which is the opposite of
    // what this test is for.
    let (how_many, across) = server.what_the_keyboard_moves_through();
    assert_eq!(
        how_many,
        grid.len(),
        "the arrow keys walk a different list from the grid a person is looking at"
    );
    assert_eq!(
        grid.len(),
        3,
        "the grid is not the browser, the Settings entry and the market, so the count above \
         proves nothing"
    );
    assert!(across > 0, "a grid of no items wide");
}

/// **A person who already has a list is given it, and nothing is owed.**
///
/// The seam the whole of task 8 turns on: `its_dock_is_filled_from_what_they_kept`
/// answers [`None`] for somebody who has chosen, so the binary writes nothing
/// and the defaults cannot run a second time. A caller that wrote anyway would
/// undo a removal on every sign-in.
#[test]
fn a_person_who_has_chosen_is_given_their_list_and_owes_no_write() {
    let (_directory, mut server) = server("already-chosen");
    let theirs = vec![
        ADockEntry::Native(ANativeAction::Settings),
        ADockEntry::Application(app("org.kde.dolphin")),
    ];

    let owed = server.its_dock_is_filled_from_what_they_kept(
        &ThePins::Chosen(theirs.clone()),
        // **A discovery that found something else entirely**, so this cannot
        // pass by the two happening to agree.
        &Discovery::Completed(vec![(ARole::Browser, app("org.mozilla.firefox"))]),
    );

    assert_eq!(
        owed, None,
        "a person who already has a list was told to write one, so their removals would be undone"
    );
    let pinned: Vec<&ADockEntry> = server.what_the_dock_is_pinned_to().each_pinned().collect();
    assert_eq!(
        pinned,
        theirs.iter().collect::<Vec<_>>(),
        "a person's own arrangement was replaced by what the machine happens to have"
    );
}

/// **A fresh machine is owed exactly the list it drew.**
///
/// Asserted as an equality between what is owed and what is pinned, rather than
/// as *something is owed*: a write of some other list would satisfy the weaker
/// claim and leave the next sign-in drawing something the person never saw.
#[test]
fn a_fresh_machine_owes_the_write_it_just_drew() {
    let (_directory, mut server) = server("fresh-owes");

    let owed = server
        .its_dock_is_filled_from_what_they_kept(
            &ThePins::NeverSet,
            &Discovery::Completed(vec![(ARole::Files, app("org.kde.dolphin"))]),
        )
        .expect("a fresh machine owes its defaults to the disk");

    let pinned: Vec<ADockEntry> = server
        .what_the_dock_is_pinned_to()
        .each_pinned()
        .cloned()
        .collect();
    assert_eq!(
        owed, pinned,
        "what is owed to the disk is not what was drawn, so the next sign-in shows something else"
    );
    assert_eq!(
        owed,
        vec![
            ADockEntry::Application(app("org.kde.dolphin")),
            ADockEntry::Native(ANativeAction::Settings),
            ADockEntry::Native(ANativeAction::Applications),
        ],
        "the fresh defaults are not Files, Settings then the market"
    );
}

/// **Nothing is owed when discovery could not answer**, which is the owner's
/// fourth clause and the one that would empty a Dock for good.
///
/// Both refusals, in one test, because they differ in remedy and not in this:
/// a scan that failed is reported and one that has not finished is waited for,
/// and **neither may be written down**.
#[test]
fn a_scan_that_could_not_answer_owes_nothing() {
    for discovery in [Discovery::Failed, Discovery::StillLooking] {
        let (_directory, mut server) = server("owes-nothing");

        let owed = server.its_dock_is_filled_from_what_they_kept(&ThePins::NeverSet, &discovery);

        assert_eq!(
            owed, None,
            "{discovery:?} was written down, so a slow or broken morning becomes a Dock that is \
             empty for good"
        );
    }
}

/// **Typing narrows the grid, and the keyboard walks what is left.**
///
/// The seam this lane has now got wrong once: the draw, the hit test and the
/// arrow keys all read `what_this_persons_more_apps_holds`, so filtering there
/// is what keeps them one list. Filtering in the drawing instead left the
/// arrows walking the unfiltered grid, and **854 of 854 tests passed** with the
/// two disagreeing.
///
/// A pinned application with no window is used for the same reason as in the
/// test above: it is in the grid only because it was pinned.
#[test]
fn typing_narrows_the_grid_and_the_keyboard_walks_what_is_left() {
    use crate::what_the_dock_has_open::WhatTheDockWasAskedFor as Asked;
    let (_directory, mut server) = server("typing-narrows");
    server.its_settings_entry_is_called("Settings".to_owned());
    server.its_dock_is_filled_from(&Discovery::Completed(vec![
        (ARole::Files, app("org.kde.dolphin")),
        (ARole::Browser, app("org.mozilla.firefox")),
    ]));
    server.the_dock_was_asked_for = Asked::MoreApps { focused: None };
    assert_eq!(
        server.what_this_persons_more_apps_holds().len(),
        4,
        "the premise: two applications, the Settings entry and the market"
    );

    for letter in "fire".chars() {
        server.what_was_typed_into_more_apps.typed(letter);
    }

    let left: Vec<String> = server
        .what_this_persons_more_apps_holds()
        .iter()
        .map(|on| server.what_it_is_called(on).to_owned())
        .collect();
    assert_eq!(
        left,
        vec!["org.mozilla.firefox".to_owned()],
        "typing did not narrow the grid to what matches"
    );
    let (how_many, _) = server.what_the_keyboard_moves_through();
    assert_eq!(
        how_many, 1,
        "the arrow keys walk a different list from the grid a person is looking at"
    );
}

/// **The Settings entry is found by the name a person reads.**
///
/// It has no identifier, so the label this session was told at sign-in is the
/// only string it can be matched by. A session that was never told has a
/// Settings entry no search finds — wrong and visible, which is why the label
/// is stored rather than defaulted to something that looks like a name.
#[test]
fn the_settings_entry_is_searchable_by_its_label() {
    use crate::what_the_dock_has_open::WhatTheDockWasAskedFor as Asked;
    let (_directory, mut server) = server("settings-searchable");
    server.its_settings_entry_is_called("Ustawienia".to_owned());
    server.its_dock_is_filled_from(&Discovery::Completed(vec![(
        ARole::Files,
        app("org.kde.dolphin"),
    )]));
    server.the_dock_was_asked_for = Asked::MoreApps { focused: None };

    for letter in "ustaw".chars() {
        server.what_was_typed_into_more_apps.typed(letter);
    }

    let left = server.what_this_persons_more_apps_holds();
    // **One, not two.** This found two rows on 2026-10-17 when the market
    // joined the fresh Dock: `Server::what_it_is_called` answered the Settings
    // label for **any** entry with no application, so the market was searched
    // under Settings' name. The market has a label of its own now, and this
    // session is never told it — so a search for *ustaw* finds the one row
    // whose name this test set.
    assert_eq!(left.len(), 1, "the Polish label did not find the entry");
    assert_eq!(
        left.first().and_then(alo_dock::OnTheDock::native),
        Some(ANativeAction::Settings),
        "something other than the Settings entry matched its label"
    );
}

/// **Closing the overview ends the search.**
///
/// A person who closes it and opens it again has started a new one. Finding it
/// still narrowed by something they typed a minute ago would be a Dock that has
/// quietly hidden most of their applications, with the only clue a few letters
/// in a field they are not looking at.
#[test]
fn closing_the_overview_clears_what_was_typed() {
    use crate::what_the_dock_has_open::WhatTheDockWasAskedFor as Asked;
    let (_directory, mut server) = server("closing-clears");
    server.its_settings_entry_is_called("Settings".to_owned());
    server.its_dock_is_filled_from(&Discovery::Completed(vec![(
        ARole::Files,
        app("org.kde.dolphin"),
    )]));
    server.the_dock_was_asked_for = Asked::MoreApps { focused: None };
    for letter in "zzz".chars() {
        server.what_was_typed_into_more_apps.typed(letter);
    }
    assert!(
        server.what_this_persons_more_apps_holds().is_empty(),
        "the premise: a search that matches nothing"
    );

    server.the_overflow_closes();

    assert!(
        server.what_was_typed_into_more_apps().is_nothing(),
        "the search survived the surface closing"
    );
    assert_eq!(
        server.what_this_persons_more_apps_holds().len(),
        3,
        "the grid is still narrowed after the surface closed and opened again"
    );
}
