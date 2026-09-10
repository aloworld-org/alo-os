//! The plan's acceptance for *what the overlay shows when the agent has
//! nothing to say yet*, one test per criterion.
//!
//! `docs/autonomy/v0-01-delivery-plan.md`, task 3: *the overlay's state is a
//! value derived from the daemon's own answers, with a case for each of
//! nothing granted, nothing chosen and ready; every string externalised; and
//! the nothing chosen case says what to do rather than being empty.*
//!
//! These tests reach the machine the way the daemon does, and not through a
//! seam of the test's own: the person's settings are **written to a file and
//! read back with `alo_choosing::Settings::at`**, the grants are an
//! `alo_capability::Grants` asked what is active, and what is leaving comes
//! off an `alo_egress::Indicator` that really permitted something. The strings
//! come from `alo_saying::everything_this_machine_can_say` — the vocabulary a
//! shell really holds — so a string this crate declares and nobody collects
//! fails here rather than on somebody's screen.
//!
//! Nothing here draws anything, and nothing asserts a pixel.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{Ask, Grant, Grantee, Grants, Reach};
use alo_choosing::Settings;
use alo_egress::{Destination, EgressPolicy, Indicator, Leaving, Why};
use alo_overlay::words::{EVERY_COUNTED, EVERY_WORD};
use alo_overlay::{AtRest, Granted, Quiet, Standing, WouldAnswer};
use alo_saying::everything_this_machine_can_say;
use alo_strings::{Said, Strings};

/// The moment every test here is written against.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent this machine has, as its description names it.
fn the_agent() -> Grantee {
    Grantee::named("@files")
}

/// The strings a shell really holds: everything the machine can say.
fn what_this_machine_says() -> Strings {
    Strings::of(everything_this_machine_can_say().unwrap())
}

/// A settings file of this test's own, written to disk and read back through
/// the door `alo-agentd` reads it through.
fn settings_saying(what: &str, said: &str) -> Settings {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder: PathBuf = std::env::temp_dir().join(format!(
        "alo-overlay-at-rest-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    drop(fs::remove_dir_all(&folder));
    fs::create_dir_all(&folder).unwrap();
    let file = folder.join(alo_choosing::THE_SETTINGS);
    fs::write(&file, said).unwrap();
    Settings::at(&file).unwrap()
}

/// A person who chose a model that runs on their own machine.
fn chose_a_local_model() -> Settings {
    settings_saying(
        "local",
        "format = 1\n\n[answers]\ncatalogue = \"mistral-small\"\n",
    )
}

/// A person who chose a provider they added themselves.
fn chose_a_provider() -> Settings {
    settings_saying(
        "provider",
        "format = 2\n\n[answers]\nprovider = { name = \"Mistral\", model = \"mistral-small-latest\" }\n\n[[provider]]\nname = \"Mistral\"\nendpoint = \"https://api.mistral.ai\"\nregion = \"the EU\"\n",
    )
}

/// A machine where the agent has been granted one folder, until an hour after
/// noon.
fn one_folder_granted() -> Grants {
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked(
            "@files",
            Reach::Folder(PathBuf::from("/home/anna/Invoices")),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    grants
}

/// **The overlay's state is a value derived from the daemon's own answers,
/// with a case for each of *nothing granted*, *nothing chosen* and *ready*.**
///
/// Each of the three is reached from a machine that really is in that state —
/// a settings file on disk, a grant list asked what is active — rather than
/// from a state somebody named. There is no constructor that takes a
/// [`Standing`], which is what makes this a derivation and not a claim.
#[test]
fn the_overlays_state_is_derived_with_a_case_for_each_of_the_three() {
    let nothing_chosen = AtRest::of_this_machine(
        &the_agent(),
        &Grants::default(),
        &settings_saying("untouched-machine", "format = 1\n"),
        &Indicator::default(),
        noon(),
    );
    assert_eq!(nothing_chosen.standing(), Standing::NothingChosen);

    let nothing_granted = AtRest::of_this_machine(
        &the_agent(),
        &Grants::default(),
        &chose_a_local_model(),
        &Indicator::default(),
        noon(),
    );
    assert_eq!(nothing_granted.standing(), Standing::NothingGranted);
    assert_eq!(
        nothing_granted.answering(),
        &WouldAnswer::Something {
            source: alo_models::InferenceSource::ThisMachine,
            model: "mistral-small".to_owned(),
        }
    );

    let ready = AtRest::of_this_machine(
        &the_agent(),
        &one_folder_granted(),
        &chose_a_local_model(),
        &Indicator::default(),
        noon(),
    );
    assert_eq!(ready.standing(), Standing::Ready);
    assert_eq!(ready.granted(), Granted::of_how_many(1));
    assert_eq!(ready.quiet(), Quiet::NothingIsLeaving);

    // Three states, three sentences, and no two of them the same.
    let strings = what_this_machine_says();
    let sentences: Vec<String> = [&nothing_chosen, &nothing_granted, &ready]
        .into_iter()
        .map(|at_rest| at_rest.said(&strings).into_text())
        .collect();
    assert_eq!(sentences.len(), 3);
    for sentence in &sentences {
        assert_eq!(
            sentences.iter().filter(|other| *other == sentence).count(),
            1,
            "two of the three states read the same: {sentence}"
        );
    }
}

/// **Every string externalised.** Every word this crate declares is in the
/// vocabulary the machine really assembles, every line of every state renders
/// from it without being a bug or a blank, and a shell that declared nothing
/// is told so rather than shown English by accident.
#[test]
fn every_string_the_overlay_shows_is_in_the_machines_own_vocabulary() {
    let vocabulary = everything_this_machine_can_say().unwrap();
    for word in EVERY_WORD {
        assert!(
            vocabulary.phrase(&word.key()).is_some(),
            "the machine cannot say {}",
            word.named()
        );
    }
    for counted in EVERY_COUNTED {
        assert!(
            vocabulary.plural(&counted.key()).is_some(),
            "the machine cannot count {}",
            counted.named()
        );
    }

    let strings = what_this_machine_says();
    for at_rest in every_state() {
        for line in at_rest.lines(&strings) {
            assert!(!line.is_a_bug(), "{line}");
            assert!(!line.text().trim().is_empty(), "a line came out blank");
        }
    }

    // And the refusal path of externalisation itself: nothing here falls back
    // to English written in the source. A vocabulary that never received this
    // crate's list answers with the key, marked as this repository's bug.
    let nothing = Strings::of(alo_strings::Vocabulary::empty());
    for at_rest in every_state() {
        for line in at_rest.lines(&nothing) {
            assert!(
                line.is_a_bug(),
                "a line was shown from somewhere that is not the vocabulary: {line}"
            );
        }
    }
}

/// **The *nothing chosen* case says what to do rather than being empty.**
///
/// Not merely non-empty: it names the two things a person can actually do on
/// a machine where nothing answers questions, and the three readings beneath
/// it are sentences of their own rather than blanks waiting for a value.
#[test]
fn the_nothing_chosen_case_says_what_to_do_rather_than_being_empty() {
    let at_rest = AtRest::of_this_machine(
        &the_agent(),
        &Grants::default(),
        &settings_saying("nobody-has-chosen", "format = 1\n"),
        &Indicator::default(),
        noon(),
    );
    assert_eq!(at_rest.standing(), Standing::NothingChosen);

    let strings = what_this_machine_says();
    let state = at_rest.said(&strings);
    assert!(!state.is_a_bug(), "{state}");
    assert!(state.text().contains("Choose a model"), "{state}");
    assert!(state.text().contains("add a provider"), "{state}");

    // Every line under it is a sentence too — an empty state is the one place
    // a blank line reads as *this machine has nothing to tell you*.
    let lines = at_rest.lines(&strings).map(Said::into_text);
    assert_eq!(lines[1], "no model or provider has been chosen");
    assert_eq!(lines[2], "nothing is granted");
    assert_eq!(lines[3], "nothing is leaving this machine");
}

/// **A provider the person chose is never shown as answering on this
/// machine.** The refusal path that matters most on this surface: a line
/// saying *on this machine* above a question about to be sent to a company
/// would be the reassuring reading and the false one.
#[test]
fn a_provider_choice_is_never_shown_as_answering_on_this_machine() {
    let at_rest = AtRest::of_this_machine(
        &the_agent(),
        &one_folder_granted(),
        &chose_a_provider(),
        &Indicator::default(),
        noon(),
    );
    assert_eq!(at_rest.standing(), Standing::Ready);

    let line = at_rest.lines(&what_this_machine_says())[1]
        .text()
        .to_owned();
    assert_eq!(line, "mistral-small-latest answers, by Mistral, in the EU");
    assert!(
        !line.contains("on this machine"),
        "a question addressed to a provider was shown as answered here: {line}"
    );
}

/// **A grant that has expired is not shown as something the agent can
/// reach**, and the machine falls back to *nothing granted* rather than
/// claiming to be ready — which is what the daemon would do with the very
/// next verb.
#[test]
fn an_expired_grant_is_not_shown_as_something_the_agent_can_reach() {
    let grants = one_folder_granted();
    let after = noon() + hour();
    assert!(
        !grants.permits(
            &the_agent(),
            &Ask::path("/home/anna/Invoices/march.pdf"),
            after
        ),
        "the grant is still active, so this test proves nothing"
    );

    let at_rest = AtRest::of_this_machine(
        &the_agent(),
        &grants,
        &chose_a_local_model(),
        &Indicator::default(),
        after,
    );
    assert_eq!(at_rest.granted(), Granted::Nothing);
    assert_eq!(at_rest.standing(), Standing::NothingGranted);
    assert_eq!(
        at_rest.lines(&what_this_machine_says())[2].text(),
        "nothing is granted"
    );
}

/// **An egress the policy refused is never shown as leaving.** Nothing left,
/// so there is nothing to show — an overlay that reported the connections a
/// rule stopped would teach people to ignore the one line on it that matters.
#[test]
fn an_egress_the_policy_refused_is_never_shown_as_leaving() {
    let mut indicator = Indicator::default();
    let refused = indicator.beginning(
        &EgressPolicy::NothingLeaves,
        Leaving::because(
            &the_agent(),
            Why::Fetching,
            Destination::at("alo.example").unwrap(),
        ),
        noon(),
    );
    assert!(
        refused.is_err(),
        "the policy permitted it, so this proves nothing"
    );

    let at_rest = AtRest::of_this_machine(
        &the_agent(),
        &one_folder_granted(),
        &chose_a_local_model(),
        &indicator,
        noon(),
    );
    assert_eq!(at_rest.quiet(), Quiet::NothingIsLeaving);
    assert_eq!(
        at_rest.lines(&what_this_machine_says())[3].text(),
        "nothing is leaving this machine"
    );
}

/// Every state a machine can be in at rest, built the way the tests above
/// build them, so no test that walks all three quietly skips one.
fn every_state() -> Vec<AtRest> {
    let mut leaving = Indicator::default();
    drop(leaving.beginning(
        &EgressPolicy::Anywhere,
        Leaving::because(
            &the_agent(),
            Why::Fetching,
            Destination::at("alo.example").unwrap(),
        ),
        noon(),
    ));
    vec![
        AtRest::of_this_machine(
            &the_agent(),
            &Grants::default(),
            &settings_saying("every-state-untouched", "format = 1\n"),
            &Indicator::default(),
            noon(),
        ),
        AtRest::of_this_machine(
            &the_agent(),
            &Grants::default(),
            &chose_a_local_model(),
            &Indicator::default(),
            noon(),
        ),
        AtRest::of_this_machine(
            &the_agent(),
            &one_folder_granted(),
            &chose_a_provider(),
            &leaving,
            noon(),
        ),
    ]
}
