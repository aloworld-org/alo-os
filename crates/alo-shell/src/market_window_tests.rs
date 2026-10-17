//! What the window of what a person can install does, and every way it
//! refuses to do anything.
//!
//! The list these read is the **decided** catalogue, not a fixture, because
//! what a person sees is that list and a test against an invented one would
//! hold the window to a shape nothing ships.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;

/// The decided catalogue, as a machine has it.
fn the_catalogue() -> Catalogue {
    Catalogue::decided().unwrap()
}

/// An open window on the decided catalogue.
fn opened() -> MarketWindow {
    let mut window = MarketWindow::closed();
    window.opened(the_catalogue(), Vec::new());
    window
}

/// What is on show, or a panic saying the window was not browsing.
fn browsing(window: &MarketWindow) -> Browsing<'_> {
    match window.shows() {
        MarketShows::Browsing(browsing) => browsing,
        MarketShows::Nothing => unreachable!("the window is not browsing"),
    }
}

#[test]
fn a_closed_window_shows_nothing_and_answers_no_key() {
    let mut window = MarketWindow::closed();
    assert!(!window.is_open());
    assert_eq!(window.shows(), MarketShows::Nothing);
    for key in [
        MarketKey::Up,
        MarketKey::Down,
        MarketKey::First,
        MarketKey::Last,
        MarketKey::Before,
        MarketKey::After,
        MarketKey::Close,
        MarketKey::Nothing,
    ] {
        assert_eq!(window.pressed(key), MarketPressed::Nothing, "{key:?}");
    }
    assert_eq!(window.shows(), MarketShows::Nothing);
}

#[test]
fn an_opened_window_shows_every_application_from_the_first() {
    let window = opened();
    assert!(window.is_open());
    let on_show = browsing(&window);
    assert_eq!(on_show.under(), None);
    assert_eq!(on_show.at(), 0);
    assert_eq!(on_show.how_many(), the_catalogue().how_many());
    assert_eq!(
        on_show.whichever_they_are_on().map(|one| one.application()),
        the_catalogue().every().first().map(Listed::application)
    );
}

/// **The order is the catalogue's**, and this window does not have one of its
/// own.
#[test]
fn the_order_is_the_catalogues_own() {
    let window = opened();
    let shown: Vec<String> = browsing(&window)
        .rows()
        .map(|one| one.application().identifier().to_owned())
        .collect();
    let catalogue = the_catalogue();
    let decided: Vec<String> = catalogue
        .every()
        .iter()
        .map(|one| one.application().identifier().to_owned())
        .collect();
    assert_eq!(shown, decided);
}

#[test]
fn moving_down_and_up_walks_the_list() {
    let mut window = opened();
    assert_eq!(window.pressed(MarketKey::Down), MarketPressed::Moved);
    assert_eq!(browsing(&window).at(), 1);
    assert_eq!(window.pressed(MarketKey::Up), MarketPressed::Moved);
    assert_eq!(browsing(&window).at(), 0);
}

/// **It stops at either end rather than wrapping**, so the thing somebody was
/// about to choose does not move out from under them.
#[test]
fn moving_stops_at_each_end_and_says_nothing_happened() {
    let mut window = opened();
    assert_eq!(window.pressed(MarketKey::Up), MarketPressed::Nothing);
    assert_eq!(browsing(&window).at(), 0);

    assert_eq!(window.pressed(MarketKey::Last), MarketPressed::Moved);
    let last = browsing(&window).how_many() - 1;
    assert_eq!(browsing(&window).at(), last);
    assert_eq!(window.pressed(MarketKey::Down), MarketPressed::Nothing);
    assert_eq!(browsing(&window).at(), last);
}

#[test]
fn home_and_end_reach_the_first_and_the_last() {
    let mut window = opened();
    assert_eq!(window.pressed(MarketKey::Last), MarketPressed::Moved);
    assert_eq!(browsing(&window).at(), browsing(&window).how_many() - 1);
    assert_eq!(window.pressed(MarketKey::First), MarketPressed::Moved);
    assert_eq!(browsing(&window).at(), 0);
    // Asking for where you already are is not a move.
    assert_eq!(window.pressed(MarketKey::First), MarketPressed::Nothing);
}

/// **Narrowing wraps, and it returns to the first application**, because the
/// row somebody was on belongs to the list they have just left.
#[test]
fn narrowing_walks_the_kinds_of_work_and_wraps() {
    let mut window = opened();
    assert_eq!(browsing(&window).under(), None);

    assert_eq!(window.pressed(MarketKey::After), MarketPressed::Narrowed);
    assert_eq!(browsing(&window).under(), Category::EVERY.first().copied());

    // Back to all of them, then round the other way to the last kind.
    assert_eq!(window.pressed(MarketKey::Before), MarketPressed::Narrowed);
    assert_eq!(browsing(&window).under(), None);
    assert_eq!(window.pressed(MarketKey::Before), MarketPressed::Narrowed);
    assert_eq!(browsing(&window).under(), Category::EVERY.last().copied());
}

#[test]
fn narrowing_returns_to_the_first_application() {
    let mut window = opened();
    assert_eq!(window.pressed(MarketKey::Last), MarketPressed::Moved);
    assert_ne!(browsing(&window).at(), 0);
    assert_eq!(window.pressed(MarketKey::After), MarketPressed::Narrowed);
    assert_eq!(browsing(&window).at(), 0);
}

/// **Every kind of work has at least one application in it**, because
/// `Catalogue::read` refuses a list that leaves one empty — which is why this
/// window has no *nothing here* state to draw.
#[test]
fn every_kind_of_work_has_something_under_it() {
    let mut window = opened();
    let mut seen = 0;
    for kind in Category::EVERY {
        assert_eq!(window.pressed(MarketKey::After), MarketPressed::Narrowed);
        let on_show = browsing(&window);
        assert_eq!(on_show.under(), Some(kind), "{}", kind.named());
        assert!(on_show.how_many() > 0, "{}", kind.named());
        for one in on_show.rows() {
            assert_eq!(one.category(), kind);
        }
        seen += 1;
    }
    assert_eq!(seen, 8, "the loop walked every kind of work");
}

/// Narrowed, the window shows that kind and nothing else — and the counts of
/// every kind add up to the whole list.
#[test]
fn the_kinds_of_work_together_are_the_whole_list() {
    let mut window = opened();
    let whole = browsing(&window).how_many();
    let mut counted = 0;
    for _ in Category::EVERY {
        window.pressed(MarketKey::After);
        counted += browsing(&window).how_many();
    }
    assert_eq!(counted, whole);
}

#[test]
fn closing_empties_the_window_and_opening_starts_over() {
    let mut window = opened();
    window.pressed(MarketKey::After);
    window.pressed(MarketKey::Down);
    assert_eq!(window.pressed(MarketKey::Close), MarketPressed::Closed);
    assert!(!window.is_open());
    assert_eq!(window.shows(), MarketShows::Nothing);

    // **Opening does not remember**: a window that reopened narrowed would
    // show a short list to somebody who did not narrow it.
    window.opened(the_catalogue(), Vec::new());
    assert_eq!(browsing(&window).under(), None);
    assert_eq!(browsing(&window).at(), 0);
}

/// The first application, whatever the decided list begins with.
fn the_first() -> String {
    the_catalogue()
        .every()
        .first()
        .map(|one| one.application().identifier().to_owned())
        .unwrap_or_default()
}

/// **Asking is not installing.** The window records that the person asked and
/// hands the host what to do; nothing on the machine has changed.
#[test]
fn asking_records_the_ask_and_nothing_else() {
    let mut window = opened();
    let first = the_first();
    let one = browsing(&window).whichever_they_are_on().unwrap();
    assert_eq!(
        browsing(&window).how_it_is_going(one),
        HowItIsGoing::NotAsked
    );

    assert_eq!(
        window.pressed(MarketKey::Install),
        MarketPressed::AskedToInstall
    );
    assert_eq!(
        window
            .what_they_asked_for()
            .map(|one| one.application().identifier().to_owned()),
        Some(first)
    );
    let one = browsing(&window).whichever_they_are_on().unwrap();
    assert_eq!(browsing(&window).how_it_is_going(one), HowItIsGoing::Asked);
}

/// **One at a time.** A second ask while the first is unanswered would make
/// the first answer ambiguous, so it does nothing.
#[test]
fn a_second_ask_while_one_is_going_does_nothing() {
    let mut window = opened();
    assert_eq!(
        window.pressed(MarketKey::Install),
        MarketPressed::AskedToInstall
    );
    window.pressed(MarketKey::Down);
    assert_eq!(window.pressed(MarketKey::Install), MarketPressed::Nothing);
    assert_eq!(
        window
            .what_they_asked_for()
            .map(|one| one.application().identifier().to_owned()),
        Some(the_first())
    );
}

#[test]
fn the_host_says_it_arrived_and_the_row_says_so() {
    let mut window = opened();
    let first = the_first();
    window.pressed(MarketKey::Install);
    window.it_arrived(&first);

    assert_eq!(window.what_they_asked_for(), None);
    let one = browsing(&window).whichever_they_are_on().unwrap();
    assert_eq!(browsing(&window).how_it_is_going(one), HowItIsGoing::Here);

    // **And it cannot be asked for again**, because there is nothing to do.
    assert_eq!(window.pressed(MarketKey::Install), MarketPressed::Nothing);
}

#[test]
fn the_host_says_it_was_refused_and_the_row_keeps_the_reason() {
    let mut window = opened();
    let first = the_first();
    window.pressed(MarketKey::Install);
    let why = Stopped::Refused(alo_software::NotDone::AlreadyInstalled {
        application: first.clone(),
    });
    window.it_was_refused(&first, why.clone());

    assert_eq!(window.what_they_asked_for(), None);
    let one = browsing(&window).whichever_they_are_on().unwrap();
    assert_eq!(
        browsing(&window).how_it_is_going(one),
        HowItIsGoing::Refused(&why)
    );

    // **A refusal does not lock the row.** Asking again is allowed, and it
    // clears the old reason rather than showing it beside a new attempt.
    assert_eq!(
        window.pressed(MarketKey::Install),
        MarketPressed::AskedToInstall
    );
    let one = browsing(&window).whichever_they_are_on().unwrap();
    assert_eq!(browsing(&window).how_it_is_going(one), HowItIsGoing::Asked);
}

/// **An answer about something else is not an answer.** The host saying a
/// different application arrived leaves the ask standing.
#[test]
fn an_answer_about_another_application_is_ignored() {
    let mut window = opened();
    window.pressed(MarketKey::Install);
    window.it_arrived("org.example.SomethingElse");
    assert_eq!(
        window
            .what_they_asked_for()
            .map(|one| one.application().identifier().to_owned()),
        Some(the_first())
    );
}

/// What the machine already has says so, and is not offered again.
#[test]
fn what_is_already_here_says_so_from_the_moment_it_opens() {
    let mut window = MarketWindow::closed();
    let first = the_first();
    window.opened(the_catalogue(), vec![first.clone()]);
    let one = browsing(&window).whichever_they_are_on().unwrap();
    assert_eq!(one.application().identifier(), first);
    assert_eq!(browsing(&window).how_it_is_going(one), HowItIsGoing::Here);
    assert_eq!(window.pressed(MarketKey::Install), MarketPressed::Nothing);
}

/// **Nothing here installs.** The window records an ask and holds an answer;
/// it has no way to reach the machine, so the list it was opened on is the
/// list it still holds however many keys are pressed.
#[test]
fn no_key_changes_what_is_on_the_machine() {
    let mut window = opened();
    let before: Vec<String> = browsing(&window)
        .rows()
        .map(|one| one.application().identifier().to_owned())
        .collect();
    for key in [
        MarketKey::Up,
        MarketKey::Down,
        MarketKey::First,
        MarketKey::Last,
        MarketKey::Nothing,
    ] {
        window.pressed(key);
    }
    let after: Vec<String> = browsing(&window)
        .rows()
        .map(|one| one.application().identifier().to_owned())
        .collect();
    assert_eq!(before, after);
}
