//! A knock can be made here from a verified sign-in, and from nothing else —
//! and this crate is not a second authenticator, a second reader of the wire,
//! or a second thing that opens sessions.
//!
//! Three of the four are held by reading this crate's own source and manifest
//! rather than by exercising a road, for the reason `alo-clipboard` reads a
//! manifest to keep a turn out of a clipboard: what matters is not that today's
//! code does not do these things, but that tomorrow's cannot without somebody
//! deleting a test that says why.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// This crate's own directory.
fn ours() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_owned()
}

/// Every `.rs` file this crate ships, as a path and its text, with unit tests
/// cut off — a test may knock for a number it made up, and holding one to a
/// rule about shipped code would only teach somebody to spell it differently.
fn every_file() -> Vec<(PathBuf, String)> {
    let mut read = Vec::new();
    for entry in std::fs::read_dir(ours().join("src")).unwrap() {
        let at = entry.unwrap().path();
        if at.extension().is_none_or(|kind| kind != "rs") {
            continue;
        }
        let whole = std::fs::read_to_string(&at).unwrap();
        let code = match whole.split_once("#[cfg(test)]") {
            Some((code, _)) => code.to_owned(),
            None => whole,
        };
        read.push((at, code));
    }
    assert!(read.len() > 4, "this crate has more files than that");
    read
}

/// **`Knock::on_behalf_of` is called once in this crate, in one private
/// function, and that function takes a session.**
///
/// `alo_sessiond::Knock::on_behalf_of` is public — it has to be, because the
/// door's own tests and the opener itself make knocks. What this crate can do
/// with it is the thing being held: one call, reachable only from
/// [`alo_greeting::Greeting::signs_in`], from a value that cannot exist unless
/// a password verified *and* the machine description agreed about the number.
/// A second call added anywhere is a door onto a uid nobody authenticated, and
/// it is a failing build.
#[test]
fn the_only_knock_this_crate_makes_is_made_from_a_session() {
    let mut calls = Vec::new();
    for (at, code) in every_file() {
        for (which, line) in code.lines().enumerate() {
            if line.contains("Knock::on_behalf_of") && !line.trim_start().starts_with("//") {
                calls.push((at.clone(), which + 1, line.trim().to_owned()));
            }
        }
    }

    assert_eq!(
        calls.len(),
        1,
        "a knock is made somewhere new in this crate: {calls:#?}"
    );
    let (at, _, line) = calls.first().unwrap();
    assert!(at.ends_with("greeting.rs"), "{}", at.display());
    assert_eq!(line, "Knock::on_behalf_of(session.uid())");

    // And the function it is in is private, and takes the session. Read as the
    // line above it in the file, because the whole of the argument is that
    // there is no other way in.
    let written = std::fs::read_to_string(ours().join("src/greeting.rs")).unwrap();
    assert!(
        written.contains("fn for_whom(session: &Session) -> Knock {"),
        "the one knock is not made in a private function taking a session"
    );
    assert!(
        !written.contains("pub fn for_whom"),
        "the function that makes the knock is public, so a caller can knock without signing \
         anybody in"
    );
}

/// **There is no second authenticator here.** Nothing in this crate hashes,
/// verifies or compares anything a person typed: `alo-accounts` decides who is
/// here, and a crate above it that grew its own opinion would be two answers to
/// one question — with the timing promise kept in only one of them.
#[test]
fn nothing_here_verifies_a_password_itself() {
    let manifest = std::fs::read_to_string(ours().join("Cargo.toml")).unwrap();
    for rented in ["argon2", "getrandom", "sha2", "hmac", "pbkdf2", "bcrypt"] {
        assert!(
            !manifest.contains(rented),
            "this crate depends on {rented}, which is something only an authenticator needs"
        );
    }
    for (at, code) in every_file() {
        for shape in ["verifies(", "Hashed", "hash(", "Argon"] {
            assert!(
                !code.contains(shape),
                "{} does its own `{shape}`",
                at.display()
            );
        }
    }

    // And what a person typed leaves this crate exactly once, on the line that
    // hands it to the one authenticator this machine has.
    let handed_on: usize = every_file()
        .iter()
        .map(|(_, code)| code.matches("signs_in(name, password)").count())
        .sum();
    assert_eq!(
        handed_on, 1,
        "what somebody typed is handed somewhere other than to alo-accounts"
    );
}

/// **It cannot open a session; it can only ask.** `alo-sessiond` is the
/// privileged component ADR 0024 priced, and a greeter that could open a
/// session itself would be ADR 0018's *one privileged component* argument
/// thrown away a second time. So nothing here names `logind`, a bus, or the
/// half of `alo-sessiond` that decides.
#[test]
fn it_asks_for_a_session_and_cannot_open_one() {
    let manifest = std::fs::read_to_string(ours().join("Cargo.toml")).unwrap();
    for rented in ["zbus", "rustix"] {
        assert!(
            !manifest.contains(rented),
            "this crate depends on {rented}, and asking for a session takes none of it"
        );
    }
    for (at, code) in every_file() {
        for shape in ["Logind", "Opening", "CreateSession"] {
            assert!(
                !code.contains(shape),
                "{} names `{shape}`, which belongs to the component that decides",
                at.display()
            );
        }
    }
}

/// **The wire is `alo-sessiond`'s, unchanged.** Additive only: no new message,
/// no field added to `Knock` — which is the whole of what keeps a password off
/// that wire — and no second reader of the line, because two readers of one
/// conversation is two accounts of it and one of them is a privileged
/// process's.
#[test]
fn the_wire_is_the_openers_own_and_nothing_was_added_to_it() {
    let knock = alo_sessiond::Knock::on_behalf_of(1000);
    assert_eq!(knock.written(), "open 1000");
    assert_eq!(alo_sessiond::Knock::read(&knock.written()).unwrap(), knock);

    // A line with anything else on it is not a knock — a name, a path, a
    // password. The door's own rule, asserted here because this crate is what
    // would have wanted to add one.
    for more in [
        "open 1000 ada",
        "open 1000 hunter2",
        "open 1000 /etc/shadow",
    ] {
        assert!(alo_sessiond::Knock::read(more).is_err(), "{more}");
    }

    // And nothing here parses an answer itself.
    for (at, code) in every_file() {
        assert!(
            !code.contains("\"opened\"") && !code.contains("\"refused\""),
            "{} reads the wire itself rather than through alo-sessiond",
            at.display()
        );
    }
}

/// **It draws nothing.** The screen is task 13's and the desktop lane's, and
/// this crate is deliberately every part of the greeter that is not drawing.
#[test]
fn nothing_here_draws_anything() {
    let manifest = std::fs::read_to_string(ours().join("Cargo.toml")).unwrap();
    for rented in ["smithay", "wayland", "wgpu", "alo-shell", "alo-appearance"] {
        assert!(!manifest.contains(rented), "this crate depends on {rented}");
    }
}
