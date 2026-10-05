//! **The two answers on the approval surface are read aloud in the words they
//! are drawn with.**
//!
//! EN 301 549 clause 11.2.5.3 — *a control's name for a program must contain
//! the words a person sees on it* — on the surface ADR 0001 makes the most
//! consequential in the product: what a person approves is the sentence the
//! turn wrote, and the two answers follow it.
//!
//! # What was wrong, and why nothing caught it
//!
//! Until 2026-10-05 there were **two vocabularies for two buttons**.
//! `alo-approving` drew `approving.no` and `approving.approve` — *No* and
//! *Approve*, and in German *Nein* and *Genehmigen*. This crate announced
//! `access.say-no` and `access.approve-it`, its own keys, translated
//! independently into the same 24 languages. **Nothing compared them**, in
//! either direction, and neither crate's tests could: each was correct about
//! its own vocabulary.
//!
//! That is the fault
//! [ADR 0089](../../../docs/decisions/0089-what-a-control-is-called.md) found
//! on the window strip, arriving on a different surface by the same route —
//! two lists, each right about itself, with no test standing between them.
//! On the strip the lists disagreed about *which controls existed*; here they
//! agree about that and disagree about *what the controls are called*, which
//! is quieter and reaches a person in exactly the same way.
//!
//! # Why this test can no longer fail, and is kept anyway
//!
//! The fix is not an assertion that two strings match — it is that there is
//! **one string**. `Surface::Approval` names its buttons with
//! `alo_approving::words::{NO, APPROVE}`, the same constants
//! `Asked::no_said` and `Asked::approve_said` hand to the screen, so a
//! translator moving one moves both and a containment check is trivially true
//! in every language.
//!
//! This file holds that *property* rather than the strings: it asserts the
//! announced name **is** the drawn one, by key and by text, so a future change
//! that reintroduces a second vocabulary fails here rather than on somebody's
//! machine. A test that cannot fail today is worth keeping when what it
//! guards is an invariant somebody could remove in one line.

use alo_access::{Role, Surface};

/// What the approval surface reads aloud.
fn the_approval_surface() -> Vec<alo_access::Control> {
    Surface::Approval.read_aloud()
}

/// **Both answers are announced, and they are buttons.**
///
/// The list before the words: a surface that announced one answer, or
/// announced them as labels, would pass a words-only check while leaving a
/// person unable to find or press one of them.
#[test]
fn both_answers_are_announced_as_buttons() {
    let buttons = the_approval_surface()
        .into_iter()
        .filter(|control| control.role == Role::Button)
        .count();

    assert_eq!(
        buttons, 2,
        "the approval surface announces {buttons} buttons and ADR 0001 gives \
         it two answers"
    );
}

/// **Each answer is named by the word it is drawn with, by key.**
///
/// The key rather than the text, because the text is whatever language is
/// loaded and the key is what makes the two one string. A second vocabulary
/// reintroduced here would have a different key even where the English
/// happened to agree, which is the case a text comparison would miss.
#[test]
fn each_answer_is_named_by_the_word_it_is_drawn_with() {
    let names: Vec<String> = the_approval_surface()
        .iter()
        .filter(|control| control.role == Role::Button)
        .map(|control| control.name.key().as_str().to_owned())
        .collect();

    assert_eq!(
        names,
        vec![
            alo_approving::words::NO.key().as_str().to_owned(),
            alo_approving::words::APPROVE.key().as_str().to_owned(),
        ],
        "the approval answers are announced under keys of this crate's own \
         rather than the ones `alo-approving` draws"
    );
}

/// **No answer is announced in a word this crate declares.**
///
/// The other direction, and the one that would have caught the original
/// fault: `access.say-no` and `access.approve-it` were this crate's, said
/// nothing wrong, and were simply not the words on the screen. Any
/// `access.`-prefixed key on a button of this surface is that fault coming
/// back.
#[test]
fn no_answer_is_announced_in_this_crates_own_words() {
    for control in the_approval_surface() {
        if control.role != Role::Button {
            continue;
        }
        let named = control.name.key();
        let key = named.as_str();
        assert!(
            !key.starts_with("access."),
            "the answer announced as `{key}` is named in this crate's \
             vocabulary rather than in the words it is drawn with"
        );
    }
}
