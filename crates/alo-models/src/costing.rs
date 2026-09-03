//! What a model costs to run on this machine, said once and never used to
//! refuse it.
//!
//! `docs/features.md` at v0.5: *the machine warns and then gets out of the way.
//! A model too large for the memory in this laptop is **said so plainly, once**
//! — and then run anyway if that is what somebody asked for. The honest costs
//! the catalogue states are for deciding with, not for refusing with.*
//!
//! That is why this file holds a value and not a question. [`Cost`] is a thing
//! to show somebody; there is no `may_run`, no `fits` returning a `bool` that a
//! caller would be tempted to put an `if` in front of, and nothing here answers
//! `Err`. Refusing to try on hardware somebody owns is not a sovereignty
//! product's decision to make, and the reliable way to keep that promise is to
//! have nothing to say no with rather than to remember not to.
//!
//! # Two answers, not three
//!
//! It fits, or it does not. A middle band — *tight*, *only just* — was the
//! obvious third and is alo OS inventing a threshold about somebody else's
//! machine: nobody has measured where "only just" is, it would differ per
//! runtime and per quantisation, and a warning at a number we made up is a
//! warning people learn to ignore. The one case that is certain is the one
//! worth saying out loud.
//!
//! # The floor rather than an estimate
//!
//! A model answers out of memory, so what it needs is **at least** what its
//! weights take on disk. That is a floor, and it is what this file compares
//! against, because it is the only figure the machine actually knows: the
//! overhead a runtime adds on top depends on the runtime, the context length
//! and the quantisation, and multiplying by a number somebody guessed would
//! make [`Cost::Fits`] a claim alo OS cannot support.
//!
//! So the warning is conservative in the direction that costs nothing: weights
//! larger than the memory are certainly too large, weights smaller may still be
//! slow, and neither answer stops anything from running. The catalogue's
//! `min_ram_gb` is the other road to the same question and is a *stated* figure
//! — [`crate::Model`] carries one because whoever added the entry knew it, and
//! weights somebody brought come with nobody to ask.

use alo_strings::{Filling, Said, Strings};

use crate::words;

/// Bytes in a gigabyte, decimal — the way a publisher states a download size
/// and the way [`crate::Model::download_bytes`] is written.
///
/// Named rather than spelled out at the one place it is used, because the other
/// convention (1024³) is a tenth larger and a size that silently changed
/// convention would move the warning without anybody noticing.
pub const GIGABYTE: u64 = 1_000_000_000;

/// What running a model on this machine will cost, and never whether it may.
///
/// Both answers carry both numbers, which is this crate's rule from item 10:
/// the sentence counts nothing out loud, because a line saying *4 GB* would be
/// English's way of writing a quantity standing in for everybody else's.
/// Whoever draws the panel writes them the way this region writes a size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cost {
    /// The weights fit in the memory this machine has.
    Fits {
        /// What the weights alone need, in gigabytes.
        needs_gb: f32,
        /// What this machine has, in gigabytes.
        machine_gb: f32,
    },
    /// The weights alone are larger than the memory this machine has.
    ///
    /// **A warning, never a refusal.** Nothing in this crate turns it into one,
    /// and the file header says why.
    LargerThanMemory {
        /// What the weights alone need, in gigabytes.
        needs_gb: f32,
        /// What this machine has, in gigabytes.
        machine_gb: f32,
    },
}

impl Cost {
    /// What weights of this size cost on a machine with this much memory.
    ///
    /// A machine memory that is not a number cannot make this refuse anything,
    /// because the comparison falls to [`Fits`](Self::Fits) and there is no
    /// refusal here for it to fall to instead.
    #[must_use]
    pub fn of(bytes_on_disk: u64, machine_gb: f32) -> Self {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a size in gigabytes does not need more precision than f32 gives at these sizes"
        )]
        let needs_gb = bytes_on_disk as f32 / GIGABYTE as f32;
        if needs_gb > machine_gb {
            Self::LargerThanMemory {
                needs_gb,
                machine_gb,
            }
        } else {
            Self::Fits {
                needs_gb,
                machine_gb,
            }
        }
    }

    /// What the weights alone need, in gigabytes.
    #[must_use]
    pub fn needs_gb(self) -> f32 {
        match self {
            Self::Fits { needs_gb, .. } | Self::LargerThanMemory { needs_gb, .. } => needs_gb,
        }
    }

    /// What this machine has, in gigabytes.
    #[must_use]
    pub fn machine_gb(self) -> f32 {
        match self {
            Self::Fits { machine_gb, .. } | Self::LargerThanMemory { machine_gb, .. } => machine_gb,
        }
    }

    /// Whether this is the case worth saying out loud.
    ///
    /// A question about what to show, not about what to allow: every caller of
    /// this is drawing something, because there is nothing here to stop.
    #[must_use]
    pub fn larger_than_memory(self) -> bool {
        matches!(self, Self::LargerThanMemory { .. })
    }

    /// The string this crate declares for this answer.
    #[must_use]
    pub fn word(self) -> words::Word {
        match self {
            Self::Fits { .. } => words::WEIGHTS_FIT,
            Self::LargerThanMemory { .. } => words::WEIGHTS_LARGER_THAN_MEMORY,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics, for the reason [`crate::NotAllowed::said`]
    /// does not: a `Strings` that was never given [`crate::model_words`]
    /// answers with the key, marked, and the model runs either way.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// The one case worth saying out loud, and the one that is not.
    #[test]
    fn weights_larger_than_the_memory_are_the_case_that_is_said_out_loud() {
        let small = Cost::of(4 * GIGABYTE, 16.0);
        assert!(!small.larger_than_memory());
        assert!(matches!(small, Cost::Fits { .. }));

        let large = Cost::of(32 * GIGABYTE, 16.0);
        assert!(large.larger_than_memory());
        assert!(matches!(large, Cost::LargerThanMemory { .. }));
    }

    /// **Both numbers are beside the sentence and neither is inside it.** Item
    /// 10 settled that in this crate: how a language writes a quantity is that
    /// language's business, so the line holds no digits at all.
    #[test]
    fn the_sentence_counts_nothing_and_the_numbers_are_beside_it() {
        let strings = in_english();
        for cost in [Cost::of(4 * GIGABYTE, 16.0), Cost::of(32 * GIGABYTE, 16.0)] {
            let said = cost.said(&strings);
            assert!(!said.text().chars().any(|c| c.is_ascii_digit()), "{said}");
            assert_eq!(cost.machine_gb(), 16.0);
        }
        assert!((Cost::of(4 * GIGABYTE, 16.0).needs_gb() - 4.0).abs() < f32::EPSILON);
        assert!((Cost::of(32 * GIGABYTE, 16.0).needs_gb() - 32.0).abs() < f32::EPSILON);
    }

    /// Weights exactly the size of the memory are not the case this warns
    /// about: the floor is met, and inventing a margin here would be the
    /// threshold this file's header refuses to invent.
    #[test]
    fn weights_the_size_of_the_memory_are_not_warned_about() {
        assert!(!Cost::of(16 * GIGABYTE, 16.0).larger_than_memory());
    }

    /// **Nothing that arrives here can turn into a refusal**, whatever the
    /// numbers are. There is no `Err` and no `bool` meaning may-not-run, so the
    /// worst a nonsense figure can do is pick the wrong one of two sentences —
    /// and a machine size that is not a number at all falls to the quieter of
    /// them, because a comparison against it is false.
    #[test]
    fn nothing_that_arrives_here_becomes_a_refusal() {
        assert!(!Cost::of(400 * GIGABYTE, f32::NAN).larger_than_memory());
        // A machine that reported no memory warns, which is the whole of what
        // warning can do to anything.
        assert!(Cost::of(GIGABYTE, 0.0).larger_than_memory());
        assert!(!Cost::of(0, 0.0).larger_than_memory());
        // A runtime that said nothing about the size is weights that need
        // nothing, which is the answer that stops least.
        assert_eq!(Cost::of(0, 16.0).needs_gb(), 0.0);
    }

    /// The two answers do not share a sentence, and the warning is read in the
    /// reader's own language.
    #[test]
    fn the_warning_is_read_in_the_language_the_person_reads() {
        let strings = translated(&[(
            words::WEIGHTS_LARGER_THAN_MEMORY,
            "diese Gewichte sind größer als der Speicher dieses Rechners — alo OS führt sie \
             trotzdem aus, und es wird langsam sein",
        )]);
        let warned = Cost::of(32 * GIGABYTE, 16.0).said(&strings);
        assert!(warned.is_translated());
        assert!(warned.text().contains("trotzdem"), "{warned}");

        let english = in_english();
        assert_ne!(
            Cost::of(4 * GIGABYTE, 16.0).said(&english).text(),
            Cost::of(32 * GIGABYTE, 16.0).said(&english).text()
        );
    }
}
