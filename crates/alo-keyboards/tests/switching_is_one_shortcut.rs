//! **Switching keyboards is one shortcut, it is shown in the status area, and
//! it is a rented option this machine has.**
//!
//! Three things a person who types in two languages meets forty times a day,
//! and one thing they never should: a shortcut that quietly does something else
//! because they bound that chord themselves.
//!
//! The compose key is here too, for the same reason: every choice alo OS offers
//! is an option the rented keyboard data on this machine actually has, checked
//! against `/usr/share/X11/xkb/rules/evdev.lst` rather than against a list in
//! this repository.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_keyboards::{ComposeKey, Keyboards, Layout, Rented, keyboard_words, switching};
use alo_shortcuts::{Action, Chord, Key, Modifier, Modifiers, Shortcuts};
use alo_strings::{Language, Strings};

/// What this machine says, in English: this crate's words and, because the one
/// sentence about a chord fills two of its gaps from there, `alo-shortcuts`'.
fn strings() -> Strings {
    let mut vocabulary = keyboard_words().unwrap();
    alo_shortcuts::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// The machine's own list of keyboards, read.
fn this_machine() -> Rented {
    let read = Rented::read();
    assert!(read.is_ok(), "{}: {read:?}", alo_keyboards::THE_RULES);
    read.unwrap()
}

/// **One shortcut switches keyboards, and the one in use is shown.**
#[test]
fn switching_is_one_shortcut_and_the_keyboard_in_use_is_shown() {
    let rented = this_machine();
    let mut keyboards = Keyboards::offered_with(&Language::written("de").unwrap());

    // One keyboard: nothing to switch to, and nothing in the status area.
    assert_eq!(keyboards.in_the_status_area(), None);

    keyboards
        .add_for(&Language::written("el").unwrap(), &rented)
        .unwrap();
    assert_eq!(keyboards.in_the_status_area().as_deref(), Some("DE"));
    assert_eq!(keyboards.switch(), &Layout::named("gr").unwrap());
    assert_eq!(keyboards.in_the_status_area().as_deref(), Some("GR"));

    // And round again: one shortcut, not one per keyboard.
    assert_eq!(keyboards.switch(), &Layout::named("de").unwrap());
    assert_eq!(keyboards.in_the_status_area().as_deref(), Some("DE"));
}

/// **The shortcut is an option this machine's rented keyboard data has.**
///
/// The switch happens inside the rented keyboard configuration, so an option
/// name that is not in that list is a keyboard that does not load at all.
#[test]
fn the_shortcut_is_a_rented_option_this_machine_has() {
    let rented = this_machine();
    assert!(
        rented.has_option(switching::THE_OPTION),
        "{} is not an option this machine has",
        switching::THE_OPTION
    );
    let chord = switching::the_chord().unwrap();
    assert_eq!(chord.key(), Key::Space);
    assert!(chord.modifiers().holds(Modifier::Alt));
}

/// **And every compose key alo OS offers is one too.**
#[test]
fn every_compose_key_offered_is_a_rented_option_this_machine_has() {
    let rented = this_machine();
    for key in ComposeKey::ALL {
        if let Some(option) = key.option() {
            assert!(
                rented.has_option(option),
                "{key:?} is offered as {option}, which this machine does not have"
            );
        }
    }
    assert_eq!(ComposeKey::None.option(), None);
    assert_eq!(Keyboards::shipped().compose_key(), ComposeKey::None);
}

/// **Nothing the release binds is on the switch's chord**, so a person who has
/// changed nothing switches keyboards with it.
#[test]
fn nothing_the_release_binds_is_on_the_switchs_chord() {
    assert_eq!(switching::shadowed_by(&Shortcuts::shipped()), None);
}

/// **A person who has put something else on that chord is told**, rather than
/// having their own shortcut moved or the switch silently not working.
#[test]
fn a_person_who_took_the_chord_is_told_what_is_on_it() {
    let mut shortcuts = Shortcuts::shipped();
    shortcuts
        .bind(Action::MinimiseWindow, switching::the_chord().unwrap())
        .unwrap();
    assert_eq!(
        switching::shadowed_by(&shortcuts),
        Some(Action::MinimiseWindow)
    );

    let strings = strings();
    let said = switching::said_if_shadowed(&shortcuts, &strings).unwrap();
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.unfilled().is_empty(), "{said}");
    assert!(said.text().contains("Alt+Space"), "{said}");
    assert!(
        said.text()
            .contains(Action::MinimiseWindow.said(&strings).text()),
        "{said}"
    );

    // And the shortcut the person bound is still theirs: nothing here moves it.
    assert_eq!(
        shortcuts.chord_for(Action::MinimiseWindow),
        Some(switching::the_chord().unwrap())
    );
}

/// The chord this would have used is the launcher's, which is why it is not the
/// one — recorded as a test so that a later release moving the launcher is a
/// decision somebody makes rather than a coincidence.
#[test]
fn the_chord_most_systems_use_belongs_to_the_launcher_here() {
    let super_space = Chord::checked(Modifiers::just(Modifier::Super), Key::Space).unwrap();
    assert_eq!(
        Shortcuts::shipped().action_for(super_space),
        Some(Action::Launcher)
    );
    assert_ne!(switching::the_chord().unwrap(), super_space);
}
