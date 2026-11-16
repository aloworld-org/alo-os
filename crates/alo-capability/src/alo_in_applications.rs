//! **Where alo works in an application**: in a copy of its own, in the
//! person's own window, or in a copy where the application has one and the
//! person's window where it has none — which the person chooses
//! ([ADR 0143](../../../docs/decisions/0143-alo-works-inside-applications-and-may-post-in-the-persons-name.md)
//! Q6, the owner's answer A of 2026-11-10, and the owner's decision of
//! 2026-11-16).
//!
//! # What this is, and what it is not
//!
//! It is the **answer** and nothing else, beside [`crate::AloInTheTerminal`]
//! for that type's reason: a person's choices live in their own settings and
//! whoever acts is handed the answer. It is kept in the same file
//! (`docs/contracts/how-code-runs-file.md`, `in-applications`), under the same
//! rules about who may have written it, so an agent cannot turn it up for
//! itself.
//!
//! # The default, since 2026-11-16
//!
//! **alo's own copy where the application has one, and the person's own window
//! where it has none.** The owner, 2026-11-16: *"it should be automatic for any
//! application installed not for just those too, because the users will not
//! use only 3 apps"* — and, asked whether alo should then work in the person's
//! own window by default, *"Yes, automatically"*. That supersedes answer A of
//! 2026-11-10 (*refused at the default*) for applications with no copy of
//! their own; for an application with one, alo's copy is still used, so what
//! it sends is still counted.
//!
//! **Switched off** is [`AloInApplications::InItsOwnCopy`]: only alo's own
//! copy, and an application with none is refused, as it was before. It keeps
//! the word it always had in the file, so a person who chose it keeps it.
//! **A file that will not read answers that**, never the default
//! ([`AloInApplications::WHEN_UNREADABLE`]): a file somebody else may have
//! written cannot speak for the person, and the answer that watches nothing of
//! theirs is the only safe one.
//!
//! **What it costs is said, and only said** ([`AloInApplications::what_it_costs`]).
//! The indicator, the record and alo's teal border stay on at every answer:
//! nothing alo does in an application ever goes unshown (law 1). Each change
//! still waits for the person's OK at *ask each time*: where alo works is not
//! whether it asks.

use alo_strings::Word;

use crate::words;

/// Where alo works in an application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum AloInApplications {
    /// In alo's own copy where the application has one, and in the person's
    /// own window where it has none. **How a machine arrives**: a machine with
    /// nothing written down answers this (the owner, 2026-11-16).
    #[default]
    InItsOwnCopyOrTheirWindow,
    /// Only in a separate copy of the application started for alo, whose
    /// sending is counted on its own — the setting switched off.
    ///
    /// An application that cannot open a copy of its own is refused at this
    /// answer, in [`words::NO_COPY_OF_ITS_OWN`]'s sentence, which names the
    /// answer that lets alo work in the person's window and where it is.
    InItsOwnCopy,
    /// alo may work in the person's own window, where a line on the indicator
    /// says the application may be sending and nothing is counted.
    InTheirWindow,
}

impl AloInApplications {
    /// All three, in the order Settings draws them: the default first.
    pub const ALL: [Self; 3] = [
        Self::InItsOwnCopyOrTheirWindow,
        Self::InItsOwnCopy,
        Self::InTheirWindow,
    ];

    /// What a reader answers for a file that is there and will not read:
    /// **only alo's own copy**, never the default, because a file somebody else
    /// may have written cannot speak for the person.
    pub const WHEN_UNREADABLE: Self = Self::InItsOwnCopy;

    /// What this answer is, in words a person reads: the row's name in
    /// Settings.
    #[must_use]
    pub const fn the_word(self) -> Word {
        match self {
            Self::InItsOwnCopyOrTheirWindow => words::WORKS_IN_ITS_COPY_OR_YOUR_WINDOW,
            Self::InItsOwnCopy => words::WORKS_IN_ITS_OWN_COPY,
            Self::InTheirWindow => words::MAY_WORK_IN_YOUR_WINDOW,
        }
    }

    /// The line under the row's name.
    #[must_use]
    pub const fn what_it_is(self) -> Word {
        match self {
            Self::InItsOwnCopyOrTheirWindow => words::WORKS_IN_ITS_COPY_OR_YOUR_WINDOW_IS,
            Self::InItsOwnCopy => words::WORKS_IN_ITS_OWN_COPY_IS,
            Self::InTheirWindow => words::MAY_WORK_IN_YOUR_WINDOW_IS,
        }
    }

    /// Whether alo may work in the person's own window of an application —
    /// one that `has_a_copy_of_its_own` for alo, or not.
    ///
    /// **The one question every site asks**, so that the desktop opening a
    /// file, the desktop letting alo type and the shell starting alo's work
    /// cannot come to disagree. At the default, an application with a copy of
    /// its own is never worked in in the person's window: its copy is the one
    /// whose sending can be counted.
    #[must_use]
    pub const fn may_work_in_their_window_of(self, has_a_copy_of_its_own: bool) -> bool {
        match self {
            Self::InItsOwnCopyOrTheirWindow => !has_a_copy_of_its_own,
            Self::InItsOwnCopy => false,
            Self::InTheirWindow => true,
        }
    }

    /// What choosing this costs, where it costs anything.
    ///
    /// [`Some`] for every answer that lets alo into the person's window.
    /// [`None`] for alo's own copy only, which costs nothing a person needs
    /// warning about — for [`crate::HowCodeRuns::what_it_costs`]'s reason, a
    /// warning on a choice with no risk teaches a person to dismiss warnings.
    #[must_use]
    pub const fn what_it_costs(self) -> Option<Word> {
        match self {
            Self::InItsOwnCopyOrTheirWindow => Some(words::WHAT_YOUR_WINDOW_COSTS_WHERE_NO_COPY),
            Self::InItsOwnCopy => None,
            Self::InTheirWindow => Some(words::WHAT_WORKING_IN_YOUR_WINDOW_COSTS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A machine arrives with alo in its own copy where there is one and in
    /// the person's window where there is none** (the owner, 2026-11-16).
    #[test]
    fn a_machine_arrives_with_alo_in_a_copy_or_where_there_is_none_the_persons_window() {
        let arrives = AloInApplications::default();
        assert_eq!(arrives, AloInApplications::InItsOwnCopyOrTheirWindow);
        assert!(arrives.may_work_in_their_window_of(false));
        assert!(
            !arrives.may_work_in_their_window_of(true),
            "an application with a copy was worked in in the person's window"
        );
        assert_eq!(AloInApplications::ALL[0], arrives);
    }

    /// **Switched off, alo works in its own copy only**, whatever the
    /// application; turned up, in the person's window of any application.
    #[test]
    fn switched_off_never_the_persons_window_and_turned_up_always_may() {
        for has_a_copy in [false, true] {
            assert!(!AloInApplications::InItsOwnCopy.may_work_in_their_window_of(has_a_copy));
            assert!(AloInApplications::InTheirWindow.may_work_in_their_window_of(has_a_copy));
        }
    }

    /// **A file that will not read answers alo's own copy only**, never the
    /// default that lets alo into the person's window.
    #[test]
    fn a_file_that_will_not_read_lets_alo_into_no_window_of_the_persons() {
        assert_eq!(
            AloInApplications::WHEN_UNREADABLE,
            AloInApplications::InItsOwnCopy
        );
        assert_ne!(
            AloInApplications::WHEN_UNREADABLE,
            AloInApplications::default()
        );
    }

    /// **Every answer that lets alo into the person's window says what that
    /// costs**, and only alo's own copy says nothing.
    #[test]
    fn every_answer_that_reaches_the_persons_window_says_what_it_costs() {
        for answer in AloInApplications::ALL {
            let reaches = answer.may_work_in_their_window_of(false);
            assert_eq!(answer.what_it_costs().is_some(), reaches, "{answer:?}");
        }
        let cost = AloInApplications::InTheirWindow
            .what_it_costs()
            .map(|word| word.says());
        assert_eq!(
            cost,
            Some(
                "alo will show that the application is sending, but cannot count what it sends, \
                 and works where your other tabs are"
            )
        );
    }

    /// **The refusal names the answer that lets alo work in the person's
    /// window exactly**, in the words Settings draws it in, so a person can
    /// find the row it points to.
    #[test]
    fn the_refusal_names_the_row_that_lets_alo_work_in_their_window() {
        let refusal = words::NO_COPY_OF_ITS_OWN.says();
        let row = AloInApplications::InTheirWindow.the_word().says();
        assert!(refusal.contains(&format!("“{row}”")), "{refusal}");
    }
}
