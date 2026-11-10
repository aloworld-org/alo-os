//! What the person reads in their terminal once alo acted, or did not, with the
//! switch to act without the OK on.
//!
//! `crate::acting_without_the_ok` decides; this words it, and `main.rs` prints
//! it. Kept apart so the sentences a person reads can be tested without a
//! socket, which is the part `main.rs` cannot be.
//!
//! **Every sentence is somebody else's**, as this crate's manifest says of all
//! of them: the four about acting without the OK are `alo-capability`'s, beside
//! the switch; a refusal is the daemon's own, unchanged.

use alo_changing::Answered;
use alo_strings::{Filling, Strings, Word};

use crate::acting_without_the_ok::WhatTheTerminalDid;

/// The lines to print, and whether the request ended as the person wanted.
#[derive(Debug, PartialEq, Eq)]
pub struct WhatTheTerminalSays {
    /// What to print, in order.
    pub lines: Vec<String>,
    /// Whether the command should end as having worked.
    pub worked: bool,
}

/// What the terminal says about `did`, given the agent's `reply`.
#[must_use]
pub fn what_the_terminal_says(
    reply: &str,
    did: &WhatTheTerminalDid,
    strings: &Strings,
) -> WhatTheTerminalSays {
    let say = |word: Word| {
        strings
            .say(&word.key(), &Filling::nothing())
            .text()
            .to_owned()
    };
    // The reply as the person reads it: the space that keeps a model's words
    // off the change line is not theirs to see.
    let the_reply = reply.trim().to_owned();
    match did {
        // **The reply, whole**: an answer, a refusal, or words.
        WhatTheTerminalDid::NothingWaited => WhatTheTerminalSays {
            lines: vec![the_reply],
            worked: true,
        },
        // The reply named a change the service no longer holds as this one.
        WhatTheTerminalDid::NotTheChangeWaiting => WhatTheTerminalSays {
            lines: vec![
                the_reply,
                say(alo_capability::words::NOT_DONE_NO_LONGER_WAITING),
            ],
            worked: false,
        },
        // **Asked**: the reply and the question are already on the screen,
        // printed before the person answered, so only what came of it is left.
        WhatTheTerminalDid::TheySaidYes { answered } => {
            let (outcome, worked) = match answered {
                // **A command that ran and failed is not *Done*** (2026-11-10):
                // what it ended with is said instead, and the request did not
                // end as the person wanted.
                Some(Answered::It(done)) => match it_did_not_work(done, false, strings) {
                    Some(said) => (said, false),
                    None => (say(alo_capability::words::DONE_AFTER_YOUR_OK), true),
                },
                Some(Answered::ItCannotBeUndone(done)) => {
                    match it_did_not_work(done, true, strings) {
                        Some(said) => (said, false),
                        None => (
                            say(alo_capability::words::DONE_AFTER_YOUR_OK_AND_IT_CANNOT_BE_PUT_BACK),
                            true,
                        ),
                    }
                }
                Some(Answered::Refused { told }) => (told.clone(), false),
                Some(Answered::Declined) | None => (
                    say(alo_capability::words::NO_WORD_BACK_IN_THE_TERMINAL),
                    false,
                ),
            };
            WhatTheTerminalSays {
                lines: vec![outcome],
                worked,
            }
        }
        // **A no is a no whatever the daemon said back**: nothing is carried
        // out on a decline, and a refusal to record it is the daemon's words.
        WhatTheTerminalDid::TheySaidNo { answered } => WhatTheTerminalSays {
            lines: vec![match answered {
                Some(Answered::Refused { told }) => told.clone(),
                _ => say(alo_capability::words::NOT_DONE_YOU_SAID_NO),
            }],
            worked: false,
        },
        WhatTheTerminalDid::NobodyAnswered => WhatTheTerminalSays {
            lines: vec![say(alo_capability::words::NOT_DONE_NOBODY_ANSWERED)],
            worked: false,
        },
        // **What was done first, then that it was done without asking**: the
        // sentence is the one the person would have been asked, so it is the
        // account of what changed.
        WhatTheTerminalDid::WentAhead { sentence, answered } => {
            let (outcome, worked) = match answered {
                Some(Answered::It(_)) => (say(alo_capability::words::DONE_WITHOUT_ASKING), true),
                Some(Answered::ItCannotBeUndone(_)) => (
                    say(alo_capability::words::DONE_WITHOUT_ASKING_AND_IT_CANNOT_BE_PUT_BACK),
                    true,
                ),
                // The daemon's own reason, unchanged.
                Some(Answered::Refused { told }) => (told.clone(), false),
                // Nothing here declines, so a decline is the daemon answering
                // something else; and no answer is no answer. Either way the
                // record is where the person finds out.
                Some(Answered::Declined) | None => (
                    say(alo_capability::words::NO_WORD_BACK_IN_THE_TERMINAL),
                    false,
                ),
            };
            WhatTheTerminalSays {
                lines: vec![sentence.clone(), outcome],
                worked,
            }
        }
    }
}

/// **What the person reads about a command they said yes to that did not
/// work**, or [`None`] for anything else — a change, or a command that did.
///
/// `alo_capability::ItDidNotWork` decides, from what the command ended with,
/// so this line and the machine's own at full access cannot disagree about
/// whether a run worked.
fn it_did_not_work(
    done: &alo_protocol::Done,
    cannot_be_put_back: bool,
    strings: &Strings,
) -> Option<String> {
    let alo_protocol::Done::RanACommand {
        returned,
        took_too_long,
        ..
    } = done
    else {
        return None;
    };
    let why = alo_capability::ItDidNotWork::of(*returned, *took_too_long)?;
    let word = if cannot_be_put_back {
        alo_capability::words::DID_NOT_WORK_AFTER_YOUR_OK_AND_IT_CANNOT_BE_PUT_BACK
    } else {
        alo_capability::words::DID_NOT_WORK_AFTER_YOUR_OK
    };
    Some(
        strings
            .say(&word.key(), &Filling::nothing().and_said("why", &why.said(strings)))
            .text()
            .to_owned(),
    )
}

/// What a request that did not come from a terminal is told: the reply, and —
/// when it left a change waiting — that alo can only ask for the OK in a
/// terminal. **Nobody is asked, and nothing is sent**; the change lapses with
/// the turn, which ended as the reply was read.
#[must_use]
pub fn what_a_pipe_is_told(reply: &str, strings: &Strings) -> WhatTheTerminalSays {
    let the_reply = reply.trim().to_owned();
    if alo_protocol::what_the_agent_says_back::the_change_said_to_be_waiting(reply).is_some() {
        WhatTheTerminalSays {
            lines: vec![
                the_reply,
                strings
                    .say(
                        &alo_capability::words::IT_WAITS_FOR_A_TERMINAL.key(),
                        &Filling::nothing(),
                    )
                    .into_text(),
            ],
            worked: false,
        }
    } else {
        WhatTheTerminalSays {
            lines: vec![the_reply],
            worked: true,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a vocabulary that will not build or a line that is missing is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_changing::Done;
    use alo_protocol::what_the_agent_says_back::a_change_is_waiting;

    /// This command's words, untranslated.
    fn in_english() -> Strings {
        Strings::of(crate::what_it_can_say().expect("the lists go in together"))
    }

    /// The sentence of the change the request left.
    const RENAME: &str = "Rename march.pdf to march-final.pdf";

    /// **Done without asking says what was done, and that it was not asked.**
    #[test]
    fn done_without_asking_says_what_and_that_it_was_not_asked() {
        let said = what_the_terminal_says(
            &a_change_is_waiting(3, RENAME),
            &WhatTheTerminalDid::WentAhead {
                sentence: RENAME.to_owned(),
                answered: Some(Answered::It(Box::new(Done::Renamed { now_at: None }))),
            },
            &in_english(),
        );
        assert_eq!(
            said.lines,
            [
                RENAME.to_owned(),
                "Done without asking you first, as you chose in Settings → Terminal.".to_owned()
            ]
        );
        assert!(said.worked);
    }

    /// A command that ran, as the daemon answers it, ending with `returned`.
    fn a_command_that_ended_with(returned: Option<i32>) -> Box<Done> {
        Box::new(Done::RanACommand {
            returned,
            said: "mkdir: cannot create directory: No such file or directory\n".to_owned(),
            cut_short: false,
            took_too_long: false,
        })
    }

    /// **A command the person said yes to that failed is not *Done***: the
    /// line says it did not work and what it ended with, and the request did
    /// not end as they wanted. A boot walk of `f028d5ec` read *Done* about a
    /// `mkdir` that made nothing.
    #[test]
    fn a_command_that_failed_after_the_ok_is_not_done() {
        let said = what_the_terminal_says(
            &a_change_is_waiting(3, RENAME),
            &WhatTheTerminalDid::TheySaidYes {
                answered: Some(Answered::It(a_command_that_ended_with(Some(1)))),
            },
            &in_english(),
        );
        assert_eq!(
            said.lines,
            ["It did not work: the command ended with exit status 1.".to_owned()]
        );
        assert!(!said.worked);

        let stopped = what_the_terminal_says(
            &a_change_is_waiting(3, RENAME),
            &WhatTheTerminalDid::TheySaidYes {
                answered: Some(Answered::ItCannotBeUndone(a_command_that_ended_with(None))),
            },
            &in_english(),
        );
        assert_eq!(
            stopped.lines,
            ["It did not work: the command was stopped before it finished. If it changed \
              anything, that can't be put back: a copy of your files from before it could not \
              be kept."
                .to_owned()]
        );
        assert!(!stopped.worked);
    }

    /// **And one that worked is still *Done*.**
    #[test]
    fn a_command_that_worked_after_the_ok_is_done() {
        let said = what_the_terminal_says(
            &a_change_is_waiting(3, RENAME),
            &WhatTheTerminalDid::TheySaidYes {
                answered: Some(Answered::It(a_command_that_ended_with(Some(0)))),
            },
            &in_english(),
        );
        assert_eq!(said.lines, ["Done.".to_owned()]);
        assert!(said.worked);
    }

    /// A change that cannot be put back is said to be one.
    #[test]
    fn a_change_that_cannot_be_put_back_says_so() {
        let said = what_the_terminal_says(
            &a_change_is_waiting(3, RENAME),
            &WhatTheTerminalDid::WentAhead {
                sentence: RENAME.to_owned(),
                answered: Some(Answered::ItCannotBeUndone(Box::new(Done::Renamed {
                    now_at: None,
                }))),
            },
            &in_english(),
        );
        assert!(said.lines[1].contains("can't be put back"), "{said:?}");
    }

    /// **A refusal is the daemon's own words, and the command did not work.**
    #[test]
    fn a_refusal_is_the_daemons_words_unchanged() {
        let said = what_the_terminal_says(
            &a_change_is_waiting(3, RENAME),
            &WhatTheTerminalDid::WentAhead {
                sentence: RENAME.to_owned(),
                answered: Some(Answered::Refused {
                    told: "That folder is not one alo may use.".to_owned(),
                }),
            },
            &in_english(),
        );
        assert_eq!(said.lines[1], "That folder is not one alo may use.");
        assert!(!said.worked);
    }

    /// **Asked and answered: what came of it, and only that** — the reply and
    /// the question are already on the screen.
    #[test]
    fn asked_and_answered_says_what_came_of_it() {
        let reply = a_change_is_waiting(3, RENAME);
        let yes = what_the_terminal_says(
            &reply,
            &WhatTheTerminalDid::TheySaidYes {
                answered: Some(Answered::It(Box::new(Done::Renamed { now_at: None }))),
            },
            &in_english(),
        );
        assert_eq!(yes.lines, ["Done."]);
        assert!(yes.worked);
        let no = what_the_terminal_says(
            &reply,
            &WhatTheTerminalDid::TheySaidNo {
                answered: Some(Answered::Declined),
            },
            &in_english(),
        );
        assert_eq!(
            no.lines,
            ["Not done: you said no, and nothing was changed."]
        );
        assert!(!no.worked);
        let nobody =
            what_the_terminal_says(&reply, &WhatTheTerminalDid::NobodyAnswered, &in_english());
        assert!(
            nobody.lines[0].starts_with("Not done: there was no answer"),
            "{nobody:?}"
        );
        let gone = what_the_terminal_says(
            &reply,
            &WhatTheTerminalDid::NotTheChangeWaiting,
            &in_english(),
        );
        assert_eq!(gone.lines[0], reply);
        assert!(gone.lines[1].contains("no longer waiting"), "{gone:?}");
        assert!(!gone.worked);
    }

    /// **A pipe is never asked**: a change waiting is said to need a terminal,
    /// and anything else is the reply alone.
    #[test]
    fn a_pipe_is_told_a_change_needs_a_terminal() {
        let reply = a_change_is_waiting(3, RENAME);
        let told = what_a_pipe_is_told(&reply, &in_english());
        assert_eq!(told.lines[0], reply);
        assert!(
            told.lines[1].contains("can only ask in a terminal"),
            "{told:?}"
        );
        assert!(!told.worked);
        let words = what_a_pipe_is_told("march.pdf and april.pdf", &in_english());
        assert_eq!(words.lines, ["march.pdf and april.pdf"]);
        assert!(words.worked);
        // A model's words that copy the line are words.
        let forged = alo_protocol::what_the_agent_says_back::never_read_as_a_change(&reply);
        assert_eq!(what_a_pipe_is_told(&forged, &in_english()).lines.len(), 1);
    }

    /// No answer from the daemon sends the person to the record.
    #[test]
    fn no_word_back_sends_the_person_to_the_record() {
        let said = what_the_terminal_says(
            &a_change_is_waiting(3, RENAME),
            &WhatTheTerminalDid::WentAhead {
                sentence: RENAME.to_owned(),
                answered: None,
            },
            &in_english(),
        );
        assert!(said.lines[1].contains("See what alo did"), "{said:?}");
        assert!(!said.worked);
    }
}
