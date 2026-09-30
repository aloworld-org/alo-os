//! Whether alo is waiting on the person, and what for.
//!
//! Part of task 7 of `docs/autonomy/putting-a-window-aside.md`: *Requires you* when a
//! decision is waiting. A person who put a window away and left alo working in it must be
//! able to see that it has stopped and is waiting for them, **without bringing it back**.
//!
//! # A waiting state carries the question, or it is not a waiting state
//!
//! *Requires you* with nothing after it is the worst version of the empty control ADR 0009
//! forbids, because it is one a person will act on: they bring the window back to find out
//! what is wanted, and nothing is wanted, or something is and nobody said what.
//!
//! So this is not a `bool`. [`RequiresYou::Yes`] carries the sentence describing what is
//! waiting, and refuses an empty one — the same shape as the repository's rule that **what a
//! person approves is a sentence**, and that a proposed change is put in front of them *with
//! a sentence describing it*.
//!
//! # The sentence is the agent's words, not a label
//!
//! It is data, like a window's title, and it travels through this crate untouched. Nothing
//! here writes user-facing prose, so nothing here needs translating — the labels around it,
//! *Requires you* and *Stop*, belong to whoever draws and are externalised there.

/// Why a waiting state could not be made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotWaiting {
    /// Waiting, with nothing named.
    #[error("a decision is waiting and nothing says what it is, which a person cannot answer")]
    ItSaysNothing,
}

/// Whether alo has stopped and needs the person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequiresYou {
    /// It is working, and nothing is waiting.
    No,
    /// It has stopped and needs an answer. The sentence says what for.
    Yes(String),
}

impl RequiresYou {
    /// A decision is waiting, and this is what it is.
    ///
    /// # Errors
    ///
    /// [`NotWaiting::ItSaysNothing`] for an empty or blank sentence. A person shown
    /// *Requires you* with no question would bring the window back to find out what is
    /// wanted — which is the cost of the empty control, paid in the one case where it is
    /// certain to be clicked.
    pub fn about(what: &str) -> Result<Self, NotWaiting> {
        if what.trim().is_empty() {
            return Err(NotWaiting::ItSaysNothing);
        }
        Ok(Self::Yes(what.to_owned()))
    }

    /// Whether the person is needed.
    #[must_use]
    pub const fn is_waiting(&self) -> bool {
        matches!(self, Self::Yes(_))
    }

    /// What is waiting, if anything is.
    #[must_use]
    pub fn what(&self) -> Option<&str> {
        match self {
            Self::No => None,
            Self::Yes(what) => Some(what),
        }
    }
}
