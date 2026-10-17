//! **A forgotten password never loses a person's data**, which is a launch
//! item in the owner's words and was a wall rather than a loss.
//!
//! # What was wrong
//!
//! The password is an Argon2id hash and **not a key**. ADR 0054 encrypts the
//! disk against a chip-sealed PIN or a passphrase, and the recovery key is
//! written down *and typed back* before an install finishes. So a person
//! forgetting their password stands at their own sign-in screen with every
//! byte intact, on a disk that opens, and cannot get in.
//!
//! Nothing in `Accounts` set a new one: it offered `none`, `created`,
//! `signs_in`, `how_many`, `numbers` and `name_of`. This is the door through
//! the wall.
//!
//! # Why it grants nobody anything
//!
//! The authority is the recovery key, checked before this is reached. Whoever
//! holds it can already read the whole root filesystem by opening the disk, so
//! setting a new password hands them nothing they did not have — which is the
//! argument `ROADMAP.md` records.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_accounts::{Accounts, NotCreated, NotSetAgain, NotSignedIn};

/// The person this machine has.
const THEM: &str = "disan";

/// What they will not remember.
const FORGOTTEN: &str = "the one they forgot";

/// What they set instead.
const CHOSEN: &str = "the one they chose at the recovery screen";

/// A machine with one account on it.
fn a_machine_with_an_account() -> Accounts {
    let mut accounts = Accounts::none().expect("a machine that can hash");
    accounts
        .created(THEM, 1000, FORGOTTEN)
        .expect("an account this machine has room for");
    accounts
}

/// **The whole of it: forgotten, set again, and in.**
#[test]
fn a_person_who_forgot_their_password_signs_in_with_a_new_one() {
    let mut accounts = a_machine_with_an_account();

    accounts
        .password_set_again(THEM, CHOSEN)
        .expect("a person at the recovery screen");

    accounts
        .signs_in(THEM, CHOSEN)
        .expect("the password they just set");
}

/// **The old one stops working**, which is the half that makes this a change
/// of password rather than an addition of one.
#[test]
fn the_forgotten_password_no_longer_signs_anybody_in() {
    let mut accounts = a_machine_with_an_account();
    accounts
        .password_set_again(THEM, CHOSEN)
        .expect("set again");

    assert_eq!(
        accounts.signs_in(THEM, FORGOTTEN),
        Err(NotSignedIn::Refused),
        "the password they forgot still works, so nothing was replaced"
    );
}

/// **An unknown name is refused the way a wrong password is**, and this is the
/// test that matters for what this door gives away.
///
/// Sign-in spends work so that *wrong password* and *no such account* are one
/// value, one sentence and one duration. A recovery screen that answered *no
/// such account* would hand back, at a screen anybody can reach, exactly the
/// distinction sign-in exists to withhold.
#[test]
fn a_name_this_machine_does_not_have_is_told_only_that_it_was_refused() {
    let mut accounts = a_machine_with_an_account();

    let refused = accounts
        .password_set_again("somebody-else", CHOSEN)
        .expect_err("this machine has no such account");

    assert_eq!(refused, NotSetAgain::Refused);
    assert_eq!(
        refused.word(),
        NotSignedIn::Refused.word(),
        "the recovery screen says something sign-in does not, so the two can be told apart"
    );
}

/// **An empty new password is refused and the account is untouched.**
///
/// The account must still open with what it had: a failed attempt that left
/// somebody with no usable password at all would be worse than the forgotten
/// one this exists to undo.
#[test]
fn an_empty_new_password_is_refused_and_changes_nothing() {
    let mut accounts = a_machine_with_an_account();

    let refused = accounts
        .password_set_again(THEM, "")
        .expect_err("an account needs a password");

    assert_eq!(refused, NotSetAgain::NotTaken(NotCreated::NoPassword));
    accounts
        .signs_in(THEM, FORGOTTEN)
        .expect("the account must be exactly as it was");
}

/// **Setting the same password twice does not produce the same stored bytes**,
/// because a fresh salt is drawn each time.
///
/// Two machines whose people chose the same password must not be able to tell,
/// and neither must anybody reading the file.
#[test]
fn the_same_password_set_twice_is_not_the_same_on_disk() {
    let mut one = a_machine_with_an_account();
    let mut other = a_machine_with_an_account();

    one.password_set_again(THEM, CHOSEN).expect("set again");
    other.password_set_again(THEM, CHOSEN).expect("set again");

    // Observed the way the machine observes it: through the file it writes.
    let folder = std::env::temp_dir().join(format!("alo-accounts-salt-{}", std::process::id()));
    drop(std::fs::create_dir_all(&folder));
    let at_one = folder.join("one.toml");
    let at_other = folder.join("other.toml");
    alo_accounts::kept(&at_one, &one).expect("a file this test may write");
    alo_accounts::kept(&at_other, &other).expect("a file this test may write");

    assert_ne!(
        std::fs::read_to_string(&at_one).expect("what this machine wrote"),
        std::fs::read_to_string(&at_other).expect("what the other wrote"),
        "the same password produced the same bytes, so the salt is not fresh"
    );

    drop(std::fs::remove_dir_all(&folder));
}

/// **Nobody else's account is touched.**
#[test]
fn setting_one_persons_password_leaves_everybody_else_alone() {
    let mut accounts = a_machine_with_an_account();
    accounts
        .created("mac", 1001, "what mac chose")
        .expect("a second account");

    accounts
        .password_set_again(THEM, CHOSEN)
        .expect("set again");

    accounts
        .signs_in("mac", "what mac chose")
        .expect("the other account must be untouched");
}
