//! Every word a surface draws is one a running machine can say.
//!
//! **This is for the fault that reaches a person as a key.** On 2026-10-23 a
//! signed-in machine drew `«shell.provider.adding»` in Settings where it should
//! have said *Add a provider* — the row that lets somebody make a fresh machine
//! answer their questions, showing them a string key.
//!
//! Nothing in the drawing was wrong and nothing in the declaring was wrong.
//! There were **two roads to the vocabulary** and the surface took the short
//! one: `alo_saying::everything_this_machine_can_say` reads the shipped bundle,
//! and `crate::what_this_machine_can_say` reads it **and** adds this crate's
//! own words, which `alo-saying` cannot collect because it is a dependency of
//! this crate and a dependency cannot point both ways. `alo-compositor` called
//! the second; `alo-desktop` called the first.
//!
//! # Why a test here rather than one in `alo-desktop`
//!
//! The words are this crate's, so this crate is where their absence is a fact
//! rather than a guess. A test in the binary would be asserting about a
//! vocabulary it had built itself, which is the shape that let this through.
//!
//! **What this does not do** is check which function a binary calls — that is
//! `alo-desktop`'s own source and could change without this noticing. What it
//! does is make the difference between the two roads *visible as a failing
//! test* the moment the shorter one cannot say something a surface draws.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

/// Every word this crate declares for itself, by key.
fn the_shells_own_keys() -> Vec<&'static str> {
    vec![
        "shell.provider.adding",
        "shell.provider.name",
        "shell.provider.address",
        "shell.provider.key",
        "shell.provider.model",
        "shell.provider.this-machine-cannot-see",
        "shell.provider.unencrypted-on-this-network",
    ]
}

/// **The short road cannot say them, and the long road can.**
///
/// Both halves matter. Without the first the test could pass against a bundle
/// that happened to carry these keys, and prove nothing about the two roads.
/// Without the second it would not say which road a surface should take.
#[test]
fn the_shells_own_words_are_sayable_only_by_the_road_that_declares_them() {
    let bundle_only = alo_strings::Strings::of(
        alo_saying::everything_this_machine_can_say().expect("the shipped bundle reads"),
    );
    let whole = alo_shell::what_this_machine_can_say().expect("the whole vocabulary builds");

    let keys = the_shells_own_keys();
    assert!(!keys.is_empty(), "this test asserts about nothing");

    for key in keys {
        let short = bundle_only
            .say(
                &alo_strings::Key::named(key).expect("a declared key is a key"),
                &alo_strings::Filling::nothing(),
            )
            .into_text();
        let long = whole
            .say(
                &alo_strings::Key::named(key).expect("a declared key is a key"),
                &alo_strings::Filling::nothing(),
            )
            .into_text();

        assert!(
            short.contains(key),
            "the bundle alone could say `{key}` as `{short}` — then this test \
             proves nothing about the two roads, because the surface would be \
             right either way"
        );
        assert!(
            !long.contains(key),
            "`{key}` reaches a person as a key even on the road that declares \
             it: `{long}`"
        );
    }
}
