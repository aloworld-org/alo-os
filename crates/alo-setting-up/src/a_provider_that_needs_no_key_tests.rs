//! **A service that asks for no key**, added through the form: a runtime of the
//! person's own on this machine or on their network, such as Ollama or LM
//! Studio. `docs/features.md` promises *the person's own local models and
//! runtimes* and *a compatible API … using the credentials that provider
//! requires* — which, for these, is none.
//!
//! Until this file the form refused an empty key with no way round it, so the
//! only road to such a service was editing the settings file by hand.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::{AProviderBeingAddded, NotYetAProvider, WhichField};

/// Type a whole string into the field the keyboard is in.
fn types(form: &mut AProviderBeingAddded, what: &str) {
    for letter in what.chars() {
        form.typed(letter);
    }
}

/// A runtime on this machine, filled in with the key left empty — **and the
/// switch saying so not yet pressed**.
fn a_runtime_here_with_the_key_left_empty() -> AProviderBeingAddded {
    let mut form = AProviderBeingAddded::empty();
    types(&mut form, "On this computer");
    form.to_the_next_field();
    types(&mut form, "http://127.0.0.1:11434");
    form.to_the_next_field();
    // The key, left empty: a person with no key tabs past it.
    assert_eq!(form.which_field(), WhichField::Key);
    form.to_the_next_field();
    assert_eq!(
        form.which_field(),
        WhichField::NoKeyNeeded,
        "the switch comes straight after the key it is about"
    );
    form.to_the_next_field();
    types(&mut form, "llama3.2");
    form
}

/// The same form, with the switch turned on the way a person turns it on: the
/// keyboard on it, and Space.
fn a_runtime_here_that_needs_no_key() -> AProviderBeingAddded {
    let mut form = a_runtime_here_with_the_key_left_empty();
    form.the_keyboard_is_in(WhichField::NoKeyNeeded);
    assert!(form.typed(' '), "Space on the switch changes it");
    form
}

#[test]
fn a_form_nobody_has_touched_says_a_key_is_needed() {
    // **Off unless the person turns it on.** Almost every hosted service
    // needs a key, and a switch that started on would save a provider that
    // can never answer without one, with no sentence saying why.
    let form = AProviderBeingAddded::empty();
    assert!(!form.needs_no_key());
}

#[test]
fn a_service_that_needs_no_key_is_saved_with_no_key() {
    let form = a_runtime_here_that_needs_no_key();
    assert!(form.needs_no_key());
    assert!(
        form.every_field_is_filled(),
        "an empty key is not an unfinished form once they said none is needed"
    );

    let saving = form
        .what_it_would_save()
        .expect("a service that needs no key, with every other field filled in");
    assert!(
        saving.provider.key.is_none(),
        "the provider refers to no key, which alo-choosing writes as needs-a-key = false"
    );
    assert!(
        saving.key.is_none(),
        "and nothing is handed over for the keyring"
    );
    // The control: everything else is what they typed, so the absence above
    // is the key and not a provider that was not built.
    assert_eq!(saving.provider.name, "On this computer");
    assert_eq!(saving.provider.endpoint, "http://127.0.0.1:11434");
    assert_eq!(saving.model, "llama3.2");
}

#[test]
fn an_empty_key_without_the_choice_is_still_refused() {
    // **The refusal path.** Leaving the key empty is not the same as saying a
    // service needs none: a person who forgot to paste theirs must be told,
    // not handed a provider that fails at the first question.
    let form = a_runtime_here_with_the_key_left_empty();
    assert!(!form.needs_no_key());
    assert!(
        !form.every_field_is_filled(),
        "with the switch off, the key is a field still to fill in"
    );
    let refused = form
        .what_it_would_save()
        .expect_err("no key, and none waived");
    assert!(matches!(refused, NotYetAProvider::TheKey(_)), "{refused:?}");
    assert_eq!(refused.which_field(), WhichField::Key);
}

#[test]
fn space_turns_the_switch_on_and_off_and_nothing_else_does() {
    let mut form = AProviderBeingAddded::empty();
    form.the_keyboard_is_in(WhichField::NoKeyNeeded);
    assert!(!form.typed('y'), "a letter is not a press of the switch");
    assert!(!form.needs_no_key());
    assert!(form.typed(' '));
    assert!(form.needs_no_key());
    assert!(form.typed(' '));
    assert!(
        !form.needs_no_key(),
        "and the second press turns it off again"
    );
    assert!(!form.rubbed_out(), "backspace has nothing to take back");
    assert_eq!(form.what_is_in(WhichField::NoKeyNeeded), "");
}

#[test]
fn a_space_typed_anywhere_else_is_only_a_space() {
    // The control for the test above: Space toggles the switch only while the
    // keyboard is on it, or a provider name with a space in it would flip it.
    let mut form = AProviderBeingAddded::empty();
    types(&mut form, "My own");
    assert_eq!(form.what_is_in(WhichField::Name), "My own");
    assert!(!form.needs_no_key());
}

#[test]
fn a_key_typed_before_the_switch_was_turned_on_is_not_saved_and_comes_back() {
    let mut form = a_runtime_here_with_the_key_left_empty();
    form.the_keyboard_is_in(WhichField::Key);
    types(&mut form, "sk-typed-first");
    form.the_keyboard_is_in(WhichField::NoKeyNeeded);
    form.typed(' ');

    let saving = form.what_it_would_save().unwrap();
    assert!(
        saving.key.is_none(),
        "a key they said is not needed is not kept"
    );
    assert!(saving.provider.key.is_none());
    assert!(
        !format!("{form:?}{saving:?}").contains("sk-typed-first"),
        "and neither renders it"
    );

    // Turned off again, what they typed is still theirs: nothing was thrown
    // away by a switch they may have pressed by mistake.
    form.typed(' ');
    let saving = form.what_it_would_save().unwrap();
    let kept = saving.key.expect("the key they typed, back again");
    assert_eq!(kept.filed_under.as_str(), "provider/On this computer");
}

#[test]
fn typing_into_the_key_while_none_is_needed_changes_nothing() {
    // The key row reads *not needed* while the switch is on; letters typed
    // there would be a key the person cannot see going nowhere.
    let mut form = a_runtime_here_that_needs_no_key();
    form.the_keyboard_is_in(WhichField::Key);
    assert!(!form.typed('x'));
    assert_eq!(form.what_is_in(WhichField::Key), "");
}

#[test]
fn only_the_switch_is_not_typed_into() {
    for field in WhichField::EVERY {
        assert_eq!(
            field.takes_typing(),
            field != WhichField::NoKeyNeeded,
            "{field:?}"
        );
    }
}

/// A settings file nothing else in this process is using.
fn somewhere_of_its_own(named: &str) -> std::path::PathBuf {
    crate::testing::a_folder_of_our_own(&format!("no-key-{}-{named}", std::process::id()))
        .join("settings.toml")
}

/// Write what a form would save through `alo_choosing`, the one writer of the
/// person's settings, the way `alo-desktop` does: added, then chosen.
fn written(saving: &super::WhatToSave, at: &std::path::Path) {
    let mut choosing = alo_choosing::Choosing::at(at).expect("an empty settings place");
    choosing
        .adding(saving.provider.clone())
        .expect("the provider goes on the list");
    let answer = crate::Answer::FromAProvider {
        provider: saving.provider.clone(),
        model: saving.model.clone(),
    };
    choosing
        .setting_up(answer.chosen().expect("a named provider"))
        .expect("and is chosen to answer");
}

#[test]
fn a_service_that_needs_no_key_survives_the_settings_file() {
    // **The round trip through alo-choosing**: what the form saves is written
    // as `needs-a-key = false` and read back as a provider with no key, which
    // is the reading `alo-agentd` takes to mean *ask nothing of the keyring*
    // (#1386).
    let at = somewhere_of_its_own("keyless");
    let saving = a_runtime_here_that_needs_no_key()
        .what_it_would_save()
        .unwrap();
    written(&saving, &at);

    let text = std::fs::read_to_string(&at).expect("the file that was written");
    assert!(text.contains("needs-a-key = false"), "{text}");
    let settings = alo_choosing::Settings::at(&at).expect("a file that was just written");
    let provider = settings.provider().expect("the provider that was added");
    assert_eq!(provider.name, "On this computer");
    assert!(provider.key.is_none(), "read back as needing no key");
    assert_eq!(provider.models, vec!["llama3.2".to_owned()]);
}

#[test]
fn a_service_with_a_key_still_survives_the_settings_file_needing_one() {
    // The control for the test above: the same road with a key typed and the
    // switch left off writes a provider that needs one, so the line above is
    // the switch's doing and not something the writer says for every provider.
    let at = somewhere_of_its_own("keyed");
    let mut form = a_runtime_here_with_the_key_left_empty();
    form.the_keyboard_is_in(WhichField::Key);
    types(&mut form, "sk-not-a-real-key");
    let saving = form.what_it_would_save().unwrap();
    written(&saving, &at);

    let text = std::fs::read_to_string(&at).expect("the file that was written");
    assert!(!text.contains("needs-a-key = false"), "{text}");
    assert!(
        !text.contains("sk-not-a-real-key"),
        "the key reached the file"
    );
    let settings = alo_choosing::Settings::at(&at).unwrap();
    let provider = settings.provider().unwrap();
    assert_eq!(
        provider.key.as_ref().map(alo_models::SecretRef::as_str),
        Some("provider/On this computer")
    );
}
