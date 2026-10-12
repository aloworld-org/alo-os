//! The membership rule: who is on the Dock, and who is only in the panel.
//!
//! The owner's ruling of 2026-10-11, ADR 0103. Every row of its table is a test
//! here, because the table is the specification and a rule tested on the cases
//! somebody happened to think of is a rule with holes in the shape of the ones
//! they did not.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::*;
use crate::on_the_canvas::{Patch, Spot};
use crate::window::{HowItSits, Window, WindowId};

fn app(name: &str) -> AppId {
    AppId::named(name).unwrap()
}

fn window(number: u64, of: &str, sits: HowItSits) -> Window {
    Window::of(
        WindowId::numbered(number),
        Some(app(of)),
        "A window",
        Patch::of(Spot::at(0, 0), 800, 600).unwrap(),
        sits,
    )
}

fn names(shown: &[OnTheDock]) -> Vec<String> {
    shown
        .iter()
        .filter_map(|on| on.app().map(|app| app.name().to_owned()))
        .collect()
}

/// **Every row of the owner's table, in one test**, so a reader comparing the
/// code with the ruling compares two tables rather than six test names.
///
/// Each case names its own pinning and its own windows, which is the whole of
/// what the rule reads.
#[test]
fn every_row_of_the_membership_table() {
    // (what the case is, pinned, windows as they sit, on the Dock?)
    let cases: &[(&str, bool, &[HowItSits], bool)] = &[
        ("pinned, nothing open", true, &[], true),
        (
            "pinned, a window that is not minimised",
            true,
            &[HowItSits::OnTheCanvas],
            true,
        ),
        (
            "pinned, every window minimised",
            true,
            &[HowItSits::PutAside, HowItSits::PutAside],
            true,
        ),
        (
            "unpinned, a window that is not minimised",
            false,
            &[HowItSits::OnTheCanvas],
            true,
        ),
        (
            "unpinned, one of two minimised",
            false,
            &[HowItSits::PutAside, HowItSits::OnTheCanvas],
            true,
        ),
        (
            "unpinned, every window minimised",
            false,
            &[HowItSits::PutAside, HowItSits::PutAside],
            false,
        ),
        ("unpinned, nothing open", false, &[], false),
    ];

    for (what, pinned, sits, belongs) in cases {
        let mut holding = Holding::nothing();
        if *pinned {
            holding.pin(app("Notes"));
        }
        let mut windows = Windows::none();
        for (number, how) in sits.iter().enumerate() {
            windows.opened(window(number as u64, "Notes", *how));
        }
        let shown = holding.showing(&windows);
        assert_eq!(
            shown
                .iter()
                .any(|on| on.app().is_some_and(|app| app.name() == "Notes")),
            *belongs,
            "{what}: the Dock showed {:?}",
            names(&shown)
        );
    }
}

/// **A window filling the screen is not minimised**, which the table's wording
/// does not spell out and the enum does.
///
/// `HowItSits` has three answers and only one of them is *put aside*. A rule
/// written as `!matches!(how, OnTheCanvas)` would pass every case above and
/// fail this one, which is exactly why it is here.
#[test]
fn a_window_filling_the_screen_keeps_its_unpinned_application_on_the_dock() {
    let holding = Holding::nothing();
    let mut windows = Windows::none();
    windows.opened(window(1, "Player", HowItSits::FillingTheScreen));

    assert_eq!(
        names(&holding.showing(&windows)),
        vec!["Player".to_owned()],
        "a window filling the screen was treated as one put aside"
    );
}

/// **Not minimised is about the window, never about the camera.**
///
/// This type has never seen a camera and cannot ask where the view is, so the
/// mistake is unreachable rather than merely untaken — which is the architecture
/// doing the work. What this test can show is the half that *is* reachable: a
/// window far outside any plausible view, on a patch nobody is looking at, keeps
/// its application on the Dock.
#[test]
fn a_window_nobody_is_looking_at_is_still_not_minimised() {
    let holding = Holding::nothing();
    let mut windows = Windows::none();
    windows.opened(Window::of(
        WindowId::numbered(1),
        Some(app("Notes")),
        "A window",
        Patch::of(Spot::at(100_000, -100_000), 800, 600).unwrap(),
        HowItSits::OnTheCanvas,
    ));

    assert_eq!(
        names(&holding.showing(&windows)),
        vec!["Notes".to_owned()],
        "a window far from the view was treated as one put aside"
    );
}

/// **One icon per application, however many windows it has.**
#[test]
fn an_application_with_several_windows_is_one_icon() {
    let holding = Holding::nothing();
    let mut windows = Windows::none();
    for number in 0..5 {
        windows.opened(window(number, "Notes", HowItSits::OnTheCanvas));
    }

    assert_eq!(names(&holding.showing(&windows)), vec!["Notes".to_owned()]);
}

/// **Minimising the last one takes the icon away; restoring any brings it
/// back**, which is the owner's worked example in §4 and is about a *sequence*
/// rather than a state.
///
/// Asserted as four readings of one Dock as the windows change, because the
/// fault this guards against — a membership computed once and cached — passes
/// every single-state test above.
#[test]
fn the_icon_goes_when_the_last_window_is_minimised_and_returns_when_one_comes_back() {
    let holding = Holding::nothing();
    let mut windows = Windows::none();
    windows.opened(window(1, "Notes", HowItSits::OnTheCanvas));
    windows.opened(window(2, "Notes", HowItSits::OnTheCanvas));
    assert_eq!(names(&holding.showing(&windows)), vec!["Notes".to_owned()]);

    windows.opened(window(1, "Notes", HowItSits::PutAside));
    assert_eq!(
        names(&holding.showing(&windows)),
        vec!["Notes".to_owned()],
        "the icon went when one of two windows was minimised"
    );

    windows.opened(window(2, "Notes", HowItSits::PutAside));
    assert!(
        names(&holding.showing(&windows)).is_empty(),
        "the icon stayed after every window was minimised"
    );

    windows.opened(window(2, "Notes", HowItSits::OnTheCanvas));
    assert_eq!(
        names(&holding.showing(&windows)),
        vec!["Notes".to_owned()],
        "the icon did not come back when a window was restored"
    );
}

/// **A pinned application lives through the same sequence untouched**, which is
/// the other half of the owner's example and the one a filter is most likely to
/// break.
#[test]
fn a_pinned_application_keeps_its_icon_through_every_window_being_minimised() {
    let mut holding = Holding::nothing();
    holding.pin(app("Files"));
    let mut windows = Windows::none();
    windows.opened(window(1, "Files", HowItSits::OnTheCanvas));
    windows.opened(window(2, "Files", HowItSits::PutAside));
    assert_eq!(names(&holding.showing(&windows)), vec!["Files".to_owned()]);

    windows.opened(window(1, "Files", HowItSits::PutAside));
    assert_eq!(
        names(&holding.showing(&windows)),
        vec!["Files".to_owned()],
        "a pinned application left the Dock when its windows were put aside"
    );
}

/// **A pinned application keeps its place when it opens**, and the rest follow
/// in opening order.
#[test]
fn pinned_applications_keep_their_places_and_the_rest_follow_in_opening_order() {
    let mut holding = Holding::nothing();
    holding.pin(app("Files"));
    holding.pin(app("Browser"));
    let mut windows = Windows::none();
    windows.opened(window(1, "Notes", HowItSits::OnTheCanvas));
    windows.opened(window(2, "Browser", HowItSits::OnTheCanvas));
    windows.opened(window(3, "Player", HowItSits::OnTheCanvas));

    assert_eq!(
        names(&holding.showing(&windows)),
        vec![
            "Files".to_owned(),
            "Browser".to_owned(),
            "Notes".to_owned(),
            "Player".to_owned(),
        ],
        "Browser opened and left its pinned place, or the rest are not in \
         opening order"
    );
}

/// **Focusing an application does not reorder the Dock.**
///
/// The clause the plan singles out as the easy one to leave untested, because a
/// Dock drawn once looks right whatever the order rule is. It needs two
/// readings with a focus between them.
///
/// `Windows` keeps two orders on purpose — `in_use_order` for *which window a
/// click goes to* and `in_open_order` for *where an icon sits* — and
/// [`Holding::showing`] walks the second. This test is what holds those apart:
/// a `showing` that walked `each_as_used` would pass every other test in this
/// file.
#[test]
fn focusing_an_application_does_not_move_its_icon() {
    let holding = Holding::nothing();
    let mut windows = Windows::none();
    windows.opened(window(1, "Notes", HowItSits::OnTheCanvas));
    windows.opened(window(2, "Browser", HowItSits::OnTheCanvas));
    windows.opened(window(3, "Player", HowItSits::OnTheCanvas));
    let before = names(&holding.showing(&windows));

    assert!(
        windows.used(WindowId::numbered(3)),
        "the premise: it is open"
    );

    assert_eq!(
        names(&holding.showing(&windows)),
        before,
        "focusing the last application moved its icon; the Dock was {before:?}"
    );
}

/// **An application the Dock does not show is still in *More apps*.**
///
/// The one case the two lists are guaranteed to differ on, and the reason
/// `in_the_overview` exists rather than *More apps* walking its own list: an
/// unpinned application whose windows are all minimised is gone from the bar
/// by the membership rule and is, by the owner's ruling, *another route to
/// them* in the overview.
///
/// Asserted on both lists from one state, because *the Dock is right* and *the
/// overview is right* are only interesting together.
#[test]
fn an_application_with_only_minimised_windows_leaves_the_dock_and_stays_in_more_apps() {
    let holding = Holding::nothing();
    let mut windows = Windows::none();
    windows.opened(window(1, "Notes", HowItSits::PutAside));
    windows.opened(window(2, "Notes", HowItSits::PutAside));

    assert!(
        names(&holding.showing(&windows)).is_empty(),
        "it should have left the Dock"
    );
    assert_eq!(
        names(&holding.in_the_overview(&windows)),
        vec!["Notes".to_owned()],
        "and More apps should still offer a route to it"
    );
}

/// **The Dock is the overview with the membership rule applied**, which is the
/// relationship the two functions are written to guarantee.
///
/// Over a spread of states rather than one, because the claim is about the two
/// lists as a whole: same order, same rows, minus exactly what does not belong.
/// A second walk that happened to agree on a pinned application and an open one
/// would pass a single-case test.
#[test]
fn the_dock_is_the_overview_minus_what_does_not_belong() {
    let mut holding = Holding::nothing();
    holding.pin(app("Files"));
    holding.pin(app("Browser"));

    let mut windows = Windows::none();
    windows.opened(window(1, "Browser", HowItSits::PutAside));
    windows.opened(window(2, "Notes", HowItSits::OnTheCanvas));
    windows.opened(window(3, "Player", HowItSits::PutAside));
    windows.opened(window(4, "Notes", HowItSits::PutAside));
    windows.opened(window(5, "Reader", HowItSits::FillingTheScreen));

    let overview = holding.in_the_overview(&windows);
    let expected: Vec<String> = overview
        .iter()
        .filter(|on| on.belongs_on_the_dock())
        .filter_map(|on| on.app().map(|app| app.name().to_owned()))
        .collect();

    assert_eq!(names(&holding.showing(&windows)), expected);

    // And the premise: the two are not the same list, or this proves nothing.
    assert_eq!(
        names(&overview),
        vec![
            "Files".to_owned(),
            "Browser".to_owned(),
            "Notes".to_owned(),
            "Player".to_owned(),
            "Reader".to_owned(),
        ],
        "the overview is not pinned-then-opening-order"
    );
    assert_eq!(
        names(&holding.showing(&windows)),
        vec![
            "Files".to_owned(),
            "Browser".to_owned(),
            "Notes".to_owned(),
            "Reader".to_owned(),
        ],
        "Player had only a minimised window and should have left the Dock"
    );
}

/// **A pinned application with nothing open is in the overview**, which is the
/// other direction: the overview is not *what is open*.
#[test]
fn a_pinned_application_with_nothing_open_is_in_the_overview() {
    let mut holding = Holding::nothing();
    holding.pin(app("Files"));

    assert_eq!(
        names(&holding.in_the_overview(&Windows::none())),
        vec!["Files".to_owned()]
    );
}
