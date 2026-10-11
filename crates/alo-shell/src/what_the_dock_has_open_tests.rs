//! That the three states answer the three questions consistently.
#![expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected value is the failure being reported, and a \
              formatted panic names the state it was in"
)]

use super::WhatTheDockHasOpen;

/// An application, for a state that needs one.
fn app() -> alo_dock::AppId {
    alo_dock::AppId::named("an.app").unwrap()
}

/// A window list of two windows of that application.
///
/// Two rather than one, because a picker of one window is not worth showing and
/// a state built from one would be testing the wrong thing.
fn a_picker() -> alo_dock::picking::ThePicker {
    let mut windows = alo_dock::Windows::none();
    for number in 1..=2 {
        windows.opened(alo_dock::Window::of(
            alo_dock::WindowId::numbered(number),
            Some(app()),
            "A window",
            alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 800, 600).unwrap(),
            alo_dock::HowItSits::OnTheCanvas,
        ));
    }
    alo_dock::picking::ThePicker::of(
        alo_dock::previews::Previews::of(
            &windows,
            &app(),
            alo_dock::previews::Opened::ByPressingTheIcon,
        ),
        alo_appearance::TextScale::default(),
        alo_dock::Room::pixels(1_000),
    )
}

/// **Every state against every question**, as one table.
///
/// Three accessors over three states is nine answers, and the ones worth
/// holding are the pairs that must never both be true: nothing is ever both
/// *More apps* and a picker, and `is_anything` agrees with the other two rather
/// than being a fourth opinion.
#[test]
fn each_state_answers_every_question() {
    let cases = [
        (WhatTheDockHasOpen::Nothing, false, false, false),
        (WhatTheDockHasOpen::MoreApps, true, false, true),
        (
            WhatTheDockHasOpen::TheWindowsOf(a_picker()),
            false,
            true,
            true,
        ),
    ];
    for (state, more_apps, a_picker, anything) in cases {
        assert_eq!(state.is_more_apps(), more_apps, "{state:?}: is_more_apps");
        assert_eq!(
            state.the_windows_of().is_some(),
            a_picker,
            "{state:?}: the_windows_of"
        );
        assert_eq!(state.is_anything(), anything, "{state:?}: is_anything");
        assert!(
            !(state.is_more_apps() && state.the_windows_of().is_some()),
            "{state:?}: both surfaces at once"
        );
        assert_eq!(
            state.is_anything(),
            state.is_more_apps() || state.the_windows_of().is_some(),
            "{state:?}: is_anything disagrees with the other two"
        );
    }
}

/// **A frame starts with neither open**, which is what every fixture and every
/// first draw relies on.
#[test]
fn the_default_is_nothing() {
    assert_eq!(WhatTheDockHasOpen::default(), WhatTheDockHasOpen::Nothing);
}

/// **The state carries the rows**, which is what a drawing needs: each
/// window's name and whether it was put aside, neither of which is on the
/// Dock's own list of applications.
#[test]
fn a_picker_carries_the_rows_a_drawing_needs() {
    let open = WhatTheDockHasOpen::TheWindowsOf(a_picker());
    assert_eq!(
        open.the_windows_of().map(|picker| picker.rows().len()),
        Some(2),
        "the state did not carry the rows the draw needs"
    );
}

/// `how_many` windows of our application, plus one of another.
///
/// The other application is there so a test about *this* one cannot pass by
/// counting everything open.
fn windows_of(how_many: u64) -> alo_dock::Windows {
    let mut windows = alo_dock::Windows::none();
    for number in 1..=how_many {
        windows.opened(alo_dock::Window::of(
            alo_dock::WindowId::numbered(number),
            Some(app()),
            "A window",
            alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 800, 600).unwrap(),
            alo_dock::HowItSits::OnTheCanvas,
        ));
    }
    windows.opened(alo_dock::Window::of(
        alo_dock::WindowId::numbered(900),
        Some(alo_dock::AppId::named("another.app").unwrap()),
        "Somebody else's window",
        alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 800, 600).unwrap(),
        alo_dock::HowItSits::OnTheCanvas,
    ));
    windows
}

fn now(asked: &super::WhatTheDockWasAskedFor, how_many: u64) -> WhatTheDockHasOpen {
    super::what_that_means_now(
        asked,
        &windows_of(how_many),
        alo_appearance::TextScale::default(),
        alo_dock::Room::pixels(10_000),
    )
}

/// **What the person asked for decides what is drawn**, and the two states that
/// are not about an application pass straight through.
#[test]
fn nothing_and_more_apps_are_what_they_were_asked_for() {
    use super::WhatTheDockWasAskedFor as Asked;
    assert_eq!(now(&Asked::Nothing, 3), WhatTheDockHasOpen::Nothing);
    assert_eq!(now(&Asked::MoreApps, 3), WhatTheDockHasOpen::MoreApps);
}

/// **A chosen application becomes its own windows**, and only its own.
#[test]
fn a_chosen_application_becomes_its_windows_and_no_others() {
    use super::WhatTheDockWasAskedFor as Asked;
    let open = now(&Asked::TheWindowsOf(app()), 3);
    let picker = open
        .the_windows_of()
        .unwrap_or_else(|| panic!("the list did not open: {open:?}"));
    assert_eq!(
        picker.rows().len(),
        3,
        "another application's window got in"
    );
}

/// **A list with nothing left to choose between is closed, not drawn empty.**
///
/// The windows of the application a person chose can close while its list is
/// up. One window left is a list with no choice in it — `alo_dock::clicking`
/// would not have opened one for a single window — and closing costs them
/// nothing, because the window they are left with is the one a plain press
/// would now go to.
///
/// Both ends are checked: two windows still open the list, so this is not a
/// test that would pass against a conversion that never opens one.
#[test]
fn a_list_that_has_nothing_left_to_choose_between_closes() {
    use super::WhatTheDockWasAskedFor as Asked;
    let asked = Asked::TheWindowsOf(app());
    assert_eq!(
        now(&asked, 1),
        WhatTheDockHasOpen::Nothing,
        "one window left a list open with nothing to choose in it"
    );
    assert_eq!(
        now(&asked, 0),
        WhatTheDockHasOpen::Nothing,
        "an application whose windows all closed left a list open"
    );
    assert!(
        now(&asked, 2).the_windows_of().is_some(),
        "two windows should still open the list"
    );
}

/// **It is worked out again every time**, so a window closing changes the
/// answer without anything being told to refresh.
///
/// The fault this guards is a cache: rows remembered from when the list was
/// opened would still name a window that has gone, and pressing that row would
/// reach nothing. Asked of one `asked` value against two different window
/// lists, because that is exactly what a frame does twice in a row.
#[test]
fn the_rows_follow_the_windows_rather_than_being_remembered() {
    use super::WhatTheDockWasAskedFor as Asked;
    let asked = Asked::TheWindowsOf(app());

    let before = now(&asked, 4);
    let after = now(&asked, 2);

    assert_eq!(
        before.the_windows_of().map(|it| it.rows().len()),
        Some(4),
        "four windows"
    );
    assert_eq!(
        after.the_windows_of().map(|it| it.rows().len()),
        Some(2),
        "two of them closed and the list did not notice"
    );
}

/// **Escape goes back before it goes out**, and is not the Dock's when nothing
/// is open.
///
/// The owner's ruling: *Escape dismisses the current picker first, then the
/// parent popover.* Walked as a sequence rather than as three separate cases,
/// because the thing being asserted is the order — a rule that closed
/// everything at once would pass three independent checks of "it closed".
#[test]
fn escape_goes_back_then_out_then_belongs_to_the_application() {
    use super::WhatTheDockWasAskedFor as Asked;

    let picker = Asked::TheWindowsOf(app());
    let back = picker
        .when_escape_is_pressed()
        .unwrap_or_else(|| panic!("Escape did nothing to an open window list"));
    assert_eq!(
        back,
        Asked::MoreApps,
        "Escape closed everything instead of going back to the overview"
    );

    let out = back
        .when_escape_is_pressed()
        .unwrap_or_else(|| panic!("Escape did nothing to the overview"));
    assert_eq!(out, Asked::Nothing);

    assert_eq!(
        out.when_escape_is_pressed(),
        None,
        "the Dock took Escape with nothing open, so no application can be \
         cancelled while alo OS is running"
    );
}

/// **It takes two presses to leave a window list**, which is the whole of the
/// ruling and is easy to lose to a tidier implementation.
#[test]
fn leaving_a_window_list_takes_two_presses() {
    use super::WhatTheDockWasAskedFor as Asked;
    let mut open = Asked::TheWindowsOf(app());
    let mut presses = 0;
    while let Some(next) = open.when_escape_is_pressed() {
        open = next;
        presses += 1;
        assert!(presses < 5, "Escape never settled");
    }
    assert_eq!(open, Asked::Nothing);
    assert_eq!(presses, 2, "a window list should take back, then out");
}
