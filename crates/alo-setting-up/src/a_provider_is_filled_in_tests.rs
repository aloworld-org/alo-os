//! A form typed into, and read back as what it would save.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_models::ProviderError;

use super::{AProviderBeingAddded, NotYetAProvider, WhichField};

/// Type a whole string into the field the keyboard is in.
fn types(form: &mut AProviderBeingAddded, what: &str) {
    for letter in what.chars() {
        form.typed(letter);
    }
}

/// A form filled in the way a person adding OpenAI would fill it.
fn one_a_person_filled_in() -> AProviderBeingAddded {
    let mut form = AProviderBeingAddded::empty();
    types(&mut form, "OpenAI");
    form.to_the_next_field();
    types(&mut form, "https://api.openai.com");
    form.to_the_next_field();
    types(&mut form, "sk-not-a-real-key");
    form.to_the_next_field();
    types(&mut form, "gpt-4o-mini");
    form
}

#[test]
fn a_form_nobody_has_touched_is_empty_and_starts_at_the_first_field() {
    let form = AProviderBeingAddded::empty();
    assert_eq!(form.which_field(), WhichField::Name);
    for which in WhichField::EVERY {
        assert_eq!(form.what_is_in(which), "");
    }
    assert!(!form.every_field_is_filled());
}

#[test]
fn what_is_typed_lands_in_the_field_the_keyboard_is_in() {
    let mut form = AProviderBeingAddded::empty();
    types(&mut form, "OpenAI");
    assert_eq!(form.what_is_in(WhichField::Name), "OpenAI");
    // The controls at the other end: the other three took none of it, so the
    // assertion above is this field and not every field.
    assert_eq!(form.what_is_in(WhichField::Address), "");
    assert_eq!(form.what_is_in(WhichField::Key), "");
    assert_eq!(form.what_is_in(WhichField::Model), "");
}

#[test]
fn moving_on_puts_the_next_letters_in_the_next_field() {
    let mut form = AProviderBeingAddded::empty();
    types(&mut form, "OpenAI");
    assert!(form.to_the_next_field());
    types(&mut form, "https://api.openai.com");
    assert_eq!(form.what_is_in(WhichField::Name), "OpenAI");
    assert_eq!(
        form.what_is_in(WhichField::Address),
        "https://api.openai.com"
    );
}

#[test]
fn the_last_field_does_not_wrap_round_to_the_first() {
    // Tab at the end of a form that jumps back to the top is a person losing
    // their place, and what they most likely wanted was to save.
    let mut form = AProviderBeingAddded::empty();
    form.the_keyboard_is_in(WhichField::Model);
    assert!(!form.to_the_next_field());
    assert_eq!(form.which_field(), WhichField::Model);

    form.the_keyboard_is_in(WhichField::Name);
    assert!(!form.to_the_field_before());
    assert_eq!(form.which_field(), WhichField::Name);
}

#[test]
fn one_backspace_takes_back_one_letter_a_reader_would_call_one() {
    // The i18n law reaching somewhere that is not a translated string: a name
    // in Devanagari has a first `char` that is half a letter, so a backspace
    // counting `char`s would leave a fragment on the screen.
    let mut form = AProviderBeingAddded::empty();
    types(&mut form, "नमस्ते");
    assert!(form.rubbed_out());
    let after = form.what_is_in(WhichField::Name).to_owned();

    // **The contrast is asserted rather than described**, because the first
    // version of this test asserted the wrong answer and passed nothing: it
    // expected `नमस्` — which is neither what a grapheme walk gives nor what a
    // `char` walk gives — so it would have failed against correct code and
    // against the bug alike. Both answers are spelled out here, so the test
    // says which rule is in force rather than only that something changed.
    let by_chars: String = {
        let mut whole = "नमस्ते".to_owned();
        whole.pop();
        whole
    };
    assert_eq!(
        after, "नम",
        "one grapheme went: नमस्ते is three -- न, म and स्ते -- so taking one back leaves नम"
    );
    assert_ne!(
        after, by_chars,
        "and a char walk would have left नमस्त, a letter with its vowel sign torn off, which \
         is the i18n law reaching somewhere that is not a translated string"
    );
}

#[test]
fn backspace_on_an_empty_field_changes_nothing_and_says_so() {
    let mut form = AProviderBeingAddded::empty();
    assert!(!form.rubbed_out());
    assert_eq!(form.what_is_in(WhichField::Name), "");
}

#[test]
fn a_control_character_is_not_typing() {
    // A key pasted out of a terminal can carry one, and a provider name
    // holding an escape is a name nothing matches and nobody can see is wrong.
    let mut form = AProviderBeingAddded::empty();
    assert!(!form.typed('\n'));
    assert!(!form.typed('\t'));
    assert!(!form.typed('\u{1b}'));
    assert_eq!(form.what_is_in(WhichField::Name), "");
    // The control: an ordinary character in the same field is taken.
    assert!(form.typed('O'));
    assert_eq!(form.what_is_in(WhichField::Name), "O");
}

#[test]
fn only_the_key_field_is_a_secret() {
    // A drawing that decided this by counting rows would show a key the day
    // somebody adds a field above it.
    assert!(WhichField::Key.is_a_secret());
    assert!(!WhichField::Name.is_a_secret());
    assert!(!WhichField::Address.is_a_secret());
    assert!(!WhichField::Model.is_a_secret());
}

#[test]
fn a_filled_in_form_becomes_a_provider_its_key_and_a_model() {
    let form = one_a_person_filled_in();
    assert!(form.every_field_is_filled());

    let saving = form
        .what_it_would_save()
        .expect("a form a person filled in");
    assert_eq!(saving.provider.name, "OpenAI");
    assert_eq!(saving.provider.endpoint, "https://api.openai.com");
    assert_eq!(saving.model, "gpt-4o-mini");
    assert_eq!(saving.filed_under.as_str(), "provider/OpenAI");
    assert_eq!(
        saving
            .provider
            .key
            .as_ref()
            .map(alo_models::SecretRef::as_str),
        Some("provider/OpenAI"),
        "the provider refers to where its key is, which is what a settings file may hold"
    );
}

#[test]
fn the_key_is_never_rendered_by_the_form_or_by_what_it_saves() {
    let form = one_a_person_filled_in();
    assert!(
        !format!("{form:?}").contains("sk-not-a-real-key"),
        "the form rendered the key"
    );
    let saving = form.what_it_would_save().unwrap();
    assert!(
        !format!("{saving:?}").contains("sk-not-a-real-key"),
        "what it would save rendered the key"
    );
}

#[test]
fn the_region_is_unknown_because_nobody_said() {
    // `Provider` carries a region *as whoever added it stated*, and a person
    // pasting a key has stated nothing. Inventing one would be this screen
    // claiming to know where somebody else's service runs.
    let saving = one_a_person_filled_in().what_it_would_save().unwrap();
    assert_eq!(saving.provider.region, alo_models::source::Region::Unknown);
}

#[test]
fn a_form_with_no_name_says_which_field_to_fix() {
    let mut form = one_a_person_filled_in();
    form.the_keyboard_is_in(WhichField::Name);
    while form.rubbed_out() {}

    let refused = form
        .what_it_would_save()
        .expect_err("a provider with no name");
    assert_eq!(
        refused,
        NotYetAProvider::TheProvider(ProviderError::Unnamed)
    );
    assert_eq!(refused.which_field(), WhichField::Name);
}

#[test]
fn an_address_that_is_not_one_says_so_against_the_address() {
    let mut form = one_a_person_filled_in();
    form.the_keyboard_is_in(WhichField::Address);
    while form.rubbed_out() {}
    types(&mut form, "api.openai.com");

    let refused = form.what_it_would_save().expect_err("not an address");
    assert_eq!(
        refused,
        NotYetAProvider::TheProvider(ProviderError::NotAnAddress)
    );
    assert_eq!(refused.which_field(), WhichField::Address);
}

#[test]
fn a_provider_reached_over_http_somewhere_else_is_refused() {
    // Not this form's rule -- `Provider::checked` decides it -- but a person
    // meets it here, so the field it lands on is this form's answer.
    let mut form = one_a_person_filled_in();
    form.the_keyboard_is_in(WhichField::Address);
    while form.rubbed_out() {}
    types(&mut form, "http://someone-elses-machine.example");

    let refused = form
        .what_it_would_save()
        .expect_err("http off this machine");
    assert_eq!(
        refused,
        NotYetAProvider::TheProvider(ProviderError::InsecureEndpoint)
    );
    assert_eq!(refused.which_field(), WhichField::Address);
}

#[test]
fn a_runtime_on_this_machine_over_http_is_allowed() {
    // The control for the test above: http is refused because it would put a
    // key on the wire, and a loopback address puts it on no wire at all.
    let mut form = one_a_person_filled_in();
    form.the_keyboard_is_in(WhichField::Address);
    while form.rubbed_out() {}
    types(&mut form, "http://127.0.0.1:11434");

    let saving = form
        .what_it_would_save()
        .expect("a runtime on this machine is reachable over http");
    assert_eq!(saving.provider.endpoint, "http://127.0.0.1:11434");
}

#[test]
fn an_empty_key_says_so_against_the_key() {
    let mut form = one_a_person_filled_in();
    form.the_keyboard_is_in(WhichField::Key);
    while form.rubbed_out() {}

    let refused = form.what_it_would_save().expect_err("no key");
    assert!(matches!(refused, NotYetAProvider::TheKey(_)));
    assert_eq!(refused.which_field(), WhichField::Key);
}

#[test]
fn no_model_is_refused_here_rather_than_by_the_provider_over_the_network() {
    // A provider asked for a model called nothing answers with its own error,
    // over the network, about something this machine knew before it asked.
    let mut form = one_a_person_filled_in();
    form.the_keyboard_is_in(WhichField::Model);
    while form.rubbed_out() {}

    let refused = form.what_it_would_save().expect_err("no model");
    assert_eq!(refused, NotYetAProvider::NoModel);
    assert_eq!(refused.which_field(), WhichField::Model);
}

#[test]
fn space_alone_is_not_a_filled_in_field() {
    // Somebody tabbing through a form leaves spaces behind, and a name of one
    // space is a provider nobody can pick out of a list.
    let mut form = AProviderBeingAddded::empty();
    types(&mut form, "   ");
    form.to_the_next_field();
    types(&mut form, "https://api.openai.com");
    form.to_the_next_field();
    types(&mut form, "sk-not-a-real-key");
    form.to_the_next_field();
    types(&mut form, "gpt-4o-mini");

    assert!(!form.every_field_is_filled());
    assert_eq!(
        form.what_it_would_save().expect_err("a name of spaces"),
        NotYetAProvider::TheProvider(ProviderError::Unnamed)
    );
}

#[test]
fn every_field_filled_is_not_the_same_question_as_every_field_right() {
    // A person can finish typing and still be wrong, and a surface needs to
    // know which of the two it is: one decides whether to offer saving, the
    // other decides what to say when they do.
    let mut form = AProviderBeingAddded::empty();
    types(&mut form, "OpenAI");
    form.to_the_next_field();
    types(&mut form, "not-an-address");
    form.to_the_next_field();
    types(&mut form, "sk-not-a-real-key");
    form.to_the_next_field();
    types(&mut form, "gpt-4o-mini");

    assert!(
        form.every_field_is_filled(),
        "they have finished typing in all four"
    );
    assert!(
        form.what_it_would_save().is_err(),
        "and it is still not a provider"
    );
}

#[test]
fn a_trailing_slash_on_the_address_is_not_a_different_provider() {
    let mut form = one_a_person_filled_in();
    form.the_keyboard_is_in(WhichField::Address);
    while form.rubbed_out() {}
    types(&mut form, "https://api.openai.com/");

    let saving = form.what_it_would_save().unwrap();
    assert_eq!(
        saving.provider.endpoint, "https://api.openai.com",
        "the slash a person's browser put on the end is not part of the address"
    );
}

#[test]
fn the_key_is_filed_under_the_name_as_it_will_be_saved() {
    // The reference and the provider's name are built from the same trimmed
    // name. If they could differ, the machine would file a key under one name
    // and look for it under another -- reported as Missing, sending a person
    // to add a key they already added.
    let mut form = AProviderBeingAddded::empty();
    types(&mut form, "  OpenAI  ");
    form.to_the_next_field();
    types(&mut form, "https://api.openai.com");
    form.to_the_next_field();
    types(&mut form, "sk-not-a-real-key");
    form.to_the_next_field();
    types(&mut form, "gpt-4o-mini");

    let saving = form.what_it_would_save().unwrap();
    assert_eq!(saving.provider.name, "OpenAI");
    assert_eq!(saving.filed_under.as_str(), "provider/OpenAI");
    assert_eq!(
        saving
            .provider
            .key
            .as_ref()
            .map(alo_models::SecretRef::as_str),
        Some("provider/OpenAI"),
        "what the provider refers to and where the key goes are one value"
    );
}

/// **The provider a person added offers the model they named.**
///
/// This is the row Settings draws. `alo_shell::settings_answering` lists, under
/// *my provider*, each model `provider.models` holds — and **one of the four
/// with nothing under it is absent rather than drawn and disabled**. So a
/// provider saved with an empty list is a provider a person can never choose:
/// the key is kept, the choice is written, and the screen offers no row to
/// select and no way to change their mind.
///
/// Measured on a booted machine on 2026-10-23: the save succeeded — no error
/// on the service log, the form closed, which it only does when it worked —
/// and the answering section still read *nothing is chosen yet to answer
/// questions*.
///
/// `Provider::checked` does not take models and leaves the list empty, and the
/// model travelled beside the provider in [`WhatToSave`] rather than in it.
/// Deleting the two lines that put it in makes this test fail.
#[test]
fn the_provider_offers_the_model_the_person_named() {
    let saving = one_a_person_filled_in()
        .what_it_would_save()
        .expect("a filled-in form has something to save");
    assert_eq!(
        saving.provider.models,
        vec!["gpt-4o-mini".to_owned()],
        "the provider was saved knowing no models, so Settings draws no row for it and the \
         person who just added it cannot choose it"
    );
}

/// **The model is the one typed, trimmed, and nothing else is invented.**
///
/// The control at the other end. The test above would pass on a provider that
/// claimed every model in the world; this holds the list to exactly what the
/// person said, because nothing here asks a provider what it offers — that is
/// a request to the wire, and `alo_asking::Vetting` is the only door to one.
#[test]
fn the_model_list_is_exactly_what_was_typed() {
    let mut form = AProviderBeingAddded::empty();
    types(&mut form, "OpenAI");
    form.to_the_next_field();
    types(&mut form, "https://api.openai.com");
    form.to_the_next_field();
    types(&mut form, "sk-not-a-real-key");
    form.to_the_next_field();
    types(&mut form, "  o3-mini  ");
    let saving = form
        .what_it_would_save()
        .expect("a filled-in form has something to save");
    assert_eq!(
        saving.provider.models.len(),
        1,
        "more than one model was invented"
    );
    assert_eq!(
        saving.provider.models,
        vec!["o3-mini".to_owned()],
        "the model was not trimmed the way the field it came from is"
    );
    assert_eq!(
        saving.model, "o3-mini",
        "what travels beside the provider and what is in it disagree"
    );
}

/// A form filled in with this address and nothing else changed.
fn at(address: &str) -> AProviderBeingAddded {
    let mut form = one_a_person_filled_in();
    form.the_keyboard_is_in(WhichField::Address);
    while form.rubbed_out() {}
    types(&mut form, address);
    form
}

#[test]
fn an_address_at_this_machine_is_known_to_be_one() {
    // The three ways a person writes this machine, each with the port a local
    // service listens on: the warning in Settings rests on this answer.
    for address in [
        "http://127.0.0.1:11434",
        "http://[::1]:8080/v1",
        "http://localhost:1234",
    ] {
        assert!(at(address).its_address_is_this_machine(), "{address}");
    }
}

#[test]
fn an_address_off_this_machine_is_not_one() {
    // The refusal half: a hosted provider, and a name made to look like this
    // machine, are both somewhere else, so neither carries the warning.
    for address in [
        "https://api.openai.com",
        "https://localhost.attacker.example",
        "",
    ] {
        assert!(!at(address).its_address_is_this_machine(), "{address}");
    }
}

#[test]
fn an_address_at_this_machine_can_still_be_saved() {
    // **The warning takes nothing away** (law 5): the form saves a service at
    // this machine's address exactly as it saves any other.
    let form = at("http://localhost:1234");
    assert!(form.its_address_is_this_machine());
    let saving = form
        .what_it_would_save()
        .expect("a service at this machine's address is the person's to add");
    assert_eq!(saving.provider.endpoint, "http://localhost:1234");
}
