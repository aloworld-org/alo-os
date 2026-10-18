//! **What a model actually sends back**, including the shapes it gets wrong.
//!
//! Every reply here is one a model could plausibly produce. The ones that
//! ignore the shape matter most: a person who asked for an explanation must
//! not be left with nothing because a model answered in its own way.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, or an index past a length the test just \
              asserted, is the failure being reported"
)]

use super::*;

/// The marks on an explanation, in a settled order for comparing.
fn marks_on(what: &WhatACommandWillDo) -> Vec<WorthSeeing> {
    what.marked().collect()
}

/// **The shape the request asks for comes back whole.**
#[test]
fn a_reply_in_the_shape_gives_words_and_marks() {
    let reply = "MARKS: destructive, elevated\nThis deletes the project directory and needs \
                 administrator rights to do it.";
    let what = what_came_back(reply).expect("a reply with words is an explanation");

    assert_eq!(
        what.in_plain_words(),
        "This deletes the project directory and needs administrator rights to do it."
    );
    assert_eq!(
        marks_on(&what),
        vec![WorthSeeing::Destructive, WorthSeeing::NeedsElevatedRights]
    );
}

/// **`MARKS: none` is a mark line, and leaves no marks.**
#[test]
fn a_reply_saying_none_carries_no_marks() {
    let what = what_came_back("MARKS: none\nThis lists the files in this folder.")
        .expect("an explanation");
    assert_eq!(
        what.in_plain_words(),
        "This lists the files in this folder."
    );
    assert!(marks_on(&what).is_empty());
}

/// **A reply that ignored the shape is still an explanation.**
///
/// This is the one that matters: a model answering in its own way must not
/// cost a person the words they asked for.
#[test]
fn a_reply_with_no_marks_line_is_still_an_explanation() {
    let what = what_came_back("It removes every file under the current directory.")
        .expect("prose alone is an explanation");
    assert_eq!(
        what.in_plain_words(),
        "It removes every file under the current directory."
    );
    assert!(
        marks_on(&what).is_empty(),
        "nothing was marked, and nothing is invented"
    );
}

/// **The word *destructive* inside an explanation does not mark it.**
///
/// A model describing a command as destructive has described it. Only the
/// first line, and only when it says `MARKS:`, marks anything — otherwise
/// every careful explanation would mark itself.
#[test]
fn the_word_destructive_in_the_prose_marks_nothing() {
    let what = what_came_back("This command is destructive and removes the folder.")
        .expect("an explanation");
    assert!(marks_on(&what).is_empty());
    assert!(what.in_plain_words().contains("destructive"));
}

/// **A mark alo does not know is skipped, and the rest survive.**
#[test]
fn an_invented_mark_does_not_cost_the_real_ones() {
    let what = what_came_back(
        "MARKS: destructive, irreversible, leaves-the-machine\nIt uploads \
                               and then deletes.",
    )
    .expect("an explanation");
    assert_eq!(
        marks_on(&what),
        vec![WorthSeeing::Destructive, WorthSeeing::LeavesTheMachine]
    );
}

/// **A lower-case `marks:` is read too**, because a model will write one.
#[test]
fn the_marks_line_is_read_whatever_its_case() {
    let what = what_came_back("marks: elevated\nIt needs root.").expect("an explanation");
    assert_eq!(marks_on(&what), vec![WorthSeeing::NeedsElevatedRights]);
    assert_eq!(what.in_plain_words(), "It needs root.");
}

/// **Backticks around a mark do not hide it**, since the request writes the
/// names in backticks and a model will copy that.
#[test]
fn a_mark_in_backticks_is_still_a_mark() {
    let what = what_came_back("MARKS: `destructive`\nIt deletes things.").expect("an explanation");
    assert_eq!(marks_on(&what), vec![WorthSeeing::Destructive]);
}

/// **An empty reply is not an explanation.**
#[test]
fn an_empty_reply_explains_nothing() {
    assert_eq!(what_came_back(""), None);
    assert_eq!(what_came_back("   \n  \n"), None);
    assert_eq!(
        what_came_back("MARKS: none\n"),
        None,
        "marks with no words are not an explanation of anything"
    );
}

/// **A reply of several lines reads as one explanation.**
#[test]
fn several_lines_of_prose_become_one_explanation() {
    let what = what_came_back("MARKS: none\nIt lists files.\nIt changes nothing.")
        .expect("an explanation");
    assert_eq!(what.in_plain_words(), "It lists files. It changes nothing.");
}

/// **The command is the last thing in the request.**
///
/// So a command written to look like an instruction lands where alo's own
/// instructions have already been read. Not a guarantee against a command
/// written to mislead a model, and the cheap half of not inviting one.
#[test]
fn the_command_is_last_in_the_request() {
    let asked = the_request_for("rm -rf /tmp/x");
    assert!(
        asked.trim_end().ends_with("rm -rf /tmp/x"),
        "the command must be last, and the request ended: {:?}",
        &asked[asked.len().saturating_sub(40)..]
    );
}

/// **A command carrying a newline cannot end the request early.**
///
/// The door is one line each way, so a second line would arrive as something
/// the agent was never asked.
#[test]
fn a_command_with_a_newline_in_it_stays_one_line() {
    let asked = the_request_for("echo one\necho two\r\necho three");
    assert_eq!(
        asked.lines().count(),
        1,
        "the request is one line: {asked:?}"
    );
    assert!(asked.contains("echo one echo two  echo three"));
}

/// **Every explanation carries the disclaimer**, whatever came back.
#[test]
fn an_explanation_always_says_it_is_not_a_guarantee() {
    let what = what_came_back("It lists files.").expect("an explanation");
    assert_eq!(
        what.and_not_a_guarantee().named(),
        "terminal.explained.not-a-guarantee"
    );
}

/// A mark, as the reader hands one over.
fn said(
    what: alo_terminal::what_the_shell_says_about_the_prompt::AShellSaid,
) -> Result<
    alo_terminal::what_the_shell_says_about_the_prompt::AShellSaid,
    alo_terminal::what_the_shell_says_about_the_prompt::NotRead,
> {
    Ok(what)
}

/// **A typed line becomes one request, carrying the command.**
#[test]
fn a_typed_line_becomes_a_request_for_that_command() {
    use alo_terminal::what_the_shell_says_about_the_prompt::AShellSaid;

    let asked = the_requests_in(&[said(AShellSaid::TheyTyped("git push --force".to_owned()))]);

    assert_eq!(asked.len(), 1);
    assert!(
        asked[0].ends_with("git push --force"),
        "the request must carry the command last: {:?}",
        asked[0]
    );
}

/// **The cycle marks ask nothing.**
///
/// They say where a prompt and a command are. *Why is it waiting* is answered
/// from the kernel, which is better evidence than a shell's claim, so these are
/// read and kept rather than given a use they do not have.
#[test]
fn the_cycle_marks_ask_nothing() {
    use alo_terminal::what_the_shell_says_about_the_prompt::AShellSaid;

    let asked = the_requests_in(&[
        said(AShellSaid::APromptIsAboutToBeDrawn),
        said(AShellSaid::ThePromptHasEnded),
        said(AShellSaid::TheCommandIsRunning),
        said(AShellSaid::TheCommandFinished { status: Some(0) }),
    ]);

    assert!(
        asked.is_empty(),
        "a cycle mark is not a question: {asked:?}"
    );
}

/// **A refusal asks nothing**, rather than asking about a half-read command.
///
/// This is the one that would be dangerous the other way: explaining the first
/// half of a command is the same wrong answer as reading it off the screen.
#[test]
fn a_refusal_asks_nothing() {
    let asked = the_requests_in(&[Err(
        alo_terminal::what_the_shell_says_about_the_prompt::NotRead::TooLongToRead {
            declared: 2000,
            arrived: 1024,
        },
    )]);

    assert!(
        asked.is_empty(),
        "a command that could not be read is not asked about"
    );
}

/// **Two typed lines in one read become two requests**, in order.
#[test]
fn two_typed_lines_become_two_requests_in_order() {
    use alo_terminal::what_the_shell_says_about_the_prompt::AShellSaid;

    let asked = the_requests_in(&[
        said(AShellSaid::TheyTyped("first".to_owned())),
        said(AShellSaid::APromptIsAboutToBeDrawn),
        said(AShellSaid::TheyTyped("second".to_owned())),
    ]);

    assert_eq!(asked.len(), 2);
    assert!(asked[0].ends_with("first"));
    assert!(asked[1].ends_with("second"));
}

/// **Nothing said asks nothing**, which is nearly every frame.
#[test]
fn a_shell_with_nothing_to_say_asks_nothing() {
    assert!(the_requests_in(&[]).is_empty());
}
