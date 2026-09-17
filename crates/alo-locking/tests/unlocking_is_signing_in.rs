//! Unlocking is authenticated by the same composition signing in uses, and
//! never by a second, weaker road.
//!
//! The unit tests in `src/unlocking.rs` show the right password unlocking and
//! every wrong one refused. What they cannot show is that there is no *other*
//! way through — a remembered-session shortcut, a PIN, a direct call to the
//! account store that skips the greeting's order. That is a property of the
//! whole crate's source, so this reads it: exactly one place asks whether
//! anybody is who they say, and it is `alo_greeting::Greeting::signs_in`.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::PathBuf;

/// Every Rust file under this crate's `src`, with only its code — comments
/// are prose and may name anything.
fn the_code() -> Vec<(String, String)> {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let Ok(entries) = fs::read_dir(&src) else {
        panic!("this crate's source could not be read");
    };
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            panic!("{} could not be read", path.display());
        };
        let code = text
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        let named = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        found.push((named, code));
    }
    found
}

/// The fixture file makes accounts and signs one in to have a session to
/// lock; it is `cfg(test)` and never compiled into the crate.
const THE_FIXTURE: &str = "testing.rs";

/// **One call checks a password, and it is the greeting's.** Outside the
/// test fixture, `signs_in` appears once in this crate's code — on a
/// `Greeting` — and nothing reaches past it to the account store, the hash or
/// a stored credential.
#[test]
fn the_only_road_through_the_lock_is_the_greeting() {
    let code = the_code();
    let calls: Vec<&String> = code
        .iter()
        .filter(|(named, _)| named != THE_FIXTURE)
        .flat_map(|(named, text)| text.matches(".signs_in(").map(move |_| named))
        .collect();
    assert_eq!(
        calls,
        vec![&"unlocking.rs".to_owned()],
        "a password is checked somewhere other than the greeting's one call"
    );

    let Some((_, unlocking)) = code.iter().find(|(named, _)| named == "unlocking.rs") else {
        panic!("unlocking.rs is not where this test looks for it");
    };
    assert!(
        unlocking.contains("greeting.signs_in(name, password)"),
        "the unlock does not go through the greeting"
    );
    assert!(unlocking.contains("Greeting::of(accounts, uid,"));
}

/// **Nothing here holds a credential or a way around one.** No password is
/// kept, no hash is read, no PIN or token is a second key to the lock, and the
/// account store's own verification is never called directly.
#[test]
fn nothing_here_is_a_weaker_key_to_the_lock() {
    for (named, code) in the_code() {
        if named == THE_FIXTURE {
            continue;
        }
        for weaker in [
            "verify",
            "Hashed",
            "argon2",
            "pin:",
            "Pin",
            "token",
            "remember",
            "password: String",
            "password =",
            "String::from(password",
            "password.to_owned",
            "password.to_string",
        ] {
            assert!(
                !code.contains(weaker),
                "{named} contains `{weaker}`: the lock has one key, and it is the sign-in"
            );
        }
    }
}
