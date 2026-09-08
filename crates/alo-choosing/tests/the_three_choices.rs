//! The three choices a person has, read off a real settings file.
//!
//! `docs/features.md`, on 2026-09-08: **local models**, **your own API
//! provider**, and **alo**. This walks the first two through the file a
//! settings panel writes, and the third is the second — [ADR 0014](../../../docs/decisions/0014-alos-own-model-is-a-provider-like-any-other.md)
//! makes alo's own service one more provider with no special case anywhere in
//! the code, so there is nothing here for it that is not here for Mistral.
//!
//! **These are model-source choices and not privacy levels.** Where a question
//! is answered follows from the choice; what a person is promised about
//! confinement is [ADR 0021](../../../docs/decisions/0021-what-a-service-on-this-machine-vouches-for.md),
//! which is proposed and unaccepted, and nothing here turns on it.
//!
//! `on_this_machine.rs` is this file's sibling and covers the first choice on a
//! real disk. What is here is what that file could not say until a provider was
//! something a person could write down.
//!
//! # Nothing here has a credential in it
//!
//! Not because the tests are careful — because **the file has nowhere to put
//! one**. The keyring name a provider's key lives under is derived from the
//! provider's own name, so there is no field to paste a key into, and a file
//! that invents one is refused. That is asserted below rather than described.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use alo_choosing::{NotSet, Settings, Which, where_it_is};
use alo_models::{InferenceSource, Region, SourcePolicy};

/// A directory nothing else in this run is using, standing in for a home.
fn a_home_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let home = std::env::temp_dir().join(format!(
        "alo-three-choices-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    drop(fs::remove_dir_all(&home));
    fs::create_dir_all(&home).unwrap();
    home
}

/// Settings written where a settings panel would write them, and read back
/// through the only door there is.
fn written_and_read(what: &str, said: &str) -> Result<Settings, NotSet> {
    let home = a_home_of_our_own(what);
    let at = where_it_is(None, Some(home.as_os_str())).unwrap();
    fs::create_dir_all(at.parent().unwrap()).unwrap();
    fs::write(&at, said).unwrap();
    Settings::at(&at)
}

/// A file choosing a provider the person added, in the shape the contract
/// documents.
const CHOOSING_A_PROVIDER: &str = "format = 2\n\
     \n\
     [answers]\n\
     provider = { name = \"Mistral\", model = \"mistral-small-latest\" }\n\
     \n\
     [[provider]]\n\
     name = \"Mistral\"\n\
     endpoint = \"https://api.mistral.ai\"\n\
     region = \"the EU\"\n";

/// **The second choice, written down and read back.**
///
/// The person picked a provider they added and a model from it, and every part
/// of that survives the round trip: which provider, which model, where it is,
/// and the region **whoever added it stated** — never one inferred from the
/// address, which would be a reassuring label over a breach.
#[test]
fn a_provider_the_person_added_is_chosen_and_persists() {
    let settings = written_and_read("provider", CHOOSING_A_PROVIDER).unwrap();

    let chosen = settings.chosen().unwrap();
    assert_eq!(chosen.provider(), Some("Mistral"));
    assert_eq!(chosen.model(), "mistral-small-latest");

    let provider = settings.provider().unwrap();
    assert_eq!(provider.name, "Mistral");
    assert_eq!(provider.endpoint, "https://api.mistral.ai");
    assert_eq!(provider.region, Region::Declared("the EU".to_owned()));
}

/// **A key is not in the file, and there is nowhere to put one.**
///
/// The keyring name is derived — `provider/<the person's own name for it>` —
/// so the settings file never holds a credential, and a person who pastes their
/// API key into it is told the key is not one this file has rather than left
/// with a credential on their disk that alo OS quietly read.
///
/// `needs-a-key = false` is the compatible service that takes none: nothing is
/// looked up and nothing is sent.
#[test]
fn a_credential_is_referred_to_and_never_written_down() {
    let settings = written_and_read("keyed", CHOOSING_A_PROVIDER).unwrap();
    let key = settings.provider().unwrap().key.clone().unwrap();
    assert_eq!(key.as_str(), "provider/Mistral");

    let keyless = written_and_read(
        "keyless",
        &CHOOSING_A_PROVIDER.replace("region = \"the EU\"\n", "needs-a-key = false\n"),
    )
    .unwrap();
    assert!(
        keyless.provider().unwrap().key.is_none(),
        "a service that takes no credential was given a reference to one"
    );

    // And the file has no field a key could go in at all.
    let pasted = written_and_read(
        "pasted",
        &CHOOSING_A_PROVIDER.replace("region = \"the EU\"\n", "key = \"sk-live-0123456789\"\n"),
    );
    assert!(matches!(pasted, Err(NotSet::NotUnderstood { .. })));
}

/// **A choice that names a provider the list does not have is refused**, and
/// nothing in the file is used.
///
/// The same rule one list to the right of `NotBrought`: a choice is a reference
/// into a list the person keeps, and a reference resolving to nothing is a file
/// whose two halves disagree. Reading the half that parsed would be the machine
/// choosing the rest of somebody's settings for them.
#[test]
fn choosing_a_provider_nobody_added_refuses_the_file() {
    let refused = written_and_read(
        "unlisted",
        "format = 2\n\n[answers]\nprovider = { name = \"Nobody\", model = \"m\" }\n",
    )
    .unwrap_err();

    assert!(
        matches!(refused, NotSet::NoSuchProvider { ref provider, .. } if provider == "Nobody"),
        "{refused:?}"
    );
}

/// **A provider choice never becomes a local answer.**
///
/// The no-fallback guarantee at the configuration layer, and the reason
/// `Settings::chosen` stopped answering `Option<&Chosen>`: a provider choice
/// used to read as *nobody has chosen*, and a caller that then looked for a
/// runtime would have answered a question the person addressed elsewhere with
/// whatever happened to be running here.
///
/// So: nothing about this choice is on this machine, and it is not mistaken for
/// a set of brought weights that happens to share the name either.
#[test]
fn a_provider_choice_is_not_a_local_one() {
    let settings = written_and_read(
        "no-fallback",
        &CHOOSING_A_PROVIDER.replace(
            "[[provider]]",
            "[[brought]]\n\
             id = \"mistral-small-latest\"\n\
             bytes-on-disk = 4700000000\n\
             drives-verbs = \"reliably\"\n\
             \n\
             [[provider]]",
        ),
    )
    .unwrap();

    let chosen = settings.chosen().unwrap();
    assert!(
        chosen.on_this_machine().is_none(),
        "a provider choice read as a model on this machine"
    );
    assert!(
        settings.weights().is_none(),
        "a provider choice resolved to weights on this disk that share its name"
    );
    assert!(settings.brought().get("mistral-small-latest").is_some());
}

/// **Where a provider's answer comes from is the provider's own fact**, never
/// `ThisMachine` — and a rule that keeps questions here refuses it rather than
/// answering it quietly.
///
/// This is the guarantee that matters most in this file. A version of
/// `Picked::source` that could answer without the list would make a local
/// choice and a remote one the same value, which is the silent switch between
/// local and remote processing that nothing may do.
#[test]
fn a_provider_is_never_mistaken_for_this_machine() {
    let settings = written_and_read("source", CHOOSING_A_PROVIDER).unwrap();
    let chosen = settings.chosen().unwrap();

    assert_eq!(
        chosen.source(settings.providers()),
        Some(InferenceSource::Hosted {
            provider: "Mistral".to_owned(),
            region: Region::Declared("the EU".to_owned()),
        })
    );

    // A machine whose organisation keeps questions on it refuses the choice,
    // naming the rule — and offers nothing in its stead.
    let refused = chosen
        .asking(settings.providers(), Some(&SourcePolicy::ThisMachineOnly))
        .expect("the provider resolves in the person's own list")
        .expect_err("a rule that keeps questions here does not permit a provider");
    assert!(matches!(
        refused,
        alo_models::NotAllowed::NotThisMachine { .. }
    ));

    // And the same rule permits the same person's local choice, so the refusal
    // above is about the place rather than about the rule refusing everything.
    let local = written_and_read(
        "source-local",
        "format = 1\n\n[answers]\ncatalogue = \"m\"\n",
    )
    .unwrap();
    let local = local.chosen().unwrap().clone();
    assert!(
        local
            .asking(
                &alo_models::Providers::default(),
                Some(&SourcePolicy::ThisMachineOnly)
            )
            .unwrap()
            .is_ok()
    );
}

/// **A machine configured before providers existed is unchanged.**
///
/// `format = 1` is still read, exactly as it was, and nothing rewrites it or
/// asks the person to. Expand, then migrate, then contract — and nothing here
/// is the contract.
#[test]
fn settings_written_before_providers_still_read() {
    let settings = written_and_read(
        "format-one",
        "format = 1\n\n[answers]\ncatalogue = \"mistral-small\"\n\n[reading]\nlanguages = [\"de\"]\n",
    )
    .unwrap();

    let chosen = settings.chosen().unwrap().on_this_machine().unwrap();
    assert_eq!(chosen.which(), Which::Catalogue);
    assert_eq!(chosen.model(), "mistral-small");
    assert!(settings.providers().configured.is_empty());
    assert_eq!(settings.languages().len(), 1);
}
