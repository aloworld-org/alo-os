//! **No road enrols encryption without also producing the recovery key and
//! requiring the person to confirm they kept it.**
//!
//! Task 5 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`, word for
//! word, and the reason this crate exists at all. It is held twice, because
//! either half alone would be a green test that proves the wrong thing:
//!
//! - **by walking the road**, so that the one that works is shown working and
//!   every refusal on it is shown stopping;
//! - **by reading this crate's own source**, so that *there is no other road* is
//!   a measurement of the public surface rather than a claim about the road
//!   somebody happened to walk in a test.
//!
//! The second half is what a reader cannot otherwise check. A test that walks
//! one road cannot say there is not a second one three modules away; reading
//! every `pub fn` in the crate can.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_encrypting::{
    Enrolment, HowItUnlocks, NotAPin, NotWhatWasShown, Passphrase, Pin, RecoveryKey, TheChip,
    WhatToAskFor,
};

/// What the rented tool printed on a virtual disk in the pinned base on
/// 2026-09-17 (ADR 0054, measurement 4), without its line ending.
const THE_KEY: &str = "lrhdhrji-vtdlgdkr-ulukcjff-rgckhvrd-kguhibgc-gvihrfgc-ghedrdlr-uihctkkt";

/// This crate's directory.
fn here() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Every Rust file under `src/`, as `(name, text)`.
fn the_source() -> Vec<(String, String)> {
    let mut files: Vec<(String, String)> = std::fs::read_dir(here().join("src"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .map(|path| {
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            (name, std::fs::read_to_string(&path).unwrap_or_default())
        })
        .collect();
    files.sort();
    let names: Vec<&str> = files.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(files.len(), 9, "src/ could not be read: {names:?}");
    assert!(files.iter().all(|(_, text)| !text.is_empty()));
    files
}

/// One file of `src/`, by name.
fn the_file(named: &str) -> String {
    the_source()
        .into_iter()
        .find(|(name, _)| name == named)
        .map(|(_, text)| text)
        .unwrap_or_default()
}

/// Every line of a file that declares something anybody outside this crate can
/// call, trimmed.
fn every_public_function(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| line.starts_with("pub fn ") || line.starts_with("pub const fn "))
        .map(str::to_owned)
        .collect()
}

/// A recovery key, as the tool would have handed one over.
fn a_recovery_key() -> RecoveryKey {
    match RecoveryKey::as_printed(&format!("{THE_KEY}\n")) {
        Ok(key) => key,
        Err(why) => panic!("the tool's own output is a recovery key: {why}"),
    }
}

/// A PIN a person could have typed.
fn a_pin() -> Pin {
    match Pin::typed("197246", "197246") {
        Ok(pin) => pin,
        Err(why) => panic!("six characters twice is a PIN: {why}"),
    }
}

/// **The road that works, walked from end to end** — and the enrolment at the
/// end of it could not have been written without the person's typing.
#[test]
fn the_road_that_works_ends_in_an_enrolment_the_person_confirmed() {
    assert_eq!(
        WhatToAskFor::on_a_machine_with(TheChip::ReadyToUse),
        WhatToAskFor::APin
    );
    let key = a_recovery_key();
    let on_the_screen = key.as_it_is_shown().to_owned();
    assert_eq!(on_the_screen, THE_KEY);

    // The person copies it onto paper and types it back, dashes and all.
    let kept = match key.written_back(&on_the_screen) {
        Ok(kept) => kept,
        Err(again) => panic!("the key typed back is the key: {:?}", again.why()),
    };
    let how = match HowItUnlocks::sealed_to_the_chip(TheChip::ReadyToUse, a_pin()) {
        Ok(how) => how,
        Err(why) => panic!("a chip that is ready can hold a key: {why}"),
    };
    let enrolment = Enrolment::made(how, kept);
    assert!(enrolment.how_it_unlocks().is_sealed_to_the_chip());
    assert_eq!(enrolment.the_person_kept_the_key(), kept);
}

/// **The same road on a machine with no chip**, which is still encrypted and
/// still cannot be finished without the key being typed back.
#[test]
fn a_machine_with_no_chip_takes_the_other_road_and_still_keeps_its_key() {
    for chip in [TheChip::NotReady, TheChip::Absent, TheChip::NotRead] {
        assert_eq!(
            WhatToAskFor::on_a_machine_with(chip),
            WhatToAskFor::APassphrase,
            "{chip:?}"
        );
        assert!(
            HowItUnlocks::sealed_to_the_chip(chip, a_pin()).is_err(),
            "{chip:?} sealed a key"
        );
    }
    let key = a_recovery_key();
    let typed = key.as_it_is_shown().to_owned();
    let kept = match key.written_back(&typed) {
        Ok(kept) => kept,
        Err(again) => panic!("the key typed back is the key: {:?}", again.why()),
    };
    let passphrase = match Passphrase::typed("a longer secret", "a longer secret") {
        Ok(passphrase) => passphrase,
        Err(why) => panic!("fifteen characters twice is a passphrase: {why}"),
    };
    let enrolment = Enrolment::made(HowItUnlocks::a_passphrase(passphrase), kept);
    assert!(!enrolment.how_it_unlocks().is_sealed_to_the_chip());
}

/// **Every refusal on the road stops before an enrolment exists.** Each of
/// these is a person or a program getting something wrong at install, and each
/// of them leaves the machine with nothing enrolled rather than with a disk
/// nobody can recover.
#[test]
fn every_refusal_on_the_road_stops_before_anything_is_enrolled() {
    // The tool printed something that is not a recovery key. There is no key,
    // so there is nothing to show and nothing to type back.
    assert!(RecoveryKey::as_printed("").is_err());
    assert!(RecoveryKey::as_printed("not-a-key").is_err());
    assert!(RecoveryKey::as_printed(&THE_KEY.replace('-', "")).is_err());

    // The person typed nothing back. The key comes back to be shown again, and
    // no proof was made.
    let refused = a_recovery_key().written_back("   ");
    match refused {
        Ok(_) => panic!("nothing typed back was taken as the key"),
        Err(again) => {
            assert_eq!(again.why(), NotWhatWasShown::Nothing);
            assert_eq!(again.shown_again().as_it_is_shown(), THE_KEY);
        }
    }

    // The person typed one character wrong. Same.
    let nearly = THE_KEY.replacen('l', "b", 1);
    match a_recovery_key().written_back(&nearly) {
        Ok(_) => panic!("a key one character wrong was taken as the key"),
        Err(again) => assert_eq!(again.why(), NotWhatWasShown::NotThisKey),
    }

    // The PIN was too short, so there is no PIN to seal anything to.
    assert_eq!(
        Pin::typed("1234", "1234").err(),
        Some(NotAPin::TooShort { at_least: 6 })
    );

    // The chip cannot hold a key, so no key is sealed to it.
    assert!(HowItUnlocks::sealed_to_the_chip(TheChip::Absent, a_pin()).is_err());
}

/// **There is one way to make an enrolment and it takes the proof.** Read off
/// this crate's own source rather than asserted about the road above.
#[test]
fn an_enrolment_has_one_maker_and_it_takes_the_proof() {
    let enrolment = the_file("enrolment.rs");
    let makers: Vec<String> = every_public_function(&enrolment);
    assert_eq!(
        makers,
        [
            "pub fn made(how: HowItUnlocks, kept: WrittenDown) -> Self {",
            "pub const fn how_it_unlocks(&self) -> &HowItUnlocks {",
            "pub const fn the_person_kept_the_key(&self) -> WrittenDown {",
        ],
        "the way an enrolment is made changed; this test is where that is decided"
    );
    assert!(
        !enrolment.contains("derive(Default)") && !enrolment.contains("impl Default for Enrolment"),
        "an enrolment with a default is an enrolment nobody confirmed"
    );
}

/// **There is one way to make the proof, and it is a person typing the key
/// back.** `WrittenDown` has no public maker anywhere in the crate, and the one
/// crate-private maker is called from exactly one place: the arm of
/// `RecoveryKey::written_back` that matched.
#[test]
fn the_proof_has_no_maker_but_a_person_typing_the_key_back() {
    let written_down = the_file("written_down.rs");
    let makers = every_public_function(&written_down);
    assert_eq!(
        makers,
        [
            "pub const fn why(&self) -> NotWhatWasShown {",
            "pub fn shown_again(self) -> RecoveryKey {",
        ],
        "a public maker appeared in written_down.rs"
    );
    assert!(
        written_down.contains("pub(crate) const fn because_they_typed_it_back() -> Self {"),
        "the one maker is not where it was"
    );
    assert!(
        !written_down.contains("derive(Default)"),
        "a proof with a default is not a proof"
    );

    // Called from one place in the whole crate, and that place is the matching
    // arm of `written_back`.
    let calls: Vec<String> = the_source()
        .into_iter()
        .filter(|(name, text)| {
            name != "written_down.rs" && text.contains("because_they_typed_it_back()")
        })
        .map(|(name, _)| name)
        .collect();
    assert_eq!(calls, ["recovery_key.rs"], "{calls:?}");
    let recovery_key = the_file("recovery_key.rs");
    assert_eq!(
        recovery_key.matches("because_they_typed_it_back()").count(),
        1,
        "the proof is made in more than one place"
    );
    assert!(
        recovery_key.contains("if typed == self.plain {"),
        "the proof is no longer made from a typing that matched"
    );
}

/// **A recovery key does not survive being typed back**, right or wrong. It is
/// taken by value, so no later line anywhere can be holding one; a wrong typing
/// hands it back inside `Again`, which is a line of code that says out loud that
/// a person is being shown their key a second time.
#[test]
fn a_recovery_key_is_consumed_by_the_typing_that_confirms_it() {
    let recovery_key = the_file("recovery_key.rs");
    assert!(
        recovery_key
            .contains("pub fn written_back(self, typed: &str) -> Result<WrittenDown, Again>"),
        "written_back no longer takes the key by value"
    );
    let makers = every_public_function(&recovery_key);
    assert_eq!(
        makers,
        [
            "pub fn as_printed(printed: &str) -> Result<Self, NotARecoveryKey> {",
            "pub fn as_it_is_shown(&self) -> &str {",
            "pub fn written_back(self, typed: &str) -> Result<WrittenDown, Again> {",
        ],
        "the recovery key's public surface changed; this test is where that is decided"
    );
    assert!(
        !recovery_key.contains("derive(Clone")
            && !recovery_key.contains("impl Clone for RecoveryKey"),
        "a recovery key that can be cloned is a recovery key that outlives its screen"
    );
}

/// **Nothing else in the crate makes any of the three.** Every public function
/// in `src/` is read, and the only ones that can produce an `Enrolment`, a
/// `WrittenDown` or a `RecoveryKey` are the three the tests above name.
#[test]
fn no_other_public_function_in_this_crate_makes_one_of_the_three() {
    let mut makers: Vec<(String, String)> = Vec::new();
    for (name, text) in the_source() {
        let inside: Vec<&str> = text
            .lines()
            .map(str::trim)
            .take_while(|line| *line != "#[cfg(test)]")
            .collect();
        for line in every_public_function(&inside.join("\n")) {
            let makes = ["Enrolment", "WrittenDown", "RecoveryKey"]
                .into_iter()
                .any(|made| {
                    line.split("->")
                        .nth(1)
                        .is_some_and(|answer| answer.contains(made))
                })
                || (line.contains("-> Self") || line.contains("-> Result<Self"))
                    && ["enrolment.rs", "written_down.rs", "recovery_key.rs"].contains(&&*name);
            if makes {
                makers.push((name.clone(), line));
            }
        }
    }
    assert_eq!(
        makers,
        [
            (
                "enrolment.rs".to_owned(),
                "pub fn made(how: HowItUnlocks, kept: WrittenDown) -> Self {".to_owned()
            ),
            (
                "enrolment.rs".to_owned(),
                "pub const fn the_person_kept_the_key(&self) -> WrittenDown {".to_owned()
            ),
            (
                "recovery_key.rs".to_owned(),
                "pub fn as_printed(printed: &str) -> Result<Self, NotARecoveryKey> {".to_owned()
            ),
            (
                "recovery_key.rs".to_owned(),
                "pub fn written_back(self, typed: &str) -> Result<WrittenDown, Again> {".to_owned()
            ),
            (
                "written_down.rs".to_owned(),
                "pub fn shown_again(self) -> RecoveryKey {".to_owned()
            ),
        ],
        "a public function that makes one of the three appeared or moved"
    );
}

/// The crate's own directory, from this test, so that a file added to `src/`
/// without this test being rewritten fails here.
#[test]
fn the_crate_is_the_nine_files_these_tests_read() {
    let names: Vec<String> = the_source().into_iter().map(|(name, _)| name).collect();
    assert_eq!(
        names,
        [
            "chip.rs",
            "enrolment.rs",
            "lib.rs",
            "passphrase.rs",
            "pin.rs",
            "recovery_key.rs",
            "road.rs",
            "unlocking.rs",
            "written_down.rs",
        ]
    );
    assert!(PathBuf::from(here()).join("Cargo.toml").is_file());
}
