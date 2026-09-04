//! Why one execution could not be given a boundary, in a form a portable crate
//! can hold.
//!
//! ADR 0013 and ADR 0015 end on one rule: **a turn whose boundary cannot be
//! applied does not run.** This is what comes back when that happens, and it is
//! deliberately not the reason itself — the reason is a fact about a kernel, and
//! this crate compiles on hosts that have none.
//!
//! # Two audiences, and one of them is not the person
//!
//! A person is told one sentence, [`crate::words::NOT_BOUNDED`], and it is the
//! same sentence however the boundary failed: what they can act on is that
//! nothing happened, that nothing was refused either, and that the machine has
//! to be looked at. Putting *this kernel has no `file.f_path`* in front of
//! somebody would hand them a fact about their machine's internals, in English,
//! in the middle of a sentence in their own language.
//!
//! So [`NoBoundary::why`] carries that reason as text, in English, for the
//! service log the administrator is already reading, and it is never shown. It
//! is a `String` rather than a type because the crate that knows what it means
//! is Linux and this one is not — `alo-agentd`'s boundary writes its own reason
//! into one, and nothing here reads it.
//!
//! # A thread that could not be brought back is a different fact
//!
//! Every other reason means nothing was attempted: no cgroup, no entry in the
//! kernel, no work. [`NoBoundary::a_thread_is_still_inside`] means the work ran
//! and the thread that did it is still inside a boundary belonging to a turn
//! that is over — refused everything outside a grant that no longer exists,
//! which fails closed and costs the service a thread that will never serve
//! anybody again.
//!
//! That is why it is a question this type answers rather than one more reason in
//! the text: the daemon **stops** over it, the way it stops when nothing can be
//! written down, and a service cannot decide that by reading an English sentence.

use alo_strings::{Filling, Said, Strings};

use crate::words;

/// There was no boundary for this execution to run inside.
///
/// **No `Display`**, like every refusal a person reads in this workspace: the
/// road to words is [`NoBoundary::said`]. What [`NoBoundary::why`] answers is
/// not that road — it is the administrator's sentence, and it is English on
/// purpose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoBoundary {
    /// What whoever imposes boundaries on this machine said, in English.
    why: String,

    /// Whether a thread of this service is still inside a boundary that is over.
    a_thread_is_still_inside: bool,
}

impl NoBoundary {
    /// Nothing was attempted, and this is what the machine said about why.
    ///
    /// The ordinary shape: a kernel that will not load the programme, a control
    /// group that could not be made, a path the machine would not describe, or
    /// more places than one entry holds. No work ran and nothing was changed.
    #[must_use]
    pub fn because(why: String) -> Self {
        Self {
            why,
            a_thread_is_still_inside: false,
        }
    }

    /// The work ran and the thread that did it could not be brought back out.
    ///
    /// The worst of them, and the one a service does not carry on past. The
    /// thread is inside a boundary belonging to a turn that has ended, so it is
    /// refused everything the machine needs it to do — including, eventually,
    /// saying why.
    #[must_use]
    pub fn with_a_thread_still_inside(why: String) -> Self {
        Self {
            why,
            a_thread_is_still_inside: true,
        }
    }

    /// What the person is told, in the language they read.
    ///
    /// One sentence for every reason there is, and this crate's own rather than
    /// the boundary's: [`crate::words`] says why it moved here.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&words::NOT_BOUNDED.key(), &Filling::nothing())
    }

    /// What the machine said, in English, for whoever looks after it.
    ///
    /// Never shown to the person whose turn this was — see this module's
    /// documentation.
    #[must_use]
    pub fn why(&self) -> &str {
        &self.why
    }

    /// Whether a thread of this service is still inside a boundary that is over.
    ///
    /// A service that meets one of these has nothing to retry: it has lost a
    /// thread to a grant that has ended, and what it does about that is stop.
    #[must_use]
    pub fn a_thread_is_still_inside(&self) -> bool {
        self.a_thread_is_still_inside
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// **Every reason is one sentence to the person**, and the reason itself is
    /// nowhere in it: a person reading their own language does not meet a
    /// clause about a kernel in somebody else's.
    #[test]
    fn what_a_person_is_told_is_the_same_however_the_boundary_failed() {
        let strings = in_english();
        let first = NoBoundary::because(
            "this kernel does not publish its own type information at \
                                         /sys/kernel/btf/vmlinux"
                .to_owned(),
        );
        let second = NoBoundary::with_a_thread_still_inside(
            "a thread went into a turn and could not be brought back out".to_owned(),
        );

        assert_eq!(first.said(&strings).text(), second.said(&strings).text());
        assert!(!first.said(&strings).is_a_bug());
        assert!(
            !first.said(&strings).text().contains("vmlinux"),
            "the administrator's sentence reached the person"
        );
    }

    /// And the administrator's half is kept whole, in English, where a service
    /// log can carry it.
    #[test]
    fn the_reason_is_kept_for_whoever_looks_after_the_machine() {
        let why = "the kernel would not attach the boundary to file_open";
        assert_eq!(NoBoundary::because(why.to_owned()).why(), why);
    }

    /// **A thread left inside is a question rather than a reason**, because a
    /// service decides what to do about it and cannot decide that by reading a
    /// sentence.
    #[test]
    fn a_thread_left_inside_is_answerable_without_reading_the_reason() {
        assert!(
            !NoBoundary::because("nothing was attempted".to_owned()).a_thread_is_still_inside()
        );
        assert!(
            NoBoundary::with_a_thread_still_inside("it is still in there".to_owned())
                .a_thread_is_still_inside()
        );
    }

    /// The sentence is translated like every other one a person reads, which is
    /// what moving it out of a Linux-only list is for.
    #[test]
    fn the_sentence_is_read_in_the_language_the_person_reads() {
        let german = translated(&[(
            words::NOT_BOUNDED,
            "es wurde nichts getan: dieser Rechner kann einen Agenten nicht auf das beschränken, \
             was Sie ihm erlaubt haben",
        )]);
        let said = NoBoundary::because("no".to_owned()).said(&german);
        assert!(said.is_translated(), "{said}");
        assert!(said.text().starts_with("es wurde nichts"), "{said}");
    }
}
