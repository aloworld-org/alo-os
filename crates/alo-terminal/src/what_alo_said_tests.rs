//! What alo said, and the three things this type decides.

use super::WhatAloSaid;
use crate::why_a_command_is_waiting::WhyItIsWaiting;

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
        assert!(said.said().is_none());
    }
}

/// **A remark is one line, and it is the one that was said.**
#[test]
fn what_alo_said_is_one_line_and_the_thing_it_said() {
    let mut said = WhatAloSaid::nothing();
    said.says(WhyItIsWaiting::ForTheNetwork.the_word().key());

    assert_eq!(said.lines(), 1);
    assert_eq!(
        said.said(),
        Some(&WhyItIsWaiting::ForTheNetwork.the_word().key())
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
        said.said(),
        Some(&WhyItIsWaiting::StillComputing.the_word().key()),
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
    assert!(said.said().is_none());
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
        assert_eq!(said.said(), Some(&why.the_word().key()), "{why:?}");
        assert_eq!(said.lines(), 1, "{why:?}");
    }
}
