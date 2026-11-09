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

/// **An application with its own header is given Window actions alone** — no
/// second Close beside its own, which is the refusal the 2026-10-09 ruling
/// asks for.
#[test]
fn an_application_with_its_own_header_is_never_given_a_second_close() {
    let has = the_controls_of(Decorations::TheApplicationDraws);
    assert_eq!(has, vec![OnTheEdge::WindowActions]);
    assert!(!has.contains(&OnTheEdge::Close));
    assert!(!has.contains(&OnTheEdge::Minimise));
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
