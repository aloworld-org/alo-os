//! The form read back as the words a person would see on the screen.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use crate::a_provider_is_added_in_settings::WhyNotSaved;
use alo_setting_up::a_provider_is_filled_in::{AProviderBeingAddded, WhichField};
use alo_strings::{Strings, Vocabulary};

use super::the_form;

/// A vocabulary holding this form's own sentences and nothing else.
fn strings() -> Strings {
    let mut vocabulary = Vocabulary::default();
    crate::the_providers_words::declare_the_providers_words(&mut vocabulary)
        .expect("this form's own words declare");
    Strings::of(vocabulary)
}

/// Type a whole string into the field the keyboard is in.
fn types(form: &mut AProviderBeingAddded, what: &str) {
    for letter in what.chars() {
        form.typed(letter);
    }
}

/// What is drawn to the right of the label on this field's line.
fn what_is_shown(form: &AProviderBeingAddded, field: WhichField) -> String {
    let drawn = the_form(form, None, &strings());
    let at = WhichField::EVERY
        .iter()
        .position(|each| *each == field)
        .expect("one of the four");
    drawn
        .lines
        .get(at)
        .expect("a line per field")
        .parts
        .get(1)
        .expect("a label and what is in it")
        .clone()
}

#[test]
fn there_is_a_line_for_each_of_the_four_fields() {
    let drawn = the_form(&AProviderBeingAddded::empty(), None, &strings());
    assert_eq!(drawn.lines.len(), 4);
    // And a heading and a line saying what to do, above them.
    assert_eq!(drawn.said.len(), 2);
}

#[test]
fn every_label_is_a_sentence_and_not_a_literal() {
    // The i18n law: a label written into the drawing is a label nobody can
    // translate. Each of these comes back from the vocabulary.
    let drawn = the_form(&AProviderBeingAddded::empty(), None, &strings());
    let labels: Vec<&str> = drawn
        .lines
        .iter()
        .map(|line| line.parts.first().expect("a label").as_str())
        .collect();
    assert_eq!(labels, vec!["Name", "Address", "Key", "Model"]);
}

#[test]
fn what_a_person_typed_is_drawn_back_to_them() {
    let mut form = AProviderBeingAddded::empty();
    types(&mut form, "OpenAI");
    form.to_the_next_field();
    types(&mut form, "https://api.openai.com");

    assert_eq!(what_is_shown(&form, WhichField::Name), "OpenAI");
    assert_eq!(
        what_is_shown(&form, WhichField::Address),
        "https://api.openai.com▏",
        "and the field with the keyboard carries the caret"
    );
}

#[test]
fn the_key_is_never_drawn_in_clear() {
    // **The assertion this file exists for.** A key on a screen is a key on
    // anybody's shoulder, on a video call, and in the demo recording.
    let mut form = AProviderBeingAddded::empty();
    form.the_keyboard_is_in(WhichField::Key);
    types(&mut form, "sk-proj-a-real-looking-key");

    let shown = what_is_shown(&form, WhichField::Key);
    assert!(
        !shown.contains("sk-proj"),
        "the key reached the screen: {shown}"
    );
    assert!(
        shown.starts_with("••••"),
        "and what is drawn instead is marks: {shown}"
    );
}

#[test]
fn one_mark_is_drawn_for_each_letter_a_reader_would_count() {
    // Per grapheme and not per byte: a key with a letter outside ASCII in it
    // would otherwise draw more marks than it has letters, which tells a
    // person their key is longer than it is.
    let mut form = AProviderBeingAddded::empty();
    form.the_keyboard_is_in(WhichField::Key);
    types(&mut form, "abcd");
    assert_eq!(what_is_shown(&form, WhichField::Key), "••••▏");

    let mut wider = AProviderBeingAddded::empty();
    wider.the_keyboard_is_in(WhichField::Key);
    types(&mut wider, "नमस्ते");
    assert_eq!(
        what_is_shown(&wider, WhichField::Key),
        "•••▏",
        "three graphemes, not the six chars or the eighteen bytes"
    );
}

#[test]
fn the_other_three_fields_are_not_masked() {
    // The control at the other end: masking is asked of the field, so a
    // surface that masked everything would pass the test above and be useless.
    let mut form = AProviderBeingAddded::empty();
    types(&mut form, "OpenAI");
    assert_eq!(what_is_shown(&form, WhichField::Name), "OpenAI▏");
    assert!(!what_is_shown(&form, WhichField::Name).contains('•'));
}

#[test]
fn the_caret_is_only_in_the_field_with_the_keyboard() {
    let mut form = AProviderBeingAddded::empty();
    types(&mut form, "OpenAI");
    assert!(what_is_shown(&form, WhichField::Name).ends_with('▏'));
    assert_eq!(
        what_is_shown(&form, WhichField::Address),
        "",
        "a field nobody is in shows nothing, not an empty caret"
    );
}

#[test]
fn an_empty_field_with_the_keyboard_still_shows_where_they_are() {
    // So a person can see which field they are in before typing anything.
    let form = AProviderBeingAddded::empty();
    assert_eq!(what_is_shown(&form, WhichField::Name), "▏");
}

#[test]
fn the_field_with_the_keyboard_is_the_one_marked() {
    // **The caret is the mark**, and it is in that field and no other: the
    // redesign draws a form's fields as plain rows, which have no filled
    // square to mark one with.
    let mut form = AProviderBeingAddded::empty();
    form.to_the_next_field();
    let marked: Vec<bool> = WhichField::EVERY
        .into_iter()
        .map(|field| what_is_shown(&form, field).ends_with('▏'))
        .collect();
    assert_eq!(marked, vec![false, true, false, false]);
}

/// **A save that did not happen says so, above the form.**
///
/// On 2026-10-23 a person on a booted machine filled this form in, pressed
/// Enter, and the screen did not change: the save ran, the keyring refused,
/// and the reason went to the service log. This is that silence as a test.
#[test]
fn a_provider_that_was_not_saved_says_which_of_the_four_happened() {
    let form = AProviderBeingAddded::empty();
    let quiet = the_form(&form, None, &strings());

    for why in [
        WhyNotSaved::ThereIsNoKeyring,
        WhyNotSaved::TheKeyWasNotKept,
        WhyNotSaved::TheSettingsWouldNotRead,
        WhyNotSaved::ThereIsNowhereToKeep,
    ] {
        let told = the_form(&form, Some(why), &strings());
        assert_eq!(
            told.said.len(),
            quiet.said.len() + 1,
            "{why:?} drew no sentence, which is the silence this test is for"
        );
        let sentence = told.said.last().expect("a sentence was added");
        assert!(
            !sentence.contains('«'),
            "{why:?} reached a person as a key: {sentence}"
        );
        assert!(
            sentence.contains("still here"),
            "{why:?} does not tell them their typing survived: {sentence}"
        );
    }
}

/// **The four say four different things**, so a person knows which machine
/// they have.
///
/// Without this the type could answer one sentence for every case and every
/// assertion above would still pass — which is the shape of *something went
/// wrong*, the sentence `WhyNotSaved` exists to avoid.
#[test]
fn the_four_refusals_are_four_different_sentences() {
    let form = AProviderBeingAddded::empty();
    let mut seen = std::collections::BTreeSet::new();
    for why in [
        WhyNotSaved::ThereIsNoKeyring,
        WhyNotSaved::TheKeyWasNotKept,
        WhyNotSaved::TheSettingsWouldNotRead,
        WhyNotSaved::ThereIsNowhereToKeep,
    ] {
        let told = the_form(&form, Some(why), &strings());
        seen.insert(told.said.last().expect("a sentence").clone());
    }
    assert_eq!(seen.len(), 4, "the four refusals are not four sentences");
}

/// A form whose address field holds this, with the keyboard in it.
fn at(address: &str) -> AProviderBeingAddded {
    let mut form = AProviderBeingAddded::empty();
    types(&mut form, "Mine");
    form.to_the_next_field();
    types(&mut form, address);
    form
}

/// Whether the caution about this machine's address is among what is said.
fn is_cautioned(form: &AProviderBeingAddded) -> bool {
    let caution = strings()
        .say(
            &crate::the_providers_words::THIS_MACHINE_CANNOT_SEE_WHERE_IT_GOES.key(),
            &alo_strings::Filling::nothing(),
        )
        .into_text();
    the_form(form, None, &strings()).said.contains(&caution)
}

#[test]
fn a_service_at_this_machines_address_is_cautioned_before_saving() {
    // Owner's decision, 2026-10-29: alo cannot see what a local service does
    // with a question, so the person is told before they save.
    for address in [
        "http://127.0.0.1:11434",
        "http://[::1]:8080",
        "http://localhost:1234/v1",
    ] {
        assert!(is_cautioned(&at(address)), "{address}");
    }
}

#[test]
fn a_provider_off_this_machine_is_not_cautioned() {
    // The control: a hosted provider's questions are on the indicator, so a
    // sentence saying they would not be is false for it.
    assert!(!is_cautioned(&at("https://api.openai.com")));
    assert!(!is_cautioned(&AProviderBeingAddded::empty()));
    assert_eq!(
        the_form(&at("https://api.openai.com"), None, &strings())
            .said
            .len(),
        2,
        "a heading and what to do, and nothing else"
    );
}

#[test]
fn the_caution_is_a_sentence_and_not_its_key() {
    let said = the_form(&at("http://localhost:1234"), None, &strings()).said;
    assert!(
        said.iter().any(|line| line.starts_with("alo cannot see")),
        "{said:?}"
    );
}

#[test]
fn a_service_at_this_machines_own_network_address_is_cautioned_and_another_is_not() {
    // Owner's decision, 2026-10-30: the caution covers the addresses the
    // machine holds on its networks, as the desktop read them.
    let on_this_machine = |address: &str| {
        let mut form = at(address);
        form.this_machine_is_also_at(vec![std::net::IpAddr::from([192, 168, 1, 20])]);
        form
    };
    assert!(is_cautioned(&on_this_machine("http://192.168.1.20:8000")));
    assert!(!is_cautioned(&on_this_machine("http://192.168.1.21:8000")));
    assert!(
        !is_cautioned(&at("http://192.168.1.20:8000")),
        "a form told nothing about this machine's network takes it for somewhere else"
    );
}

/// Whether the caution about plain http across the person's network is among
/// what is said.
fn is_cautioned_as_unencrypted(form: &AProviderBeingAddded) -> bool {
    let caution = strings()
        .say(
            &crate::the_providers_words::OTHERS_ON_THIS_NETWORK_COULD_READ_IT.key(),
            &alo_strings::Filling::nothing(),
        )
        .into_text();
    the_form(form, None, &strings()).said.contains(&caution)
}

#[test]
fn plain_http_to_another_computer_on_the_persons_network_is_cautioned_in_plain_words() {
    // Owner's decision, 2026-10-30: the person is told, before Enter, that
    // the question and the key cross their network unencrypted and who could
    // read them.
    let form = at("http://192.168.1.21:11434");
    assert!(is_cautioned_as_unencrypted(&form));
    assert!(
        !is_cautioned(&form),
        "another computer is not this machine, and the sentence about this machine is false for it"
    );
    let said = the_form(&form, None, &strings()).said;
    let caution = said
        .iter()
        .find(|line| line.contains("unencrypted"))
        .expect("the caution is a sentence and not its key");
    assert!(caution.contains("your questions and your key"), "{caution}");
    assert!(caution.contains("Anyone else on that network"), "{caution}");
}

#[test]
fn only_plain_http_to_the_persons_network_is_cautioned_as_unencrypted() {
    // The refusal half: https, loopback, the internet (refused at Enter
    // instead) and an empty form say nothing about it.
    for form in [
        at("https://192.168.1.21:8443"),
        at("http://127.0.0.1:11434"),
        at("http://198.51.100.7:8000"),
        at("https://api.openai.com"),
        AProviderBeingAddded::empty(),
    ] {
        assert!(!is_cautioned_as_unencrypted(&form), "{form:?}");
    }
}

#[test]
fn this_machines_own_network_address_over_http_gets_both_cautions() {
    // By the time a question is asked, an address this machine held when the
    // provider was saved may be another computer's.
    let mut form = at("http://192.168.1.20:8000");
    form.this_machine_is_also_at(vec![std::net::IpAddr::from([192, 168, 1, 20])]);
    assert!(is_cautioned(&form));
    assert!(is_cautioned_as_unencrypted(&form));
}
