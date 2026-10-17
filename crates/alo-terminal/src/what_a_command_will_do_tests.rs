//! The explanation is held to the three things this shape gets wrong: a mark
//! read as a verdict, an unmarked command read as a safe one, and a disclaimer
//! a caller can drop.

use super::*;

/// An explanation with nothing marked, which is the shape most answers have.
fn listing_a_folder() -> WhatACommandWillDo {
    WhatACommandWillDo::of("lists the files in this folder", &[])
}

/// An explanation that marked all three, which is the shape the drawing side
/// has to cope with and the one a person most needs to read.
fn wiping_a_disk() -> WhatACommandWillDo {
    WhatACommandWillDo::of(
        "erases every partition on the second disk and copies an image onto it from a server",
        &WorthSeeing::ALL,
    )
}

/// **The plain words come back exactly as they arrived.**
///
/// They are a model's sentence and nothing here may edit, trim or re-word them:
/// a transcript showing a changed version of what came back would be a third
/// account of the moment, after the model's and the person's.
#[test]
fn the_plain_words_come_back_as_they_arrived() {
    let said = "lists the files in this folder";
    assert_eq!(WhatACommandWillDo::of(said, &[]).in_plain_words(), said);
}

/// **Nothing marked is nothing marked**, and the type offers no way to read it
/// as *nothing is destructive*.
///
/// This is the test behind the module header's claim. It is written as a
/// property of the explanation rather than of a method, because the point is
/// that **there is no method**: a `is_safe` or `is_destructive` would make the
/// wrong reading one call away, and the only durable way to not have it is to
/// not have it.
#[test]
fn an_unmarked_explanation_claims_nothing_about_safety() {
    let quiet = listing_a_folder();
    assert_eq!(quiet.how_many_marks(), 0);
    assert_eq!(quiet.marked().count(), 0);
    // And it still carries the disclaimer, which is the only thing said about
    // what is not known.
    assert_eq!(
        quiet.and_not_a_guarantee(),
        words::AN_EXPLANATION_AND_NOT_A_GUARANTEE
    );
}

/// **Every explanation carries the disclaimer**, whatever it says and whatever
/// it marked.
///
/// Asked of every shape the type has rather than of one, because *always* is the
/// claim and a single case would not be evidence for it.
#[test]
fn every_explanation_carries_the_disclaimer() {
    let every_shape = [
        listing_a_folder(),
        wiping_a_disk(),
        WhatACommandWillDo::of("", &[]),
        WhatACommandWillDo::of("needs sudo", &[WorthSeeing::NeedsElevatedRights]),
    ];
    for explained in every_shape {
        assert_eq!(
            explained.and_not_a_guarantee(),
            words::AN_EXPLANATION_AND_NOT_A_GUARANTEE,
            "an explanation saying {:?} did not carry the disclaimer",
            explained.in_plain_words()
        );
    }
}

/// **Each mark has its own word, and no two share one.**
///
/// Three marks reading as the same sentence would be three marks a person
/// cannot tell apart, which is the same fault as one mark that says nothing.
#[test]
fn each_mark_has_a_word_of_its_own() {
    for one in WorthSeeing::ALL {
        for other in WorthSeeing::ALL {
            if one == other {
                continue;
            }
            assert_ne!(
                one.the_word().key(),
                other.the_word().key(),
                "{one:?} and {other:?} share a word"
            );
        }
    }
}

/// **The disclaimer is not one of the marks.**
///
/// It is said about the explanation; a mark is said about the command. If the
/// disclaimer's key ever appeared as a mark it would be counted, filtered and
/// sorted as though it were a claim about the command, which it is not.
#[test]
fn the_disclaimer_is_not_a_mark() {
    for mark in WorthSeeing::ALL {
        assert_ne!(
            mark.the_word().key(),
            words::AN_EXPLANATION_AND_NOT_A_GUARANTEE.key(),
            "{mark:?} is worded as the disclaimer"
        );
    }
}

/// **The same marks twice are one mark**, so an answer that repeated itself does
/// not put a line on the screen twice.
#[test]
fn a_repeated_mark_is_counted_once() {
    let twice = WhatACommandWillDo::of(
        "deletes the folder",
        &[WorthSeeing::Destructive, WorthSeeing::Destructive],
    );
    assert_eq!(twice.how_many_marks(), 1);
    assert_eq!(
        twice.marked().collect::<Vec<_>>(),
        vec![WorthSeeing::Destructive]
    );
}

/// **The order marks arrived in is not kept**, so two answers that marked the
/// same things are the same explanation.
///
/// The control for the test above: a type that kept arrival order would pass
/// that one and fail this.
#[test]
fn two_answers_that_marked_the_same_things_are_equal() {
    let one = WhatACommandWillDo::of(
        "sends it",
        &[WorthSeeing::LeavesTheMachine, WorthSeeing::Destructive],
    );
    let other = WhatACommandWillDo::of(
        "sends it",
        &[WorthSeeing::Destructive, WorthSeeing::LeavesTheMachine],
    );
    assert_eq!(one, other);
}

/// **Different plain words are different explanations**, which is the control
/// for the test above: a type that compared only its marks would pass that one.
#[test]
fn different_words_are_different_explanations() {
    assert_ne!(
        WhatACommandWillDo::of("deletes one file", &[WorthSeeing::Destructive]),
        WhatACommandWillDo::of("deletes every file", &[WorthSeeing::Destructive])
    );
}

/// **`ALL` is every variant**, so a fourth mark added to the enum and left out
/// of `ALL` is caught here rather than going unmarked everywhere `ALL` is used.
///
/// Walked by matching rather than counted, because a count asserted against a
/// literal is a test that is edited to pass.
#[test]
fn all_is_every_mark() {
    for mark in WorthSeeing::ALL {
        match mark {
            WorthSeeing::Destructive
            | WorthSeeing::NeedsElevatedRights
            | WorthSeeing::LeavesTheMachine => {}
        }
    }
    assert_eq!(WorthSeeing::ALL.len(), 3);
}

/// **An explanation with no words is still an explanation**, because an answer
/// that came back empty is a thing that happened and not a thing to hide.
///
/// What it must not do is lose the disclaimer or invent a mark: an empty
/// sentence with a mark on it would be a claim with nothing behind it.
#[test]
fn an_empty_answer_keeps_its_shape() {
    let nothing_said = WhatACommandWillDo::of("", &[]);
    assert_eq!(nothing_said.in_plain_words(), "");
    assert_eq!(nothing_said.how_many_marks(), 0);
    assert_eq!(
        nothing_said.and_not_a_guarantee(),
        words::AN_EXPLANATION_AND_NOT_A_GUARANTEE
    );
}
