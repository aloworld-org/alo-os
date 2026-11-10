//! **Where alo works in an application**: in a copy of its own, or in the
//! person's own window — which the person chooses
//! ([ADR 0143](../../../docs/decisions/0143-alo-works-inside-applications-and-may-post-in-the-persons-name.md)
//! Q6, and the owner's answer A of 2026-11-10).
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
//! **The default is alo's own copy**, so that only what that copy sends is
//! counted on the indicator and nothing of the person's own use is. Turned up,
//! alo may work in the person's own window, and the indicator says *that* the
//! application is sending, never how much — counting the person's window would
//! be watching them, which ADR 0143 does not offer at any setting.
//!
//! **What it costs is said, and only said** ([`AloInApplications::what_it_costs`]),
//! in ADR 0143's own words. The indicator and the record stay on at both
//! answers: nothing alo does in an application ever goes unshown (law 1).

use alo_strings::Word;

use crate::words;

/// Where alo works in an application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum AloInApplications {
    /// In a separate copy of the application started for alo, whose sending
    /// is counted on its own. **How a machine arrives**: a machine with nothing
    /// written down answers this.
    ///
    /// An application that cannot open a copy of its own is refused at this
    /// answer, in [`words::NO_COPY_OF_ITS_OWN`]'s sentence, which names the
    /// other answer and where it is.
    #[default]
    InItsOwnCopy,
    /// alo may work in the person's own window, where a line on the indicator
    /// says the application may be sending and nothing is counted.
    InTheirWindow,
}

impl AloInApplications {
    /// Both, in the order Settings draws them: the default first.
    pub const ALL: [Self; 2] = [Self::InItsOwnCopy, Self::InTheirWindow];

    /// What this answer is, in words a person reads: the row's name in
    /// Settings.
    #[must_use]
    pub const fn the_word(self) -> Word {
        match self {
            Self::InItsOwnCopy => words::WORKS_IN_ITS_OWN_COPY,
            Self::InTheirWindow => words::MAY_WORK_IN_YOUR_WINDOW,
        }
    }

    /// The line under the row's name.
    #[must_use]
    pub const fn what_it_is(self) -> Word {
        match self {
            Self::InItsOwnCopy => words::WORKS_IN_ITS_OWN_COPY_IS,
            Self::InTheirWindow => words::MAY_WORK_IN_YOUR_WINDOW_IS,
        }
    }

    /// Whether alo may work in the person's own window.
    ///
    /// A method rather than a `matches!` at each site that asks, for
    /// [`crate::HowCodeRuns::runs_without_asking`]'s reason: so that the sites
    /// cannot come to disagree.
    #[must_use]
    pub const fn may_work_in_their_window(self) -> bool {
        matches!(self, Self::InTheirWindow)
    }

    /// What choosing this costs, where it costs anything.
    ///
    /// [`Some`] for the person's own window only, carrying ADR 0143's words.
    /// [`None`] for alo's own copy, which costs nothing a person needs warning
    /// about — for [`crate::HowCodeRuns::what_it_costs`]'s reason, a warning on
    /// a choice with no risk teaches a person to dismiss warnings.
    #[must_use]
    pub const fn what_it_costs(self) -> Option<Word> {
        match self {
            Self::InItsOwnCopy => None,
            Self::InTheirWindow => Some(words::WHAT_WORKING_IN_YOUR_WINDOW_COSTS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **alo's own copy until the person chooses otherwise.**
    #[test]
    fn a_machine_arrives_with_alo_in_its_own_copy() {
        assert_eq!(
            AloInApplications::default(),
            AloInApplications::InItsOwnCopy
        );
        assert!(!AloInApplications::default().may_work_in_their_window());
        assert_eq!(AloInApplications::ALL[0], AloInApplications::default());
    }

    /// **The cost is said for the one answer that has one**, in ADR 0143's
    /// words: that the indicator can then say only *that* something is sent.
    #[test]
    fn only_the_persons_own_window_says_what_it_costs() {
        assert_eq!(AloInApplications::InItsOwnCopy.what_it_costs(), None);
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

    /// **The refusal names the other answer exactly**, in the words Settings
    /// draws it in, so a person can find the row it points to.
    #[test]
    fn the_refusal_names_the_row_that_lets_alo_work_in_their_window() {
        let refusal = words::NO_COPY_OF_ITS_OWN.says();
        let row = AloInApplications::InTheirWindow.the_word().says();
        assert!(refusal.contains(&format!("“{row}”")), "{refusal}");
    }
}
