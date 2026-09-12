//! A provider that will not say where it runs is unknown, and unknown never
//! satisfies a policy — through the person's own settings, on a real disk.
//!
//! `docs/features.md`, v0.5: *a provider that will not say where it runs is
//! reported as **unknown**, never assumed to be nearby — and unknown never
//! satisfies a policy naming a region.* This file is that sentence as tests,
//! from the file a person's choice is written in to the permission a question
//! is put under, with the refusal beside the acceptance.
//!
//! The rule is `alo_models::SourcePolicy`'s and the value is
//! `alo_models::Region::Unknown`. What this file measures is the road between
//! them that runs through this crate: a provider written with no region is
//! read back as unknown rather than as any region, the bound an organisation
//! sets refuses it with a sentence saying the provider did not say — never that
//! it runs elsewhere — and a machine with no bound is not affected.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_choosing::{Choosing, Picked, Settings, where_it_is};
use alo_models::{NotAllowed, Provider, Region, SourcePolicy};
use alo_strings::Strings;

/// A home directory of this test's own, with nothing in it.
fn a_login_of_our_own(what: &str) -> PathBuf {
    let home = std::env::temp_dir().join(format!("alo-choosing-unknown-region-{what}"));
    if home.exists() {
        std::fs::remove_dir_all(&home).unwrap();
    }
    std::fs::create_dir_all(&home).unwrap();
    home
}

/// Where this session's settings are, worked out the way the daemon works it
/// out.
fn the_settings_of(home: &Path) -> PathBuf {
    where_it_is(None, Some(home.as_os_str())).expect("a login with a home directory has settings")
}

/// Everything the machine can say, which is what a process really holds.
fn everything_this_machine_can_say() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// A provider added the way a settings surface adds one when the person left
/// the region blank: **nothing** is stated, and nothing is inferred from the
/// address — `api.example.eu` is not evidence of anything.
fn a_provider_that_has_not_said() -> Provider {
    Provider::checked("Somewhere", "https://api.example.eu", Region::Unknown, None).unwrap()
}

/// Settings on a real disk choosing a provider that has not said where it
/// runs, and the settings read back off that disk.
fn a_machine_choosing_a_silent_provider(what: &str) -> (PathBuf, Settings) {
    let home = a_login_of_our_own(what);
    let at = the_settings_of(&home);
    let mut choosing = Choosing::at(&at).unwrap();
    choosing.adding(a_provider_that_has_not_said()).unwrap();
    choosing
        .answered_by(Some(
            Picked::from_a_provider("Somewhere", "a-model").unwrap(),
        ))
        .unwrap();
    let read = Settings::at(&at).unwrap();
    (at, read)
}

/// **A provider saved with no region is unknown, a value of its own** — not a
/// default region, not a region inferred from a `.eu` address, and not written
/// into the file as a region called anything.
#[test]
fn a_provider_saved_with_no_region_is_unknown_and_never_a_default_region() {
    let (at, read) = a_machine_choosing_a_silent_provider("saved");

    assert_eq!(read.provider().unwrap().region, Region::Unknown);
    assert_ne!(
        read.provider().unwrap().region,
        Region::Declared("unknown".to_owned())
    );
    // The file says nothing about a region at all, rather than a word standing
    // in for one that the reader would have to know is not a place.
    let written = std::fs::read_to_string(&at).unwrap();
    assert!(!written.contains("region"), "{written}");
    assert!(!written.contains("unknown"), "{written}");
    // And what was read is the same thing a person would have typed by hand.
    assert_eq!(
        read.chosen().unwrap().source(read.providers()),
        Some(alo_models::InferenceSource::Hosted {
            provider: "Somewhere".to_owned(),
            region: Region::Unknown,
        })
    );
}

/// **A bound naming a region refuses the provider, and the sentence says the
/// provider did not say where it runs** — not that it runs elsewhere, which
/// nobody knows — in the vocabulary the whole machine loads. Beside it, the
/// acceptance: a provider that said it runs in that region is permitted.
#[test]
fn a_bound_naming_a_region_refuses_it_as_unknown_and_says_the_provider_did_not_say() {
    let (_, read) = a_machine_choosing_a_silent_provider("bounded");
    let strings = everything_this_machine_can_say();
    let bound = SourcePolicy::InRegion("the EU".to_owned());

    let refused = read
        .chosen()
        .unwrap()
        .asking(read.providers(), Some(&bound))
        .expect("the choice names a provider the list has")
        .unwrap_err();
    assert!(
        matches!(refused, NotAllowed::RegionUnstated { .. }),
        "{refused:?}"
    );
    let said = refused.said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(
        said.text().contains("Somewhere has not said where it runs"),
        "{said}"
    );
    assert!(said.text().contains("in the EU only"), "{said}");
    assert!(!said.text().contains("does not meet"), "{said}");
    assert!(!said.text().contains("outside"), "{said}");

    // The acceptance beside it: the same bound, a provider that said.
    let home = a_login_of_our_own("declared");
    let at = the_settings_of(&home);
    let mut choosing = Choosing::at(&at).unwrap();
    choosing
        .adding(
            Provider::checked(
                "Mistral",
                "https://api.mistral.ai",
                Region::Declared("the EU".to_owned()),
                None,
            )
            .unwrap(),
        )
        .unwrap();
    choosing
        .answered_by(Some(Picked::from_a_provider("Mistral", "a-model").unwrap()))
        .unwrap();
    let declared = Settings::at(&at).unwrap();
    assert!(
        declared
            .chosen()
            .unwrap()
            .asking(declared.providers(), Some(&bound))
            .unwrap()
            .is_ok(),
        "a provider that said it runs in the region was refused"
    );
}

/// **A personal machine with no bound is not affected.** Unknown is honest
/// rather than forbidden: with no rule, the same choice is permitted, and there
/// is no refusal — so nothing about a machine no organisation manages changes
/// because a provider was silent about its region.
#[test]
fn a_machine_with_no_bound_puts_the_question_to_the_provider_the_person_chose() {
    let (_, read) = a_machine_choosing_a_silent_provider("unbounded");

    let permitted = read
        .chosen()
        .unwrap()
        .asking(read.providers(), None)
        .expect("the choice names a provider the list has")
        .unwrap();
    assert_eq!(
        permitted.source(),
        &alo_models::InferenceSource::Hosted {
            provider: "Somewhere".to_owned(),
            region: Region::Unknown,
        }
    );
    assert!(permitted.causes_egress());
}

/// **What a person is shown beside the answer says unknown, not a guessed
/// place.** The clause for this source, in the machine's vocabulary, says the
/// provider has not said where it runs and names no region — `.eu` in the
/// address is not one.
#[test]
fn what_the_person_is_shown_for_such_a_provider_says_unknown_and_names_no_place() {
    let (_, read) = a_machine_choosing_a_silent_provider("shown");
    let strings = everything_this_machine_can_say();

    let shown = read
        .chosen()
        .unwrap()
        .source(read.providers())
        .unwrap()
        .shown(&strings);
    assert!(shown.contains("has not said where it runs"), "{shown}");
    assert!(shown.contains("Somewhere"), "{shown}");
    assert!(!shown.contains("in the EU"), "{shown}");
    assert!(!shown.contains("eu"), "{shown}");
}
