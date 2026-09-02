//! What alo OS needs a model runtime to do, said in our words.
//!
//! [ADR 0006](../../../docs/decisions/0006-the-pinned-model-runtime.md) pins
//! Ollama as the runtime we ship, and puts this trait in front of it. Nothing
//! outside the adapter names an Ollama endpoint or response field: the runtime
//! is a dependency we can replace, not a shape the product is stuck in.
//!
//! The vocabulary here is deliberately about **disk and memory**, because that
//! is what a person actually experiences. "Installed" means the weights are on
//! this machine and something is using the space. "Loaded" means the model is
//! in video memory now and answering quickly. Those are different questions,
//! they have different costs, and a runtime that conflates them makes the
//! honest disk accounting `docs/features.md` promises impossible.

use std::fmt;

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
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    /// The runtime is not running, or not reachable where it was expected.
    #[error("the model runtime is not reachable")]
    Unreachable,
    /// The model is not in the catalogue, so alo OS does not offer it. This is
    /// a refusal, not a failure: the catalogue is what makes the licence
    /// promise in `docs/features.md` true.
    #[error("{0} is not a model this system offers")]
    NotOffered(String),
    /// The runtime does not have this model, and was not asked to fetch it.
    #[error("{0} is not installed")]
    NotInstalled(String),
    /// There is not enough disk for the download.
    #[error("not enough disk: {needed_bytes} bytes needed, {free_bytes} free")]
    NotEnoughDisk {
        /// What the download will take.
        needed_bytes: u64,
        /// What the disk has.
        free_bytes: u64,
    },
    /// The runtime answered, but not with anything usable.
    #[error("the model runtime gave an answer that could not be used")]
    Unusable,
    /// The operation was refused by the runtime itself.
    #[error("the model runtime refused: {0}")]
    Refused(&'static str),
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
    /// [`RuntimeError::Refused`] if the runtime declined.
    fn fetch(&self, id: &str, progress: &mut dyn ProgressSink) -> Result<(), RuntimeError>;

    /// Remove a model's weights, giving the disk back.
    ///
    /// # Errors
    /// [`RuntimeError::NotInstalled`] if it was not there to remove.
    fn remove(&self, id: &str) -> Result<(), RuntimeError>;

    /// Take a model out of video memory without removing it from disk.
    ///
    /// The distinction people care about on a one-card machine: unloading
    /// makes room for something else and costs only the time to load it again.
    ///
    /// # Errors
    /// [`RuntimeError::NotInstalled`] if the model is not on this machine.
    fn unload(&self, id: &str) -> Result<(), RuntimeError>;
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
    /// it says what to do about it rather than naming an internal state.
    #[test]
    fn a_full_disk_says_how_much_was_needed() {
        let e = RuntimeError::NotEnoughDisk {
            needed_bytes: 5_000_000_000,
            free_bytes: 1_000_000_000,
        };
        let said = e.to_string();
        assert!(said.contains("5000000000"), "{said}");
        assert!(said.contains("1000000000"), "{said}");
    }
}
