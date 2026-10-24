//! The two lines that decide whether a person can keep their provider key.
//!
//! `alo-keyring.service` unlocks the login keyring with a password of no
//! characters, delivered as one newline on standard input. The unit argues the
//! decision; this holds the mechanism in place.
//!
//! # This is a revert guard, and it is not a proof of service
//!
//! Saying which it is matters, because this repository has four recorded cases
//! of a check standing in for the thing it was named after. **This one does
//! not start the daemon and cannot tell you a key is storable.** What it can
//! tell you is that nobody removed the byte.
//!
//! That is worth a test because of how the fault behaves. On 2026-10-22 the
//! unit shipped `StandardInput=null` — end of file, which is not an empty
//! password. The daemon started, systemd reported the unit `active`, it took
//! `org.freedesktop.secrets` on the bus, and it created **no collection at
//! all**, so every key a person tried to keep died on *object does not exist at
//! path …/collection/login*. Delete `StandardInputData=` today and every other
//! check in this repository still passes: `--version` still runs the binary,
//! the activation file still names the service, `systemctl` still says
//! `active`. The absence is also the **default**, which is what it was.
//!
//! So the text is the only thing here that can fail, which is the narrow case
//! where asserting text is right rather than lazy.
//!
//! # What proves service, and where it runs
//!
//! A round trip: store a key through one process, read it back in another.
//! Two of those exist.
//!
//! - `alo-secrets/tests/a_keyring_unlocked_the_way_the_machine_unlocks_it.rs`
//!   drives `TheKeyring` against a daemon unlocked with an empty password, and
//!   **runs on every build**. It was shown to fail against the bug: fed end of
//!   file instead, both its cases panic with *the fixture's keyring never
//!   offered a collection to store a secret in*.
//! - The unit itself, started by systemd under the image's own systemd 257 in a
//!   container built from the recipe's pinned base, where a key stored by one
//!   process was read back by another and survived two restarts. **That was
//!   measured once, by hand, on 2026-10-22, and nothing runs it again.**
//!   `docs/misreadings/a-service-that-starts-and-serves-nothing.md` holds it.
//!
//! Neither of those covers the unit's ordering against `user@1000.service`,
//! which the container run had to strip to start at all.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_image::THE_IMAGE;

/// Where the unit lives, relative to `image/`.
const THE_UNIT: &str = "usr/lib/systemd/user/alo-keyring.service";

/// What the unit must say, and why each half of it.
///
/// `StandardInput=data` chooses the source; `StandardInputData=Cg==` is the
/// payload, base64 for a single newline. Either one alone does nothing: the
/// first with no data is an empty stream, which is end of file again.
const BOTH_LINES: [&str; 2] = ["StandardInput=data", "StandardInputData=Cg=="];

/// The unit, read off the image this repository ships.
fn the_unit() -> String {
    let at = Path::new(THE_IMAGE).join(THE_UNIT);
    match std::fs::read_to_string(&at) {
        Ok(text) => text,
        Err(why) => panic!("the keyring unit this repository ships did not read: {why}"),
    }
}

/// **The keyring is handed an empty password, not an empty stream.**
#[test]
fn the_keyring_unit_delivers_one_newline_on_standard_input() {
    let unit = the_unit();
    for line in BOTH_LINES {
        assert!(
            unit.lines().any(|said| said.trim() == line),
            "`{line}` is not in {THE_UNIT}. Without both of these the daemon \
             starts, goes active, serves org.freedesktop.secrets and creates no \
             collection, so no key a person adds can be stored."
        );
    }
}

/// **And it does not say the thing that was broken**, which a revert would
/// reintroduce by name rather than by deletion.
///
/// Separate from the test above because the two fail for different reasons: one
/// catches the line going missing, the other catches it being changed back.
/// `StandardInput=null` beside `StandardInputData=` would also be refused by
/// systemd, but a unit that fails to start is a different symptom from one that
/// starts and holds nothing, and only the second is what shipped.
#[test]
fn the_keyring_unit_does_not_take_its_input_from_nothing() {
    let unit = the_unit();
    assert!(
        !unit.lines().any(|said| said.trim() == "StandardInput=null"),
        "{THE_UNIT} is back to `StandardInput=null`, which is end of file and \
         not an empty password: see \
         docs/misreadings/a-service-that-starts-and-serves-nothing.md"
    );
}
