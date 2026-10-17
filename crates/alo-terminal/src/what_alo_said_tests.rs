//! What alo said, and the things this type decides.

use super::{AloSLine, AloSRemark, WhatAloSaid};
use crate::what_a_command_will_do::{WhatACommandWillDo, WorthSeeing};
use crate::why_a_command_is_waiting::WhyItIsWaiting;

/// The remark's key, for a test that does not care which variant carried it.
///
/// Answers [`None`] for an explanation, which is the honest answer: an
/// explanation is not one of our words and has no single key.
fn our_key(said: &WhatAloSaid) -> Option<alo_strings::Key> {
    match said.remark() {
        Some(AloSRemark::InOurWords(key)) => Some(key.clone()),
        Some(AloSRemark::AnExplanation(_)) | None => None,
    }
}

/// **A terminal nobody has asked carries no alo line.**
///
/// The case every drawing hits first, and the one a `Vec` would have got right
/// by accident. Asserted on both the default and the named constructor, because
/// a `Default` that disagreed with `nothing()` would be two answers to one
/// question.
#[test]
fn nothing_said_is_nothing_drawn() {
    for said in [WhatAloSaid::default(), WhatAloSaid::nothing()] {
        assert_eq!(said.lines(), 0);
        assert!(said.remark().is_none());
    }
}

/// **A remark is one line, and it is the one that was said.**
#[test]
fn what_alo_said_is_one_line_and_the_thing_it_said() {
    let mut said = WhatAloSaid::nothing();
    said.says(WhyItIsWaiting::ForTheNetwork.the_word().key());

    assert_eq!(said.lines(), 1);
    assert_eq!(
        our_key(&said),
        Some(WhyItIsWaiting::ForTheNetwork.the_word().key())
    );
}

/// **Asking twice replaces, and does not grow.**
///
/// The decision this type exists to make. A `Vec` would have appended, and a
/// transcript that grew by a line every time somebody asked would push the
/// program's own output off the screen.
#[test]
fn asking_again_replaces_rather_than_appending() {
    let mut said = WhatAloSaid::nothing();
    said.says(WhyItIsWaiting::ForTheKeyboard.the_word().key());
    said.says(WhyItIsWaiting::StillComputing.the_word().key());

    assert_eq!(said.lines(), 1, "two asks left two lines");
    assert_eq!(
        our_key(&said),
        Some(WhyItIsWaiting::StillComputing.the_word().key()),
        "the second ask did not replace the first"
    );
}

/// **And it can be emptied again**, for when the command has finished.
///
/// Without this a reply about a command that has gone would sit on screen
/// looking current, which is worse than no reply: a person would act on it.
#[test]
fn nothing_to_say_clears_what_was_said() {
    let mut said = WhatAloSaid::nothing();
    said.says(WhyItIsWaiting::ForAnotherProcess.the_word().key());
    said.nothing_to_say();

    assert_eq!(said.lines(), 0);
    assert!(said.remark().is_none());
}

/// **Every answer can be said**, so none of the five is unsayable.
///
/// Walks `WhyItIsWaiting::ALL` rather than naming one, because a sixth answer
/// added later should arrive here without anybody remembering to extend a list.
#[test]
fn every_answer_a_person_can_get_is_one_alo_can_say() {
    for why in WhyItIsWaiting::ALL {
        let mut said = WhatAloSaid::nothing();
        said.says(why.the_word().key());
        assert_eq!(our_key(&said), Some(why.the_word().key()), "{why:?}");
        assert_eq!(said.lines(), 1, "{why:?}");
    }
}

/// **An explanation is its words, each mark, and the disclaimer** — in that
/// order, because the disclaimer is about everything above it.
#[test]
fn an_explanation_is_its_words_then_its_marks_then_the_disclaimer() {
    let mut said = WhatAloSaid::nothing();
    said.explains(WhatACommandWillDo::of(
        "erases the disk",
        &[WorthSeeing::Destructive, WorthSeeing::NeedsElevatedRights],
    ));

    let lines = said.the_lines();
    assert_eq!(lines.len(), 4, "{lines:?}");
    assert_eq!(
        lines.first(),
        Some(&AloSLine::AsTheModelSaidIt("erases the disk".to_owned())),
        "the plain words are not the first line"
    );
    assert_eq!(
        lines.last(),
        Some(&AloSLine::InOurWords(
            crate::words::AN_EXPLANATION_AND_NOT_A_GUARANTEE.key()
        )),
        "the disclaimer is not the last line"
    );
}

/// **The count and the lines agree, for every shape this type has.**
///
/// `lines` is counted and `the_lines` is built, which is two answers about one
/// subject — and the only thing that makes that safe is this. A gutter label
/// beside the wrong row is what disagreement looks like on a screen.
#[test]
fn the_count_and_the_lines_never_disagree() {
    let mut shapes = vec![WhatAloSaid::nothing(), WhatAloSaid::default()];
    for why in WhyItIsWaiting::ALL {
        let mut said = WhatAloSaid::nothing();
        said.says(why.the_word().key());
        shapes.push(said);
    }
    for marks in [
        Vec::new(),
        vec![WorthSeeing::Destructive],
        vec![WorthSeeing::Destructive, WorthSeeing::LeavesTheMachine],
        WorthSeeing::ALL.to_vec(),
    ] {
        let mut said = WhatAloSaid::nothing();
        said.explains(WhatACommandWillDo::of("does a thing", &marks));
        shapes.push(said);
    }

    for said in shapes {
        assert_eq!(
            said.lines(),
            said.the_lines().len(),
            "the count and the lines disagree for {said:?}"
        );
    }
}

/// **An explanation that marked nothing is still two lines**: its words and the
/// disclaimer.
///
/// The case where a caller might reasonably expect one, and the one where
/// dropping the disclaimer would be least noticed — an explanation with nothing
/// marked is exactly the one a person is most likely to act on.
#[test]
fn an_explanation_with_no_marks_still_says_it_is_not_a_guarantee() {
    let mut said = WhatAloSaid::nothing();
    said.explains(WhatACommandWillDo::of("lists the folder", &[]));

    assert_eq!(said.lines(), 2);
    assert_eq!(
        said.the_lines().last(),
        Some(&AloSLine::InOurWords(
            crate::words::AN_EXPLANATION_AND_NOT_A_GUARANTEE.key()
        ))
    );
}

/// **A model's sentence never arrives as one of our words.**
///
/// The whole reason [`AloSLine`] has two variants. A drawing path handed
/// `InOurWords` will look the text up in the vocabulary and find nothing, or
/// worse find something — and either way it would be claiming authorship of a
/// stranger's sentence.
#[test]
fn a_models_sentence_is_never_labelled_as_ours() {
    let mut said = WhatAloSaid::nothing();
    said.explains(WhatACommandWillDo::of("terminal.who.alo", &[]));

    let lines = said.the_lines();
    assert_eq!(
        lines.first(),
        Some(&AloSLine::AsTheModelSaidIt("terminal.who.alo".to_owned())),
        "a model's words that look like a key were carried as a key"
    );
}

/// **Asking one question after the other replaces**, across the two kinds.
///
/// The single slot holds one remark and not one of each: somebody who asked what
/// a command would do and then asked why it is waiting has moved on, and the
/// first answer is about something else now.
#[test]
fn an_explanation_and_a_waiting_answer_replace_one_another() {
    let mut said = WhatAloSaid::nothing();
    said.explains(WhatACommandWillDo::of("runs the build", &[]));
    said.says(WhyItIsWaiting::StillComputing.the_word().key());
    assert_eq!(said.lines(), 1);
    assert_eq!(
        our_key(&said),
        Some(WhyItIsWaiting::StillComputing.the_word().key())
    );

    said.explains(WhatACommandWillDo::of("runs the build", &[]));
    assert_eq!(said.lines(), 2, "the waiting answer was not replaced");
    assert_eq!(our_key(&said), None, "an explanation answered with a key");
}

/// **And an explanation can be cleared too**, the same way.
#[test]
fn nothing_to_say_clears_an_explanation() {
    let mut said = WhatAloSaid::nothing();
    said.explains(WhatACommandWillDo::of("deletes it", &WorthSeeing::ALL));
    said.nothing_to_say();

    assert_eq!(said.lines(), 0);
    assert!(said.remark().is_none());
    assert!(said.the_lines().is_empty());
}
