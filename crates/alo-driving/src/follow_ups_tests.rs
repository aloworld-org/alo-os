//! What a follow-up is shown, and how an answer to one is scored.

use super::*;
use crate::testing::{answering, the_verbs};

/// The follow-up of this name.
fn the(named: &str) -> AFollowUp {
    THE_FOLLOW_UPS
        .into_iter()
        .find(|follow_up| follow_up.named() == named)
        .unwrap_or_else(|| panic!("no follow-up is called {named}"))
}

/// One correct answer per follow-up, written out rather than generated.
fn right_answer(named: &str) -> String {
    match named {
        "the-april-one" => answering(
            "read",
            "read_file",
            &[("file", "\"/home/anna/Invoices/april.pdf\"")],
        ),
        "the-same-again" => answering(
            "propose",
            "rename_file",
            &[
                ("file", "\"/home/anna/Invoices/scan002.pdf\""),
                ("name", "\"april.pdf\""),
            ],
        ),
        "somewhere-else" => answering(
            "propose",
            "move_file",
            &[
                ("file", "\"/home/anna/Invoices/march.pdf\""),
                ("into", "\"/home/anna/Paid\""),
            ],
        ),
        "that-window" => answering(
            "propose",
            "arrange_application",
            &[
                ("application", "\"org.alo.Writer\""),
                ("where", "\"left_half\""),
            ],
        ),
        _ => answering(
            "read",
            "find_in_folder",
            &[
                ("folder", "\"/home/anna/Archive/taxes\""),
                ("named", "\"october\""),
                ("most", "20"),
            ],
        ),
    }
}

/// **Every follow-up has a correct answer this machine would act on**, so the
/// set can be driven at all — and every name is different.
#[test]
fn every_follow_up_can_be_driven() {
    let verbs = the_verbs();
    for follow_up in THE_FOLLOW_UPS {
        assert_eq!(
            follow_up.scored(&verbs, &right_answer(follow_up.named())),
            FollowedUp::Drove,
            "{}",
            follow_up.named()
        );
    }
    let mut names: Vec<&str> = THE_FOLLOW_UPS.iter().map(AFollowUp::named).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), THE_FOLLOW_UPS.len());
}

/// **The right verb about the wrong thing is not driving** — the failure only a
/// follow-up can show: reading march.pdf when the conversation meant April.
#[test]
fn the_right_verb_about_the_wrong_thing_is_scored_as_the_wrong_one() {
    let wrong = answering(
        "read",
        "read_file",
        &[("file", "\"/home/anna/Invoices/march.pdf\"")],
    );
    assert_eq!(
        the("the-april-one").scored(&the_verbs(), &wrong),
        FollowedUp::TheWrongOne
    );
    assert!(matches!(
        the("the-april-one").scored(&the_verbs(), "the April one is april.pdf"),
        FollowedUp::DidNotDrive(_)
    ));
}

/// **A follow-up is shown the graded words, then the conversation, then the
/// follow-up last**, built by the function a real turn uses.
#[test]
fn a_follow_up_is_shown_the_turns_words_and_its_conversation() {
    let follow_up = the("the-april-one");
    let shown = follow_up.prompt_under(Instructions::SHOWN_TO_A_TURN, &the_verbs());
    assert!(shown.starts_with(Instructions::SHOWN_TO_A_TURN.text()));
    assert!(shown.contains(&follow_up.the_conversation_so_far().told()));
    assert!(shown.ends_with("The request: and read the April one"));
}
