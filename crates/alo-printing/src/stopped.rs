//! What is wrong with a printer that stopped, as a closed set, and what a
//! person does about each.
//!
//! ★ *Printers, solved — found, set up, and fixed when they stop.* This file is
//! the last clause. Finding a printer is solved everywhere; **saying what is
//! wrong in a sentence a person can act on** is solved nowhere, and the reason
//! is that every system passes on what the printer said — a reason keyword, a
//! status code, a light pattern — and leaves the person to translate it.
//!
//! # Five things, and no sixth
//!
//! [`Stopped`] is out of paper, jammed, out of ink, refused the document, or
//! not answering. There is no member carrying what the printer said, because a
//! member like that is where a code would reach a person: a printer reporting
//! a door open, a fuser fault or a word nobody has defined yet is **not
//! answering**, with [`Tried`] saying what this machine did to find out. A
//! person reading *not answering — this machine asked it how it is* walks to
//! the printer and looks at its own screen, which is the right thing to do and
//! the only thing a sixth sentence could have told them.
//!
//! # Which one, when a printer reports several
//!
//! A jam is said before empty paper and empty paper before empty ink, because
//! that is the order a person has to fix them in: a printer with paper stuck in
//! it will not take new paper, and one with no paper will not show whether its
//! ink is gone. One sentence, the one to act on first; when it is fixed and the
//! printer is asked again, the next is said.
//!
//! # What a printer's report is read as
//!
//! The printing service reports a printer's state and a list of reasons, each
//! a keyword that may carry `-report`, `-warning` or `-error` on the end. A
//! report or a warning does not stop a printer and is not read as a stop:
//! *toner low* is not *out of toner*, and a machine that said it was would be
//! wrong about the one thing this file is for.

use alo_strings::{Filling, Said, Strings};

use crate::words;

/// What this machine tried, when a printer is not answering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tried {
    /// It asked its own printing service, which did not respond.
    AskingThisMachinesPrinting,
    /// It asked the printer how it is, and got nothing it can act on.
    AskingThePrinter,
    /// It looked for how the printer was set up, and did not find it.
    LookingForItsSetUp,
}

/// What is wrong with a printer that stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stopped {
    /// It has no paper.
    OutOfPaper,
    /// Paper is stuck in it.
    Jammed,
    /// It has run out of ink or toner.
    OutOfInk,
    /// It refused the document.
    RefusedTheJob,
    /// It is not answering, and this is what was tried.
    NotAnswering {
        /// What this machine did to find out.
        tried: Tried,
    },
}

impl Stopped {
    /// Every member there is. What a test walks, and what a settings panel
    /// that explains each would list.
    pub const EVERY: [Self; 7] = [
        Self::OutOfPaper,
        Self::Jammed,
        Self::OutOfInk,
        Self::RefusedTheJob,
        Self::NotAnswering {
            tried: Tried::AskingThisMachinesPrinting,
        },
        Self::NotAnswering {
            tried: Tried::AskingThePrinter,
        },
        Self::NotAnswering {
            tried: Tried::LookingForItsSetUp,
        },
    ];

    /// The string this crate declares for this.
    #[must_use]
    pub fn word(self) -> words::Word {
        match self {
            Self::OutOfPaper => words::OUT_OF_PAPER,
            Self::Jammed => words::JAMMED,
            Self::OutOfInk => words::OUT_OF_INK,
            Self::RefusedTheJob => words::REFUSED_THE_JOB,
            Self::NotAnswering {
                tried: Tried::AskingThisMachinesPrinting,
            } => words::NOT_ANSWERING_THE_SERVICE,
            Self::NotAnswering {
                tried: Tried::AskingThePrinter,
            } => words::NOT_ANSWERING_THE_PRINTER,
            Self::NotAnswering {
                tried: Tried::LookingForItsSetUp,
            } => words::NOT_ANSWERING_NOT_SET_UP,
        }
    }

    /// What is wrong and what to do about it, in the language the person reads.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }

    /// What is wrong with a printer, from what the printing service reports
    /// about it — or [`None`] when nothing is.
    ///
    /// `state` is the printer's state (3 idle, 4 printing, 5 stopped),
    /// `reasons` its reason keywords, and `accepting` whether it is taking
    /// documents.
    #[must_use]
    pub fn of_the_printer(
        state: Option<i32>,
        reasons: &[&str],
        accepting: Option<bool>,
    ) -> Option<Self> {
        /// The printer state meaning stopped.
        const STOPPED: i32 = 5;

        let stopping: Vec<&str> = reasons
            .iter()
            .filter_map(|reason| stopping_reason(reason))
            .collect();
        let any = |test: fn(&str) -> bool| stopping.iter().any(|reason| test(reason));

        if any(is_a_jam) {
            return Some(Self::Jammed);
        }
        if any(|reason| matches!(reason, "media-empty" | "media-needed")) {
            return Some(Self::OutOfPaper);
        }
        if any(|reason| matches!(reason, "marker-supply-empty" | "toner-empty" | "ink-empty")) {
            return Some(Self::OutOfInk);
        }
        if !stopping.is_empty() || state == Some(STOPPED) {
            return Some(Self::NotAnswering {
                tried: Tried::AskingThePrinter,
            });
        }
        if accepting == Some(false) {
            return Some(Self::RefusedTheJob);
        }
        None
    }
}

/// A reason keyword that stops a printer, without its severity — or [`None`]
/// for one that does not.
fn stopping_reason(reason: &str) -> Option<&str> {
    let reason = reason.trim();
    if reason.is_empty() || reason == "none" {
        return None;
    }
    if reason.ends_with("-report") || reason.ends_with("-warning") {
        return None;
    }
    // A low supply is a warning whatever it is suffixed with, and never a stop.
    if reason.ends_with("-low") || reason.contains("-low-") {
        return None;
    }
    Some(reason.strip_suffix("-error").unwrap_or(reason))
}

/// Whether a reason means paper is stuck.
fn is_a_jam(reason: &str) -> bool {
    reason == "jam" || reason.ends_with("-jam") || reason.contains("-jam-")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Each of the closed set is read from what printers really report.**
    #[test]
    fn each_thing_that_is_wrong_is_read_from_what_printers_report() {
        let cases: [(&[&str], Stopped); 8] = [
            (&["media-empty-error"], Stopped::OutOfPaper),
            (&["media-needed"], Stopped::OutOfPaper),
            (&["media-jam-error"], Stopped::Jammed),
            (&["input-tray-media-jam"], Stopped::Jammed),
            (&["marker-supply-empty-error"], Stopped::OutOfInk),
            (&["toner-empty"], Stopped::OutOfInk),
            (
                &["offline-report", "connecting-to-device"],
                Stopped::NotAnswering {
                    tried: Tried::AskingThePrinter,
                },
            ),
            (
                &["door-open-error"],
                Stopped::NotAnswering {
                    tried: Tried::AskingThePrinter,
                },
            ),
        ];
        for (reasons, stopped) in cases {
            assert_eq!(
                Stopped::of_the_printer(Some(5), reasons, Some(true)),
                Some(stopped),
                "{reasons:?}"
            );
        }
    }

    /// **A warning is not a stop.** Toner running low and a report the
    /// printer sends for information leave it ready.
    #[test]
    fn a_warning_or_a_report_is_not_a_stop() {
        for reasons in [
            &["none"][..],
            &["toner-low-report"],
            &["marker-supply-low-warning"],
            &["media-low"],
            &["media-empty-warning"],
            &[],
        ] {
            assert_eq!(
                Stopped::of_the_printer(Some(3), reasons, Some(true)),
                None,
                "{reasons:?}"
            );
        }
    }

    /// **A printer stopped for no reason it gave is not answering**, never a
    /// silent ready and never a code.
    #[test]
    fn a_printer_stopped_for_no_reason_it_gave_is_not_answering() {
        assert_eq!(
            Stopped::of_the_printer(Some(5), &["none"], Some(true)),
            Some(Stopped::NotAnswering {
                tried: Tried::AskingThePrinter
            })
        );
        assert_eq!(
            Stopped::of_the_printer(Some(3), &["none"], Some(false)),
            Some(Stopped::RefusedTheJob)
        );
    }

    /// A jam is said first, then paper, then ink: the order they are fixed in.
    #[test]
    fn the_first_thing_to_fix_is_the_thing_said() {
        assert_eq!(
            Stopped::of_the_printer(
                Some(5),
                &[
                    "marker-supply-empty-error",
                    "media-empty-error",
                    "media-jam-error"
                ],
                Some(true)
            ),
            Some(Stopped::Jammed)
        );
        assert_eq!(
            Stopped::of_the_printer(
                Some(5),
                &["marker-supply-empty-error", "media-empty-error"],
                None
            ),
            Some(Stopped::OutOfPaper)
        );
    }

    /// Every member has a sentence of its own.
    #[test]
    fn every_member_has_a_sentence_of_its_own() {
        let mut named: Vec<&str> = Stopped::EVERY
            .iter()
            .map(|stop| stop.word().named())
            .collect();
        named.sort_unstable();
        named.dedup();
        assert_eq!(named.len(), Stopped::EVERY.len());
    }
}
