#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::SystemTime;

use alo_capability::Grants;

use super::{KeepsSecrets, NoKeyringHere, NotKept};
use crate::portal::Portal;
use crate::request::Request;

/// **A machine with no keyring refuses a secret, in the word the contract
/// already has.**
#[test]
fn a_secret_asked_of_a_machine_with_no_keyring_is_unavailable() {
    let mut into: Vec<u8> = Vec::new();
    let refused = NoKeyringHere
        .hand_over(
            &Request::of("org.kde.dolphin", Portal::Secret).expect("a request"),
            &Grants::default(),
            SystemTime::UNIX_EPOCH,
            &mut into,
        )
        .expect_err("a machine with no keyring has no secret to hand over");

    assert_eq!(refused, NotKept::Unavailable);
}

/// **Nothing is written when there is nothing to write.**
///
/// A caller that read the buffer after a refusal would otherwise read whatever
/// was in it, and a buffer with bytes in it after a refused secret is the
/// shape of a secret leaking.
#[test]
fn nothing_reaches_the_buffer_on_a_machine_with_no_keyring() {
    let mut into: Vec<u8> = b"what was already here".to_vec();
    let before = into.clone();

    drop(NoKeyringHere.hand_over(
        &Request::of("org.kde.dolphin", Portal::Secret).expect("a request"),
        &Grants::default(),
        SystemTime::UNIX_EPOCH,
        &mut into,
    ));

    assert_eq!(
        into, before,
        "a refusal must not write, and must not clear what a caller already had"
    );
}
