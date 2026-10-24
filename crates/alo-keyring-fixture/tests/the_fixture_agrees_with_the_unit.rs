//! This fixture feeds the daemon exactly what the image's unit feeds it.
//!
//! **This is the guard for the fault that actually happened.** On 2026-10-22
//! the unit shipped `StandardInput=null` — end of file, which is not an empty
//! password — and no test could see it, because the fixture fed
//! a synthetic password of its own instead. The daemon started, systemd called
//! `active`, it took `org.freedesktop.secrets` on the bus, and it created no
//! collection at all, so every key a person tried to keep died on *object does
//! not exist at path …/collection/login*.
//!
//! The revert guard in
//! `alo-image/tests/the_keyring_is_unlocked_with_an_empty_password.rs` catches
//! that line being deleted or put back. **It does not catch this**: the fixture
//! and the unit drifting apart again, which is what went wrong and which
//! happened without anybody touching the unit at all. One of them changing is
//! enough, and either direction fails here.
//!
//! So this reads the unit, decodes what it puts on standard input, and asserts
//! it is the bytes this crate sends. The agreement becomes something the build
//! checks rather than something a person noticed once.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

use alo_keyring_fixture::WHAT_THE_UNIT_PUTS_ON_STDIN;

/// The unit, found from this crate rather than through `alo-image`, so that a
/// dev-only fixture does not take a dependency on the image crate to read one
/// file.
const THE_UNIT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../image/usr/lib/systemd/user/alo-keyring.service"
);

/// What the unit declares, decoded.
fn what_the_unit_sends() -> Vec<u8> {
    let text = match std::fs::read_to_string(THE_UNIT) {
        Ok(text) => text,
        Err(why) => panic!("the keyring unit this repository ships did not read: {why}"),
    };
    let Some(said) = text
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("StandardInputData="))
    else {
        panic!(
            "{THE_UNIT} declares no `StandardInputData=`. Without it the daemon \
             is handed end of file, which is not an empty password, and no key \
             a person adds can be stored."
        )
    };
    STANDARD
        .decode(said)
        .expect("`StandardInputData=` is base64, as systemd defines it")
}

/// **The fixture sends what the unit sends.**
///
/// Fails if either side changes alone, which is the whole point: the bug this
/// guards against was introduced by the two disagreeing, not by either one
/// being obviously wrong.
#[test]
fn the_fixture_feeds_what_the_unit_feeds() {
    assert_eq!(
        what_the_unit_sends(),
        WHAT_THE_UNIT_PUTS_ON_STDIN,
        "the fixture and `alo-keyring.service` disagree about what reaches the \
         daemon's standard input. Whichever is right, every keyring test in \
         this repository is now measuring a machine that does not ship."
    );
}

/// **And the unit really does declare it**, so the test above cannot pass by
/// comparing two absences.
///
/// Without this, a unit with no `StandardInputData=` at all would be caught by
/// a panic inside the helper rather than by an assertion, which is a worse
/// failure to read.
#[test]
fn the_unit_declares_what_it_sends_at_all() {
    assert!(
        !what_the_unit_sends().is_empty(),
        "the unit sends nothing, which is end of file and not an empty password"
    );
}
