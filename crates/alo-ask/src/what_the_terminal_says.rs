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
        // **As the road without the switch prints it**: the reply, whole.
        WhatTheTerminalDid::NothingWaited | WhatTheTerminalDid::NotTheChangeWaiting => {
            WhatTheTerminalSays {
                lines: vec![the_reply],
                worked: true,
            }
        }
        WhatTheTerminalDid::SwitchedOffFirst => WhatTheTerminalSays {
            lines: vec![
                the_reply,
                say(alo_capability::words::NOT_DONE_IT_WAS_SWITCHED_OFF),
            ],
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

    /// **Switched off first: the waiting change is shown, and said not done.**
    #[test]
    fn switched_off_first_says_it_was_not_done() {
        let reply = a_change_is_waiting(3, RENAME);
        let said =
            what_the_terminal_says(&reply, &WhatTheTerminalDid::SwitchedOffFirst, &in_english());
        assert_eq!(said.lines[0], reply);
        assert!(said.lines[1].starts_with("Not done"), "{said:?}");
        assert!(!said.worked);
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
