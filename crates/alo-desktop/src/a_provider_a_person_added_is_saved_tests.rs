//! A provider saved against a real keyring and a real settings file, then read
//! back by the thing that would use it.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_keyring_fixture::AKeyringOfOurOwn;
use alo_models::SecretRef;
use alo_secrets::{NotStored, TheKeyring};
use alo_setting_up::a_provider_is_filled_in::AProviderBeingAddded;

use super::{NotSaved, a_provider_a_person_added_is_saved};

/// A settings file nothing else in this process is using.
fn somewhere_of_its_own() -> std::path::PathBuf {
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let mine = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let at = std::env::temp_dir().join(format!("alo-saving-{}-{mine}", std::process::id()));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    at.join("settings.toml")
}

/// Type a whole string into the field the keyboard is in.
fn types(form: &mut AProviderBeingAddded, what: &str) {
    for letter in what.chars() {
        form.typed(letter);
    }
}

/// The form as a person adding OpenAI would leave it.
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
fn a_provider_a_person_added_can_be_asked_afterwards() {
    // **The claim a person cares about, end to end**: they fill the form in,
    // press save, and the machine can both find the key and knows this is what
    // answers them. Read back through `what_answers_here`, which is the
    // function `alo-desktop`'s own `a_task_for` calls when somebody presses
    // Enter in the Bar — so this is the real question, not a paraphrase.
    let keyring = AKeyringOfOurOwn::started("saved");
    let ours = TheKeyring::opened(&keyring.bus()).unwrap();
    let at = somewhere_of_its_own();
    let saving = one_a_person_filled_in().what_it_would_save().unwrap();

    // Both controls at the start: nothing answers, and no key is filed.
    assert!(
        alo_choosing::what_answers_here(&at).is_none(),
        "the control: nothing answered before"
    );
    assert!(matches!(
        ours.look_up(&SecretRef::named("provider/OpenAI")),
        Err(NotStored::Missing)
    ));

    a_provider_a_person_added_is_saved(&ours, &at, &saving).expect("a filled-in provider saves");

    assert!(
        alo_choosing::what_answers_here(&at).is_some(),
        "after saving, the machine knows what answers a person's questions -- which is \
         exactly what a_task_for asks before it opens a window"
    );
    assert!(
        ours.look_up(&SecretRef::named("provider/OpenAI")).is_ok(),
        "and the key is where the provider says it is"
    );
}

#[test]
fn the_provider_is_on_the_persons_own_list_under_the_name_they_gave() {
    let keyring = AKeyringOfOurOwn::started("listed");
    let ours = TheKeyring::opened(&keyring.bus()).unwrap();
    let at = somewhere_of_its_own();
    let saving = one_a_person_filled_in().what_it_would_save().unwrap();

    a_provider_a_person_added_is_saved(&ours, &at, &saving).unwrap();

    let settings = alo_choosing::Settings::at(&at).expect("a settings file that was just written");
    let provider = settings.provider().expect("the provider that was added");
    assert_eq!(provider.name, "OpenAI");
    assert_eq!(provider.endpoint, "https://api.openai.com");
    assert_eq!(
        provider.key.as_ref().map(SecretRef::as_str),
        Some("provider/OpenAI"),
        "the file holds where the key is and never the key"
    );
}

#[test]
fn the_settings_file_never_holds_the_key_itself() {
    // The rule the whole design rests on: a settings file can be read, logged
    // or put in a support bundle, so what is in it is a reference.
    let keyring = AKeyringOfOurOwn::started("no-key-in-file");
    let ours = TheKeyring::opened(&keyring.bus()).unwrap();
    let at = somewhere_of_its_own();
    let saving = one_a_person_filled_in().what_it_would_save().unwrap();

    a_provider_a_person_added_is_saved(&ours, &at, &saving).unwrap();

    let written = std::fs::read_to_string(&at).expect("the settings file");
    assert!(
        !written.contains("sk-not-a-real-key"),
        "the key reached the settings file"
    );
    // The control: the file is not empty and does name the provider, so the
    // absence above is the key and not an unwritten file.
    assert!(written.contains("OpenAI"), "the file names the provider");
}

#[test]
fn a_keyring_that_will_not_take_the_key_leaves_the_settings_exactly_as_they_were() {
    // **The whole reason the key is written first.** If the settings went
    // first, this machine would afterwards look set up and fail at the moment
    // a person asked it something -- and the obvious thing for them to do
    // about it, adding the key, is the thing they just did.
    let keyring = AKeyringOfOurOwn::started("refused-save");
    let ours = TheKeyring::opened(&keyring.bus()).unwrap();
    let at = somewhere_of_its_own();
    let saving = one_a_person_filled_in().what_it_would_save().unwrap();

    // The control first: this keyring takes a key while it is reachable, so
    // the refusal below is the keyring going away and not something else
    // about this provider.
    ours.keep(
        &SecretRef::named("provider/Control"),
        &alo_models::Secret::typed("sk-control").unwrap(),
    )
    .expect("a reachable keyring takes a key");

    keyring.stop_letting_anyone_reach_the_keyring();

    let refused = a_provider_a_person_added_is_saved(&ours, &at, &saving)
        .expect_err("a keyring nothing can reach refuses");
    assert!(
        matches!(refused, NotSaved::KeyNotKept(_)),
        "it is the key that did not happen, and it says so: {refused:?}"
    );
    assert!(
        alo_choosing::what_answers_here(&at).is_none(),
        "nothing answers, because the settings were never touched"
    );
    assert!(
        !at.exists(),
        "and no settings file was created on the way to failing"
    );
}

#[test]
fn saving_a_second_provider_over_the_first_leaves_one_key_and_one_choice() {
    // A person who pastes a new key over an old one has one key afterwards,
    // and the machine answers with the newer provider.
    let keyring = AKeyringOfOurOwn::started("twice");
    let ours = TheKeyring::opened(&keyring.bus()).unwrap();
    let at = somewhere_of_its_own();

    let first = one_a_person_filled_in().what_it_would_save().unwrap();
    a_provider_a_person_added_is_saved(&ours, &at, &first).unwrap();

    let mut again = AProviderBeingAddded::empty();
    types(&mut again, "OpenAI");
    again.to_the_next_field();
    types(&mut again, "https://api.openai.com");
    again.to_the_next_field();
    types(&mut again, "sk-the-one-that-works");
    again.to_the_next_field();
    types(&mut again, "gpt-4o");
    let second = again.what_it_would_save().unwrap();

    a_provider_a_person_added_is_saved(&ours, &at, &second)
        .expect("a person correcting their key saves again");

    let settings = alo_choosing::Settings::at(&at).unwrap();
    assert_eq!(
        settings.chosen().expect("something answers").model(),
        "gpt-4o",
        "the machine asks for what they chose the second time"
    );
    let written = std::fs::read_to_string(&at).unwrap();
    assert!(!written.contains("sk-the-one-that-works"));
    assert!(!written.contains("sk-not-a-real-key"));
}

#[test]
fn a_provider_on_this_machine_over_http_saves_too() {
    // The control for the address rule: a local runtime is reachable over
    // http because nothing leaves the machine, and saving must not have a
    // second opinion about that.
    let keyring = AKeyringOfOurOwn::started("local");
    let ours = TheKeyring::opened(&keyring.bus()).unwrap();
    let at = somewhere_of_its_own();

    let mut form = AProviderBeingAddded::empty();
    types(&mut form, "This machine");
    form.to_the_next_field();
    types(&mut form, "http://127.0.0.1:11434");
    form.to_the_next_field();
    types(&mut form, "not-really-needed");
    form.to_the_next_field();
    types(&mut form, "llama3");
    let saving = form.what_it_would_save().unwrap();

    a_provider_a_person_added_is_saved(&ours, &at, &saving).unwrap();
    assert!(alo_choosing::what_answers_here(&at).is_some());
}
