//! That Allow once answers one action and widens nothing.

use super::{Answered, ThePermissions};

/// **Allow once answers the action and leaves the grants where they were.**
///
/// The failure this refuses is the easy one: *they allowed it once, so stop
/// asking.* It reads as kindness and is how an approval becomes a session —
/// which `CLAUDE.md` forbids by name: *what a person approves is that sentence,
/// and an approval is never a session.*
#[test]
fn allow_once_answers_the_action_and_widens_nothing() {
    let mut permissions = ThePermissions::holding(2);
    permissions.is_asking("Rename 3 files in Documents/invoices");

    let answered = permissions.allow_once();

    assert_eq!(answered, Answered::AllowedOnce);
    assert_eq!(
        permissions.how_many_grants(),
        2,
        "allowing one action added a standing grant, so an approval became a session"
    );
    assert_eq!(
        permissions.asking(),
        None,
        "the action is still being asked about after it was answered"
    );
}

/// **Deny answers it too, and also widens nothing** — which is worth asserting
/// rather than assuming, because the tempting shortcut is to treat a denial as
/// a standing refusal and stop offering.
#[test]
fn deny_answers_the_action_and_changes_nothing_else() {
    let mut permissions = ThePermissions::holding(1);
    permissions.is_asking("Install Blender");

    assert_eq!(permissions.deny(), Answered::Denied);
    assert_eq!(permissions.how_many_grants(), 1);
    assert_eq!(permissions.asking(), None);
}

/// **Answering twice answers nothing the second time**, and says so rather
/// than quietly doing nothing.
///
/// Two callers produce this: a double press, and a window somebody left open
/// while the task moved on. A surface that read the second as *denied* would
/// record a refusal the person never made.
#[test]
fn answering_when_nothing_is_asked_says_so() {
    let mut permissions = ThePermissions::holding(0);

    assert_eq!(permissions.allow_once(), Answered::NothingWasAsked);
    assert_eq!(permissions.deny(), Answered::NothingWasAsked);

    permissions.is_asking("Join the network \"office\"");
    assert_eq!(permissions.allow_once(), Answered::AllowedOnce);
    assert_eq!(
        permissions.allow_once(),
        Answered::NothingWasAsked,
        "allowing the same action twice allowed it twice"
    );
}

/// **A task holding nothing and asking nothing is the ordinary state**, and
/// the area still has something true to say about it.
#[test]
fn a_task_asking_for_nothing_holds_nothing() {
    let permissions = ThePermissions::nothing_yet();

    assert_eq!(permissions.asking(), None);
    assert_eq!(permissions.how_many_grants(), 0);
}

/// **What is asked is the sentence a person reads**, not a verb name or an
/// identifier.
///
/// The capability model's own rule: a person approves *that sentence*. An area
/// that showed `files.rename` would be asking somebody to approve a thing they
/// have not been told.
#[test]
fn what_is_asked_is_the_sentence_and_not_a_verb_name() {
    let mut permissions = ThePermissions::nothing_yet();
    permissions.is_asking("Rename 3 files in Documents/invoices");

    assert_eq!(
        permissions.asking(),
        Some("Rename 3 files in Documents/invoices"),
        "the area holds something other than the words the person was shown"
    );
}
