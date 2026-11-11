//! What a reader is told about a band, held to the specification's names.

use super::*;

/// **Window actions, Minimise, Close** — the specification's three, in the
/// order they are drawn.
#[test]
fn a_shell_drawn_window_has_the_three_in_their_drawn_order() {
    assert_eq!(
        the_controls_of(Decorations::TheShellDraws),
        vec![
            OnTheEdge::WindowActions,
            OnTheEdge::Minimise,
            OnTheEdge::Close
        ]
    );
}

/// **The names are the specification's, and the reader says them whatever is
/// drawn** — a concealed band still has all three.
#[test]
fn a_reader_hears_the_three_by_their_own_names_without_hovering() {
    let names: Vec<&str> = what_a_reader_is_told(Decorations::TheShellDraws)
        .iter()
        .map(|it| it.name.says())
        .collect();
    assert_eq!(names, vec!["Window actions", "Minimise", "Close"]);
}

/// **Window actions is told as a control that opens something**, so a reader
/// can be told whether its panel is expanded; the other two are plain buttons.
#[test]
fn window_actions_opens_something_and_the_others_do_not() {
    let told = what_a_reader_is_told(Decorations::TheShellDraws);
    let states: Vec<alo_access::State> = told.iter().map(|it| it.state).collect();
    assert_eq!(
        states,
        vec![
            alo_access::State::OpensSomething,
            alo_access::State::CanBeUsed,
            alo_access::State::CanBeUsed
        ]
    );
    assert!(told.iter().all(alo_access::tree::Control::can_be_used));
}

/// What an application that draws these buttons of its own is given.
fn given_to(minimise: bool, close: bool) -> Vec<OnTheEdge> {
    the_controls_of(Decorations::TheApplicationDraws(
        crate::window_edge::ItsOwnButtons { minimise, close },
    ))
}

/// **An application that shows Minimise and Close is given Window actions
/// alone**: no second Minimise or Close beside its own. VSCodium and the GNOME
/// applications, as measured.
#[test]
fn an_application_with_both_of_its_own_is_never_given_a_second() {
    assert_eq!(given_to(true, true), vec![OnTheEdge::WindowActions]);
}

/// **An application that shows Close and no Minimise is given Minimise**, and
/// still no second Close. Firefox, as measured. The owner, 2026-11-12: *we can
/// only add the ones that are not there.*
#[test]
fn an_application_with_only_its_own_close_is_given_minimise_and_no_close() {
    assert_eq!(
        given_to(false, true),
        vec![OnTheEdge::WindowActions, OnTheEdge::Minimise]
    );
}

/// **An application that shows neither gets all three**, in the order the
/// shell's band draws them, so a window is never short of a way to close.
#[test]
fn an_application_with_neither_of_its_own_is_given_all_three() {
    assert_eq!(given_to(false, false), the_controls_of(Decorations::TheShellDraws));
    assert_eq!(
        given_to(true, false),
        vec![OnTheEdge::WindowActions, OnTheEdge::Close]
    );
}

/// **Where alo cannot know, Window actions and nothing else**: an unknown
/// application is never given a duplicate on a guess.
#[test]
fn an_application_alo_cannot_know_is_given_window_actions_only() {
    assert_eq!(
        the_controls_of(Decorations::TheApplicationDraws(
            crate::window_edge::ItsOwnButtons::NOT_KNOWN
        )),
        vec![OnTheEdge::WindowActions]
    );
}

/// **Window actions in every decoration mode where the window can be
/// placed.** No application draws alo's panel, so it is never left out for
/// one, whatever buttons that application has.
#[test]
fn window_actions_is_added_whatever_the_application_draws() {
    for (minimise, close) in [(true, true), (true, false), (false, true), (false, false)] {
        assert_eq!(given_to(minimise, close).first(), Some(&OnTheEdge::WindowActions));
    }
    assert_eq!(
        the_controls_of(Decorations::TheShellDraws).first(),
        Some(&OnTheEdge::WindowActions)
    );
}

/// **Each control does what the chord of the same name does**, so a press and
/// a key are one action.
#[test]
fn each_control_is_the_action_its_chord_performs() {
    assert_eq!(
        what_it_does(OnTheEdge::WindowActions),
        Action::WindowOptions
    );
    assert_eq!(what_it_does(OnTheEdge::Minimise), Action::MinimiseWindow);
    assert_eq!(what_it_does(OnTheEdge::Close), Action::CloseWindow);
}

/// **The words a band says are exactly the three**, which is what
/// `alo-access` counts as said by this crate's surface.
#[test]
fn the_band_says_exactly_three_words() {
    let keys: Vec<String> = every_word_this_band_says()
        .iter()
        .map(|it| it.named().to_owned())
        .collect();
    assert_eq!(
        keys,
        vec!["access.window-actions", "access.minimise", "access.close"]
    );
}
