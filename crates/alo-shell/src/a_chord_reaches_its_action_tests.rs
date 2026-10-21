//! The one fact this road depends on that lives in another crate.
//!
//! The road itself is walked in `tests/a_chord_reaches_its_action`, which
//! presses evdev codes against a real display and asks what the canvas did.
//! Nothing is repeated here: a second test of the road that did not press a key
//! would be exactly the kind of green that task 19 was written about.

use super::*;
use alo_shortcuts::{Action, Key, Modifier, Modifiers};

/// **The shipped chord for *Show all* is one this road can make from a key.**
///
/// `the_chord_a_key_makes` reads the **unshifted** symbol, which is what a
/// person's key cap says. That makes a digit reachable on every layout and
/// makes `⊞`+`+` reachable only where `+` is a key of its own — so a shipped
/// default moved onto a shifted symbol would quietly stop arriving on an
/// American keyboard.
///
/// Asserted against `alo-shortcuts` rather than written out, so a default that
/// moves moves this with it rather than leaving it asserting a key nobody has.
#[test]
fn show_all_ships_with_a_chord_an_unshifted_key_can_make() {
    let shipped = Shortcuts::shipped();
    let made = Chord::checked(Modifiers::just(Modifier::Super), Key::Digit0)
        .expect("Super and a digit is a chord");
    assert_eq!(
        shipped.chord_for(Action::ShowAllOnTheCanvas),
        Some(made),
        "the shipped Show all chord is no longer ⊞+0"
    );
    assert_eq!(
        shipped.action_for(made),
        Some(Action::ShowAllOnTheCanvas),
        "the chord this road makes does not reach the action"
    );
}

/// **Every action this release declares has a road, and none answers
/// `NoRoad`.**
///
/// This is the check that would have caught four dead chords, and it is written
/// as *press each one and see what comes back* rather than as a grep over match
/// arms, because a grep over match arms is what missed them: `Action::TheAgent`
/// appears sixty times in this workspace — in bindings, words, translations,
/// refusal messages and tests — and in **zero** dispatch arms. Mentions and
/// roads are different populations, and only one of them is a promise.
///
/// # What the three expected answers mean
///
/// - `Carried` — a road did it.
/// - `Refused` — a road answered and said no, which is ordinary: a session
///   built here has no output and no focused window, so most actions refuse.
///   **That it refused is the proof the arm exists**, because an action with no
///   arm cannot reach a refusal.
/// - `NotBound` — impossible here, since each action is bound before it is
///   pressed, and asserted against so a binding that silently failed does not
///   read as a pass.
///
/// `NoRoad` is the failure, and it names itself.
///
/// # Why each action is bound to a chord of this test's own
///
/// Reading the shipped chord would leave this test measuring only the actions
/// the release happens to bind. Binding every one of them to the same free
/// chord in turn presses **all** of `Action::ALL`, including any added without
/// a default — which is exactly the shape a new dead action would arrive in.
#[test]
fn every_action_this_release_declares_has_a_road() {
    use std::os::unix::fs::PermissionsExt as _;

    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");

    // One chord, rebound to each action in turn. Alt and a function key, which
    // `Chord::checked` admits and nothing in the release ships.
    let chord = Chord::checked(Modifiers::just(Modifier::Alt), Key::F9)
        .expect("Alt and a function key is a chord");
    assert!(
        Shortcuts::shipped().action_for(chord).is_none(),
        "the release now binds Alt+F9, so this test would measure that binding instead"
    );

    let mut without_a_road = Vec::new();
    let mut carried = 0;
    let mut refused = 0;
    for action in Action::ALL {
        let mut server = Server::bind(runtime.path(), &format!("road-{action:?}"))
            .expect("a display for this action");
        // **Told where settings are kept, so Settings takes its real road.** A
        // session that is never told skips `dispatch_settings_command`
        // entirely, and the layers below then answer *nothing carries Settings
        // out* — which is true of that session and not of this compositor. The
        // first run of this test reported `[Launcher, Settings]` for exactly
        // that reason, and `Settings` was never the gap.
        server.the_settings_places_are(crate::SettingsPlaces::of(
            Some(runtime.path().as_os_str()),
            Some(runtime.path().as_os_str()),
            None,
            &runtime.path().join("grants"),
            &runtime.path().join("pairings"),
            &runtime.path().join("daemon"),
        ));
        let mut shortcuts = Shortcuts::shipped();
        shortcuts.unbind(*action);
        shortcuts
            .bind(*action, chord)
            .expect("one free chord binds to any action");
        server.the_shortcuts_are(shortcuts);

        match server.the_chord_does_what_it_names(chord) {
            WhatTheChordCameTo::Carried(which) => {
                assert_eq!(which, *action, "a road carried out the wrong action");
                carried += 1;
            }
            WhatTheChordCameTo::Refused => refused += 1,
            WhatTheChordCameTo::NoRoad(which) => without_a_road.push(which),
            WhatTheChordCameTo::NotBound => {
                panic!("{action:?} was bound to {chord:?} and the road says nothing is bound to it")
            }
        }
    }

    // **No exceptions, and the assertion is equality rather than containment**,
    // so the list cannot grow quietly.
    //
    // **`Action::Launcher` was the one entry here until 2026-10-20.** Its
    // rustdoc carried the reason: `Super`+`Space` was bound and *which surface
    // the launcher is* had never been decided — the Dock's overview, which
    // cannot start an application nobody has started, or the alo Bar. The
    // owner settled it, in their words, *it is already on the dock*, and
    // `crate::a_chord_opens_the_bar` is the arm. **The exception was a
    // permission and it has been spent**, which is what this line being empty
    // now means: nothing may be declared, bound, worded and translated with no
    // arm in the chain, and there is no precedent here to cite.
    assert_eq!(
        without_a_road,
        Vec::new(),
        "the set of declared actions that reach nothing has changed. Anything new here was \
         shipped, worded, translated and bound with no arm in the chain — build it, or if the \
         surface it needs is genuinely undecided, say so on the action and add it here"
    );
    // **A control, so a pass means the road ran.** Without this the test would
    // also pass if every action somehow answered `NotBound` and the loop never
    // reached a road at all — the shape of green that proves nothing.
    assert_eq!(
        carried + refused + without_a_road.len(),
        Action::ALL.len(),
        "the loop did not account for every declared action"
    );
    assert!(
        carried > 0 && refused > 0,
        "every action gave the same answer ({carried} carried, {refused} refused), which \
         means this test is measuring one road rather than all of them"
    );
}
