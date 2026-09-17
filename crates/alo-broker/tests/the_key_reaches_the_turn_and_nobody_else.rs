//! The approving key crosses from the broker to the turn as one file, readable
//! by the broker's group and nobody else, and a key in any other shape is not
//! believed.
//!
//! Task 2 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md` inherits *how
//! the broker's approving key reaches the turn that issues tokens*. The half
//! that matters is the refusals: a key somebody else could have written, read,
//! or planted is a key whose tokens somebody else can issue.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
use std::path::PathBuf;
use std::time::SystemTime;

use alo_broker::handing_over::{NotHandedOver, hand_over_a_fresh_key, the_key_handed_over};
use alo_broker::{Identity, SystemVerb, our_group, our_user};

/// A directory of this test's own.
fn somewhere(what: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!("alo-broker-key-{}-{what}", std::process::id()));
    drop(std::fs::remove_dir_all(&at));
    std::fs::create_dir_all(&at).unwrap();
    at
}

/// A verb to prove things with.
fn add_a_printer() -> SystemVerb {
    SystemVerb::AddPrinter(Identity::of_what_was_reported(
        b"ipp://printer.local/ipp/print",
    ))
}

/// **The key handed over is the key the broker holds**: a token the turn issues
/// under what it read is one the broker believes. The file is `0440`, in the
/// group it was handed to, and nothing is left beside it.
#[test]
fn the_key_the_turn_reads_is_the_key_the_broker_holds() {
    let at = somewhere("same").join("approving.key");
    let the_brokers = hand_over_a_fresh_key(&at, our_group()).unwrap();
    let the_turns = the_key_handed_over(&at, our_user()).unwrap();

    let verb = add_a_printer();
    let token = the_turns.issue(&verb, 3, SystemTime::now());
    assert!(the_brokers.issued(&verb, &token));

    let about = std::fs::metadata(&at).unwrap();
    assert_eq!(about.permissions().mode() & 0o7777, 0o440);
    assert_eq!(about.gid(), our_group());
    assert_eq!(about.len(), 32);
    let beside: Vec<_> = std::fs::read_dir(at.parent().unwrap())
        .unwrap()
        .flatten()
        .map(|entry| entry.file_name())
        .collect();
    assert_eq!(beside, [std::ffi::OsString::from("approving.key")]);
}

/// **A restarted broker's key replaces the last one**, and a token issued under
/// the old key is not genuine under the new — which is why the turn reads the
/// file each time rather than once.
#[test]
fn a_new_key_replaces_the_old_and_its_tokens_with_it() {
    let at = somewhere("replaced").join("approving.key");
    let before = hand_over_a_fresh_key(&at, our_group()).unwrap();
    let verb = add_a_printer();
    let old = the_key_handed_over(&at, our_user())
        .unwrap()
        .issue(&verb, 1, SystemTime::now());
    assert!(before.issued(&verb, &old));

    let after = hand_over_a_fresh_key(&at, our_group()).unwrap();
    assert!(!after.issued(&verb, &old));
    let new = the_key_handed_over(&at, our_user())
        .unwrap()
        .issue(&verb, 1, SystemTime::now());
    assert!(after.issued(&verb, &new));
}

/// **A key in any other shape is not believed**: one others can read, one its
/// group can write, one that is executable or set-id, one owned by somebody
/// other than the broker, one too short or too long, a link to a key, and no
/// key at all.
#[test]
fn a_key_anybody_else_could_have_written_or_read_is_not_believed() {
    let here = somewhere("refused");
    let at = here.join("approving.key");
    let _the_brokers = hand_over_a_fresh_key(&at, our_group()).unwrap();

    for mode in [0o444, 0o460, 0o640 | 0o002, 0o540, 0o4440, 0o660] {
        std::fs::set_permissions(&at, std::fs::Permissions::from_mode(mode)).unwrap();
        assert!(
            matches!(
                the_key_handed_over(&at, our_user()),
                Err(NotHandedOver::OpenToOthers { .. })
            ),
            "a key of mode {mode:o} was believed"
        );
    }
    std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o440)).unwrap();
    assert!(the_key_handed_over(&at, our_user()).is_ok());

    assert!(matches!(
        the_key_handed_over(&at, our_user().wrapping_add(1)),
        Err(NotHandedOver::NotTheBrokers { .. })
    ));

    for length in [0, 31, 33, 4096] {
        let wrong = here.join(format!("wrong-{length}.key"));
        std::fs::write(&wrong, vec![7_u8; length]).unwrap();
        std::fs::set_permissions(&wrong, std::fs::Permissions::from_mode(0o440)).unwrap();
        assert!(
            matches!(
                the_key_handed_over(&wrong, our_user()),
                Err(NotHandedOver::NotAKey)
            ),
            "a key of {length} bytes was believed"
        );
    }

    let linked = here.join("linked.key");
    std::os::unix::fs::symlink(&at, &linked).unwrap();
    assert!(matches!(
        the_key_handed_over(&linked, our_user()),
        Err(NotHandedOver::NotAFile)
    ));

    assert!(matches!(
        the_key_handed_over(&here.join("nothing.key"), our_user()),
        Err(NotHandedOver::Unreadable(_))
    ));
}

/// A key that cannot be written is not handed over, and the key that was there
/// before is left exactly as it was.
#[test]
fn a_key_that_cannot_be_written_leaves_the_last_one_in_place() {
    let here = somewhere("unwritable");
    let at = here.join("approving.key");
    let before = hand_over_a_fresh_key(&at, our_group()).unwrap();

    let nowhere = here.join("no-such-directory").join("approving.key");
    assert!(matches!(
        hand_over_a_fresh_key(&nowhere, our_group()),
        Err(NotHandedOver::Unreadable(_))
    ));

    let verb = add_a_printer();
    let token = the_key_handed_over(&at, our_user())
        .unwrap()
        .issue(&verb, 2, SystemTime::now());
    assert!(before.issued(&verb, &token));
}
