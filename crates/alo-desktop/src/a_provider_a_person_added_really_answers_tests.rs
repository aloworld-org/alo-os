//! **A provider a person added is one a real provider answers**, over the
//! network, with the key read back out of the keyring this road put it in.
//!
//! Every other test of this road stops at the settings file. This one carries
//! on: it fills the form in the way a person does, saves it through
//! `a_provider_a_person_added_is_saved`, reads the key back out of the keyring
//! by the reference the provider names, and asks the real service whether that
//! key works.
//!
//! # Why the whole road rather than the last step
//!
//! The faults this is for are joins, and a test of any one piece cannot see
//! them. A key filed under a name the lookup does not search for passes every
//! test of the writing and every test of the reading, and fails the person. So
//! nothing here is handed between steps by the test: the reference comes from
//! the provider the save road wrote, and the key comes from the keyring by
//! that reference.
//!
//! # It costs money and is skipped without a key
//!
//! `ALO_PROVIDER_KEY` is read from the environment, as
//! `alo-agentd/tests/the_agent_asks_a_real_provider.rs` does and for its
//! reason: the gate runs with no key and skips, and **a skip says so rather
//! than passing quietly.**

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_asking::Vetting;
use alo_capability::Grantee;
use alo_egress::Indicator;
use alo_keyring_fixture::AKeyringOfOurOwn;
use alo_models::{SecretRef, SourcePolicy};
use alo_secrets::TheKeyring;
use alo_setting_up::a_provider_is_filled_in::AProviderBeingAddded;

use super::a_provider_a_person_added_is_saved;

/// Type a whole string into the field the keyboard is in.
fn types(form: &mut AProviderBeingAddded, what: &str) {
    for letter in what.chars() {
        form.typed(letter);
    }
}

#[test]
fn a_provider_a_person_added_is_one_that_really_answers() {
    let Ok(key) = std::env::var("ALO_PROVIDER_KEY") else {
        println!(
            "SKIPPED, and this is not a pass: ALO_PROVIDER_KEY is not set, so no provider was \
             asked anything. This test reaches a real service over the network and costs money, \
             so the gate runs it with no key and it skips."
        );
        return;
    };
    let endpoint =
        std::env::var("ALO_PROVIDER").unwrap_or_else(|_| "https://api.openai.com".to_owned());
    let model = std::env::var("ALO_PROVIDER_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_owned());

    // **Filled in the way a person fills it in**, one character at a time
    // through the form's own road, rather than built as a value. A form that
    // dropped a character would otherwise be invisible to this test.
    let mut form = AProviderBeingAddded::empty();
    types(&mut form, "OpenAI");
    form.to_the_next_field();
    types(&mut form, &endpoint);
    form.to_the_next_field();
    types(&mut form, &key);
    form.to_the_next_field();
    types(&mut form, &model);

    assert!(
        form.every_field_is_filled(),
        "the form a person filled in is finished"
    );
    let saving = form
        .what_it_would_save()
        .expect("a filled-in form is a provider");

    // A real Secret Service, on a bus nobody else can reach.
    let keyring = AKeyringOfOurOwn::started("really-answers");
    let ours = TheKeyring::opened(&keyring.bus()).expect("the keyring opens on its own bus");
    let at = std::env::temp_dir()
        .join(format!("alo-really-answers-{}", std::process::id()))
        .join("settings.toml");
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    let _ = std::fs::remove_file(&at);

    a_provider_a_person_added_is_saved(&ours, &at, &saving).expect("the provider saves");

    // **The machine now knows what answers**, which is the question
    // `a_task_for` asks before it opens a window for a person's words.
    assert!(
        alo_choosing::what_answers_here(&at).is_some(),
        "after saving, this machine can answer a question"
    );

    // **The reference comes from what was written, not from this test.** A key
    // filed under a name the lookup does not search for passes every test of
    // the writing and every test of the reading, and fails the person.
    let settings = alo_choosing::Settings::at(&at).expect("the settings that were just written");
    let provider = settings.provider().expect("the provider that was saved");
    let reference: SecretRef = provider
        .key
        .clone()
        .expect("the provider names where its key is");
    println!(
        "the provider was saved, and names its key as {}",
        reference.as_str()
    );

    let found = ours
        .look_up(&reference)
        .expect("the key is in the keyring, under the name the provider gives");
    println!("the key was found in the keyring by that name");

    // And now the one that costs money: a real service, asked whether this
    // key works — **through `alo_asking::Vetting`, which is the door that puts
    // the request on the indicator.**
    //
    // This reached `alo_models::Trying` directly until 2026-10-22, and
    // `alo-asking`'s own guard caught it: *nothing that ships reaches the wire
    // except through the door that shows it*, which reads every shipped source
    // and permits three files. A `_tests.rs` under `src/` is shipped source to
    // that guard, and rightly — the request it made was real, to
    // api.openai.com, **with nothing on the egress indicator.** Law 1 is that
    // every network egress is visible at the moment it happens, and the test
    // closest to what a person does in Settings was the one going round it.
    //
    // So the indicator is asked before and after rather than passed in and
    // ignored. Quiet first, because a test that cannot tell its own request
    // from somebody else's proves nothing about either.
    let mut indicator = Indicator::default();
    assert!(
        indicator.is_quiet(),
        "the indicator was already showing something before this test sent anything"
    );

    let vetted = Vetting::provider(provider, Some(&found))
        .under(
            &Grantee::named("@settings"),
            &SourcePolicy::Anywhere,
            &mut indicator,
            std::time::SystemTime::now(),
        )
        .expect(
            "a provider a person added answers -- the key, the endpoint and the settings file \
             were every one of them written by the road under test, so a refusal here is that \
             road rather than the service",
        );
    println!("the indicator showed the request while it was in flight");

    let answer = vetted.ended(&mut indicator);
    println!("THE PROVIDER ANSWERED: {answer:?}");
    assert!(
        answer.is_a_working_provider(),
        "the provider this road saved did not answer: {answer:?}"
    );
}
