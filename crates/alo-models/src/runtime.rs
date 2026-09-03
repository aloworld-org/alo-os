//! What alo OS needs a model runtime to do, said in our words.
//!
//! [ADR 0006](../../../docs/decisions/0006-the-pinned-model-runtime.md) pins
//! Ollama as the runtime we ship, and puts this trait in front of it. Nothing
//! outside the adapter names an Ollama endpoint or response field: the runtime
//! is a dependency we can replace, not a shape the product is stuck in.
//!
//! Most of the vocabulary here is deliberately about **disk and memory**,
//! because that is what a person actually experiences. "Installed" means the
//! weights are on this machine and something is using the space. "Loaded" means
//! the model is in video memory now and answering quickly. Those are different
//! questions, they have different costs, and a runtime that conflates them
//! makes the honest disk accounting `docs/features.md` promises impossible.
//!
//! # And one of them is a question
//!
//! [`ModelRuntime::answers`] is the newest and the odd one out: the other six
//! manage models and this one uses one. It was missing until item 18a, which is
//! why `ROADMAP.md` said three separate times that nothing in this repository
//! could put a question to a model — `alo-asking` closed the half of that
//! sentence about a provider, and this closes the half about this machine.
//!
//! It is on this trait rather than beside the adapter for
//! [ADR 0006](../../../docs/decisions/0006-the-pinned-model-runtime.md)'s
//! reason: nothing outside `crate::ollama` names an Ollama endpoint, and a
//! question is exactly the call somebody would be most tempted to make directly.

use std::fmt;

use alo_strings::{Filling, Said, Strings};

use crate::words;

/// A model's weights, on this machine's disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installed {
    /// The catalogue id these weights answer to.
    pub id: String,
    /// What the disk actually lost, as the runtime reports it — not what the
    /// catalogue predicted. The two differ, and the true one is this.
    pub bytes_on_disk: u64,
    /// The quantisation actually installed, which is not always the one asked
    /// for and is the first thing worth knowing when a model behaves oddly.
    pub quantisation: Option<String>,
}

/// A model in video memory, answering now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loaded {
    /// The catalogue id.
    pub id: String,
    /// Video memory this is holding. On a machine with one card, this is the
    /// number that decides whether anything else can be loaded at all.
    pub vram_bytes: u64,
}

/// How far a download has got.
///
/// Reported rather than hidden because these downloads are gigabytes: a
/// progress-free wait of twenty minutes is indistinguishable from a hang, and
/// a person who cannot tell those apart will reboot the machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    /// Bytes fetched so far.
    pub done_bytes: u64,
    /// Bytes expected in total, where the runtime knows. It does not always
    /// know at the start, and guessing would make the bar lie.
    pub total_bytes: Option<u64>,
}

impl Progress {
    /// Completion as a fraction, when the total is known.
    #[must_use]
    pub fn fraction(&self) -> Option<f64> {
        match self.total_bytes {
            Some(total) if total > 0 =>
            {
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "a progress bar does not need more precision than f64 gives at these sizes"
                )]
                Some((self.done_bytes as f64 / total as f64).clamp(0.0, 1.0))
            }
            _ => None,
        }
    }
}

/// Why a runtime operation did not do what was asked.
///
/// Coarse on purpose, and it never carries a backend response body: an error
/// surface that quotes whatever the runtime said is a way for one component's
/// internals to end up in another's logs.
///
/// **No `Display`, and therefore not a `std::error::Error`** (item 9f). Every
/// one of these is read by a person waiting for a download or for an answer,
/// which is the moment they are least willing to guess, so the only road to
/// words is [`RuntimeError::said`] and it takes the strings that person reads.
///
/// **Every reason is a variant**, and that is what item 9f changed here rather
/// than only how the words are reached. The old `Refused(&'static str)` carried
/// a sentence an adapter wrote in English — one `to_string()` from a screen, in
/// a file whose author had no reason to think about language. An adapter that
/// needs to refuse for a reason not on this list adds one, and adds the string
/// beside it in [`crate::words`], the same way a verb is added to a closed list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    /// The runtime is not running, or not reachable where it was expected.
    Unreachable,
    /// The runtime is there, and did not answer inside the time this machine
    /// waits.
    ///
    /// **Only [`ModelRuntime::answers`] produces this**, and the difference
    /// from [`Unreachable`](Self::Unreachable) is the whole reason it exists: a
    /// listing that takes ten seconds means the runtime is not well, and a
    /// model that takes five minutes means it is thinking. ADR 0007 makes the
    /// CPU the default, so thinking slowly is the ordinary case rather than the
    /// broken one, and a person told *nothing was running* about a machine that
    /// was busy would go looking for a fault that is not there.
    TookTooLong,
    /// The model is not in the catalogue, so alo OS does not offer it. This is
    /// a refusal, not a failure: the catalogue is what makes the licence
    /// promise in `docs/features.md` true.
    NotOffered(String),
    /// The runtime does not have this model, and was not asked to fetch it.
    NotInstalled(String),
    /// There is not enough disk for the download.
    ///
    /// The two numbers are **beside** the sentence rather than inside it: a
    /// size is counted, how a language counts is that language's business
    /// (item 9a), and this crate settled in item 10 that it says nothing out
    /// loud that it would have to count.
    NotEnoughDisk {
        /// What the download will take.
        needed_bytes: u64,
        /// What the disk has.
        free_bytes: u64,
    },
    /// The runtime answered, but not with anything usable.
    Unusable,
    /// A download stopped before it had everything, so nothing was installed.
    DownloadIncomplete,
}

impl RuntimeError {
    /// The string this crate declares for this failure.
    #[must_use]
    pub fn word(&self) -> words::Word {
        match self {
            Self::Unreachable => words::RUNTIME_UNREACHABLE,
            Self::TookTooLong => words::RUNTIME_TOOK_TOO_LONG,
            Self::NotOffered(_) => words::MODEL_NOT_OFFERED,
            Self::NotInstalled(_) => words::MODEL_NOT_INSTALLED,
            Self::NotEnoughDisk { .. } => words::NOT_ENOUGH_DISK,
            Self::Unusable => words::RUNTIME_UNUSABLE,
            Self::DownloadIncomplete => words::DOWNLOAD_INCOMPLETE,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics: a `Strings` that was never given
    /// [`crate::model_words`] answers with the key, marked.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self {
            Self::NotOffered(model) | Self::NotInstalled(model) => {
                Filling::of("model", model.clone())
            }
            Self::Unreachable
            | Self::TookTooLong
            | Self::NotEnoughDisk { .. }
            | Self::Unusable
            | Self::DownloadIncomplete => Filling::nothing(),
        };
        strings.say(&self.word().key(), &filling)
    }
}

/// A sink for download progress.
///
/// A plain callback rather than a stream, so that a caller with nowhere to show
/// progress can pass [`Progress::ignored`] and the download path stays one path
/// rather than two.
pub trait ProgressSink: Send {
    /// Called as the download advances. Must not block for long: the download
    /// is waiting on it.
    fn advanced(&mut self, progress: Progress);
}

impl Progress {
    /// A sink that discards progress, for callers with nowhere to show it.
    #[must_use]
    pub fn ignored() -> impl ProgressSink {
        struct Ignore;
        impl ProgressSink for Ignore {
            fn advanced(&mut self, _progress: Progress) {}
        }
        Ignore
    }
}

impl<F: FnMut(Progress) + Send> ProgressSink for F {
    fn advanced(&mut self, progress: Progress) {
        self(progress);
    }
}

/// What alo OS asks of a model runtime.
///
/// Every method is a question or an action a person would recognise. There is
/// deliberately **no method that runs an arbitrary runtime command**: this
/// trait is reached by agent verbs (`docs/contracts/agent-verbs.md`), and an
/// escape hatch here would be an escape hatch there, which CLAUDE.md law 2
/// forbids.
pub trait ModelRuntime: fmt::Debug + Send + Sync {
    /// Which models are on this machine's disk.
    ///
    /// # Errors
    /// [`RuntimeError::Unreachable`] if the runtime is not answering.
    fn installed(&self) -> Result<Vec<Installed>, RuntimeError>;

    /// Which models are in video memory now.
    ///
    /// # Errors
    /// [`RuntimeError::Unreachable`] if the runtime is not answering.
    fn loaded(&self) -> Result<Vec<Loaded>, RuntimeError>;

    /// Fetch a model's weights from upstream, reporting progress.
    ///
    /// # Errors
    /// [`RuntimeError::NotEnoughDisk`] before starting where the size is known,
    /// [`RuntimeError::Unreachable`] if the runtime is not answering, and
    /// [`RuntimeError::DownloadIncomplete`] if it stopped part-way.
    fn fetch(&self, id: &str, progress: &mut dyn ProgressSink) -> Result<(), RuntimeError>;

    /// Remove a model's weights, giving the disk back.
    ///
    /// # Errors
    /// [`RuntimeError::NotInstalled`] if it was not there to remove.
    fn remove(&self, id: &str) -> Result<(), RuntimeError>;

    /// Put a model into video memory so it answers immediately.
    ///
    /// A model answers without this — the runtime loads on first use — but the
    /// first answer then waits for gigabytes to move. Loading deliberately is
    /// how a person avoids paying that at the moment they ask.
    ///
    /// # Errors
    /// [`RuntimeError::NotInstalled`] if the weights are not on this machine,
    /// and [`RuntimeError::Unreachable`] if the runtime is not answering.
    fn load(&self, id: &str) -> Result<(), RuntimeError>;

    /// Take a model out of video memory without removing it from disk.
    ///
    /// The distinction people care about on a one-card machine: unloading
    /// makes room for something else and costs only the time to load it again.
    ///
    /// # Errors
    /// [`RuntimeError::NotInstalled`] if the model is not on this machine.
    fn unload(&self, id: &str) -> Result<(), RuntimeError>;

    /// Put a question to a model on this machine, and answer with what it said.
    ///
    /// **Nothing leaves the machine** (ADR 0008): this is the path law 1's
    /// zero-egress claim is about, and it is why there is no indicator, no
    /// destination and no policy anywhere in this signature. There is nothing
    /// here for a rule to permit, because there is nothing here that goes
    /// anywhere.
    ///
    /// # The question is borrowed, and no implementation may keep it
    ///
    /// A `&str` for the length of one call. ADR 0001 §7 says alo OS never keeps
    /// the question a person asked, and this trait is the one place in
    /// `alo-models` where one arrives at all — so it arrives borrowed, it goes
    /// into one request body, and nothing in [`RuntimeError`] has a field it
    /// could come back in. An adapter that logged it, cached it or put it in an
    /// error would be breaking that promise on behalf of every caller, and
    /// `crate::ollama` has the test that says it does not.
    ///
    /// **The catalogue does not gate this, and [`fetch`](Self::fetch) does.**
    /// The licence promise in `docs/features.md` is about what alo OS *offers*,
    /// which is what it downloads; a model already on somebody's own disk was
    /// either fetched through that gate or put there by the person whose
    /// machine it is, and refusing to use it would be alo OS overruling its
    /// owner about their own hardware.
    ///
    /// # Errors
    /// [`RuntimeError::NotInstalled`] if the model is not on this machine,
    /// [`RuntimeError::Unreachable`] if the runtime is not answering,
    /// [`RuntimeError::TookTooLong`] if it is answering and the model has not
    /// finished, and [`RuntimeError::Unusable`] if what came back is not an
    /// answer.
    fn answers(&self, question: &str, of_model: &str) -> Result<String, RuntimeError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_reports_a_fraction_only_when_the_total_is_known() {
        let known = Progress {
            done_bytes: 50,
            total_bytes: Some(200),
        };
        assert!(
            known
                .fraction()
                .is_some_and(|f| (f - 0.25).abs() < f64::EPSILON)
        );

        // A runtime that has not yet said how big the download is must not be
        // made to look like one that has: a bar at an invented position is
        // worse than no bar.
        let unknown = Progress {
            done_bytes: 50,
            total_bytes: None,
        };
        assert_eq!(unknown.fraction(), None);
    }

    /// A runtime that over-reports is not allowed to produce a bar past the
    /// end, which looks like a bug in us rather than in it.
    #[test]
    fn progress_past_the_end_is_clamped() {
        let over = Progress {
            done_bytes: 300,
            total_bytes: Some(200),
        };
        assert_eq!(over.fraction(), Some(1.0));
    }

    #[test]
    fn a_zero_total_does_not_divide_by_zero() {
        let zero = Progress {
            done_bytes: 0,
            total_bytes: Some(0),
        };
        assert_eq!(zero.fraction(), None);
    }

    /// The error text is what a person reads when a download fails at 3am, so
    /// it says what happened rather than naming an internal state — and the two
    /// numbers are beside it, in the variant, for whoever writes them the way
    /// this region writes a size.
    #[test]
    fn a_full_disk_says_so_and_carries_the_numbers_beside_the_sentence() {
        let full = RuntimeError::NotEnoughDisk {
            needed_bytes: 5_000_000_000,
            free_bytes: 1_000_000_000,
        };
        let said = full.said(&crate::testing::in_english());
        assert!(said.text().contains("not enough room"), "{said}");
        // The sentence counts nothing out loud, so it holds no digits at all.
        assert!(!said.text().chars().any(|c| c.is_ascii_digit()), "{said}");
        assert!(matches!(
            full,
            RuntimeError::NotEnoughDisk {
                needed_bytes: 5_000_000_000,
                free_bytes: 1_000_000_000
            }
        ));
    }

    /// **A model id is the machine's and the sentence around it is the
    /// reader's.** The id is what a person types back to ask for it again, so
    /// translating it would make the sentence unusable in the one way that
    /// matters.
    #[test]
    fn a_model_that_is_not_installed_says_so_in_the_readers_language() {
        let strings = crate::testing::translated(&[(
            crate::words::MODEL_NOT_INSTALLED,
            "{model} ist nicht installiert",
        )]);
        let said = RuntimeError::NotInstalled("mistral-7b-instruct".to_owned()).said(&strings);
        assert!(said.is_translated());
        assert_eq!(said.text(), "mistral-7b-instruct ist nicht installiert");
    }

    /// A download that stopped is a reason of its own rather than a sentence an
    /// adapter wrote, which is what item 9f changed about this type.
    #[test]
    fn a_download_that_stopped_says_nothing_was_installed() {
        let said = RuntimeError::DownloadIncomplete.said(&crate::testing::in_english());
        assert!(said.text().contains("nothing was installed"), "{said}");
    }
}
