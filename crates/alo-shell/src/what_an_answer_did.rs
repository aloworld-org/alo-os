//! What a person is told in the conversation once they answered a waiting
//! change.
//!
//! **Nothing was said at all until 2026-10-26.** Walking a booted machine, a
//! person pressed *Allow once*, the daemon refused the change, and the screen
//! showed only that the answers had gone: the reply was read and dropped, and
//! the task kept saying *Waiting for your permission*. A person who cannot see
//! whether their approval worked has not been given a choice, only a button.
//!
//! So every answer ends in one line from alo, and the line says which of five
//! things happened. A refusal is the daemon's own words, unchanged, because a
//! reason re-worded here would be one nobody gave.

use alo_changing::Answered;
use alo_strings::{Filling, Strings};

use crate::the_cards_words::{
    IT_WAS_DONE, IT_WAS_DONE_AND_CANNOT_BE_UNDONE, NO_WORD_BACK, YOU_SAID_NO,
};

/// The line alo says once the daemon has answered, or failed to.
///
/// **A command that ran and did not work is never *Done.*** (2026-11-13, the
/// rule the terminal has kept since 2026-11-10): it says *It did not work:*
/// and why, as `alo_capability::ItDidNotWork` decides — and for a command
/// that failed because nobody is signed in to GitHub, why says to sign in and
/// then **ask alo again**, because alo does not run it a second time by
/// itself.
pub(crate) fn what_an_answer_did(answered: Option<&Answered>, strings: &Strings) -> String {
    // A command that did not work is said as that first: whatever it sent,
    // it is not *done*.
    if let Some(said) = answered.and_then(|answered| it_did_not_work(answered, strings)) {
        return said;
    }
    // **A publication is never *Done.***: it is on the internet, and Put back
    // can't recall it — the owner's decision of 2026-11-13, in
    // `alo_publishing`'s one line so every surface says it alike.
    if let Some(Answered::It(done) | Answered::ItCannotBeUndone(done)) = answered
        && let alo_changing::Done::Published {
            folder,
            address,
            nothing_new,
        } = done.as_ref()
    {
        return alo_publishing::published_after_your_ok(
            alo_publishing::WhatWasPublished {
                folder,
                address,
                nothing_new: *nothing_new,
            },
            strings,
        )
        .text()
        .to_owned();
    }
    // **Since 2026-11-13: never plain *Done.* for a command that sent
    // something off this computer**, beside a copy Put back could return —
    // ADR 0110 §2, *it does not recall a push*. Where nothing counted what it
    // sent, the line says so (`alo_capability::WhatItSent`).
    if let Some(Answered::It(done)) = answered
        && let Some(what) = done.what_a_command_sent()
    {
        return strings
            .say(&what.done_after_your_ok().key(), &Filling::nothing())
            .text()
            .to_owned();
    }
    let word = match answered {
        Some(Answered::Refused { told }) => return told.clone(),
        Some(Answered::It(_)) => IT_WAS_DONE,
        // **Since 2026-10-28.** The daemon ran the change and could not keep a
        // copy from before it; *Done.* alone would read as a change `alo-back`
        // can take back (ADR 0045's third term).
        Some(Answered::ItCannotBeUndone(_)) => IT_WAS_DONE_AND_CANNOT_BE_UNDONE,
        Some(Answered::Declined) => YOU_SAID_NO,
        None => NO_WORD_BACK,
    };
    strings
        .say(&word.key(), &Filling::nothing())
        .text()
        .to_owned()
}

/// What the person reads about a command they allowed that ran and did not
/// work, or [`None`] for anything else — a change, or a command that worked.
///
/// The terminal's own sentence (`alo-ask`'s `what_the_terminal_says`), so the
/// card and the terminal cannot disagree about whether a run worked.
fn it_did_not_work(answered: &Answered, strings: &Strings) -> Option<String> {
    let alo_changing::Done::RanACommand {
        returned,
        took_too_long,
        said,
        ..
    } = answered.did()?
    else {
        return None;
    };
    let why = alo_capability::ItDidNotWork::of_what_it_said(*returned, *took_too_long, said)?;
    let word = if answered.cannot_be_undone() {
        alo_capability::words::DID_NOT_WORK_AFTER_YOUR_OK_AND_IT_CANNOT_BE_PUT_BACK
    } else {
        alo_capability::words::DID_NOT_WORK_AFTER_YOUR_OK
    };
    Some(
        strings
            .say(
                &word.key(),
                &Filling::nothing().and_said("why", &why.said(strings)),
            )
            .text()
            .to_owned(),
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_changing::Done;
    use alo_strings::Vocabulary;

    /// This surface's words, untranslated.
    fn in_english() -> Strings {
        let mut vocabulary = Vocabulary::empty();
        crate::the_cards_words::declare_the_cards_words(&mut vocabulary).unwrap();
        Strings::of(vocabulary)
    }

    /// **Each of the four is told apart**, and none of them reads as another —
    /// above all, *nobody answered* never reads as *done* or as *you said no*.
    #[test]
    fn each_outcome_says_something_different() {
        let strings = in_english();
        let done = what_an_answer_did(
            Some(&Answered::It(Box::new(Done::Renamed { now_at: None }))),
            &strings,
        );
        let declined = what_an_answer_did(Some(&Answered::Declined), &strings);
        let silent = what_an_answer_did(None, &strings);

        assert_eq!(done, "Done.");
        assert_eq!(declined, "You said no, so nothing was changed.");
        assert!(silent.contains("did not say"), "{silent}");
        assert_ne!(silent, done);
        assert_ne!(silent, declined);
    }

    /// **A change that cannot be undone says so, and plainly.** It is told
    /// apart from *Done.*, it says it was done, and it never softens into
    /// *may not*.
    #[test]
    fn a_change_that_cannot_be_undone_says_so() {
        let strings = in_english();
        let done = what_an_answer_did(
            Some(&Answered::It(Box::new(Done::Renamed { now_at: None }))),
            &strings,
        );
        let not_kept = what_an_answer_did(
            Some(&Answered::ItCannotBeUndone(Box::new(Done::Renamed {
                now_at: None,
            }))),
            &strings,
        );

        assert_eq!(
            not_kept,
            "Done. This change cannot be undone: alo could not keep a copy of your files from \
             before it."
        );
        assert_ne!(not_kept, done);
        assert!(!not_kept.contains("may"), "{not_kept}");
        assert!(!not_kept.contains("might"), "{not_kept}");
    }

    /// **A command that sent something off this computer is never plain
    /// *Done.***, and nor is one nothing counted; one counted as sending
    /// nothing still is (ADR 0110 §2).
    #[test]
    fn a_command_that_sent_something_is_not_told_as_wholly_undoable() {
        let mut vocabulary = Vocabulary::empty();
        crate::the_cards_words::declare_the_cards_words(&mut vocabulary).unwrap();
        alo_capability::words::declare_into(&mut vocabulary).unwrap();
        let strings = Strings::of(vocabulary);
        let ran = |sent: Option<u64>| {
            Answered::It(Box::new(Done::RanACommand {
                returned: Some(0),
                said: String::new(),
                cut_short: false,
                took_too_long: false,
                sent_bytes: sent,
            }))
        };
        assert_eq!(what_an_answer_did(Some(&ran(Some(0))), &strings), "Done.");
        let sent = what_an_answer_did(Some(&ran(Some(10_000))), &strings);
        assert!(
            sent.starts_with("Done. Put back can return your files on this computer")
                && sent.contains("sent something off this computer"),
            "{sent}"
        );
        let uncounted = what_an_answer_did(Some(&ran(None)), &strings);
        assert!(
            uncounted.contains("could not count whether it sent anything"),
            "{uncounted}"
        );
    }

    /// **A publication is never *Done.***, kept or not: the person reads that
    /// it is on the internet and Put back can't recall it (the owner's
    /// decision of 2026-11-13).
    #[test]
    fn a_publication_says_it_is_on_the_internet_and_cannot_be_recalled() {
        let mut vocabulary = Vocabulary::empty();
        crate::the_cards_words::declare_the_cards_words(&mut vocabulary).unwrap();
        alo_publishing::words::declare_into(&mut vocabulary).unwrap();
        let strings = Strings::of(vocabulary);
        let published = || {
            Box::new(Done::Published {
                folder: "/home/anna/site".to_owned(),
                address: "https://anna.github.io/".to_owned(),
                nothing_new: false,
            })
        };
        for answered in [
            Answered::It(published()),
            Answered::ItCannotBeUndone(published()),
        ] {
            let said = what_an_answer_did(Some(&answered), &strings);
            assert_eq!(
                said,
                "Published: it is on the internet now, as your GitHub Pages site at \
                 https://anna.github.io/. Put back can't recall it."
            );
        }
    }

    /// **A refusal is the daemon's own words, unchanged** — the walk's own
    /// refusal, as the record held it.
    #[test]
    fn a_refusal_is_told_in_the_daemons_words() {
        let told = "/home/alo/Pictures really leads to /var/home/alo/Pictures";
        let said = what_an_answer_did(
            Some(&Answered::Refused {
                told: told.to_owned(),
            }),
            &in_english(),
        );
        assert_eq!(said, told);
    }

    /// This surface's words and the machine's own about a command, untranslated.
    fn with_what_a_command_says() -> Strings {
        let mut vocabulary = Vocabulary::empty();
        crate::the_cards_words::declare_the_cards_words(&mut vocabulary).unwrap();
        alo_capability::words::declare_into(&mut vocabulary).unwrap();
        Strings::of(vocabulary)
    }

    /// A command the person allowed, and what it ended with.
    fn a_command(returned: Option<i32>, said: &str) -> Answered {
        Answered::ItCannotBeUndone(Box::new(Done::RanACommand {
            returned,
            said: said.to_owned(),
            cut_short: false,
            took_too_long: false,
            sent_bytes: Some(0),
        }))
    }

    /// **A command that failed because nobody is signed in to GitHub says so,
    /// and says to ask alo again** — never *Done.*, because alo does not run it
    /// a second time by itself.
    #[test]
    fn a_command_nobody_signed_in_for_says_to_sign_in_and_ask_again() {
        let said = what_an_answer_did(
            Some(&a_command(
                Some(128),
                "fatal: could not read Username for 'https://github.com': terminal prompts \
                 disabled\n",
            )),
            &with_what_a_command_says(),
        );
        assert!(
            said.starts_with("It did not work: you are not signed in to GitHub."),
            "{said}"
        );
        for part in ["Settings → Accounts", "ask alo again", "does not try again"] {
            assert!(said.contains(part), "{part:?} is not in {said:?}");
        }
        assert!(!said.starts_with("Done"), "{said}");
    }

    /// **Any other failure is said as what it ended with**, and a command that
    /// worked is *Done.* as before.
    #[test]
    fn any_other_failure_says_what_it_ended_with_and_a_success_is_done() {
        let strings = with_what_a_command_says();
        let failed = what_an_answer_did(
            Some(&a_command(Some(1), "mkdir: cannot create directory\n")),
            &strings,
        );
        assert!(failed.starts_with("It did not work:"), "{failed}");
        assert!(failed.contains("exit status 1"), "{failed}");
        assert!(!failed.contains("GitHub"), "{failed}");

        let worked = what_an_answer_did(
            Some(&Answered::It(Box::new(Done::RanACommand {
                returned: Some(0),
                said: String::new(),
                cut_short: false,
                took_too_long: false,
                sent_bytes: Some(0),
            }))),
            &strings,
        );
        assert_eq!(worked, "Done.");
    }
}
