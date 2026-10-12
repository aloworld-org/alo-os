//! What a task changed, as a person is told it.
//!
//! Carried beside [`crate::Stopping::Stopped`] because *the work has ended* and
//! *this is what it did first* are one sentence to the person reading them, and
//! two values that can come apart if they are stored apart.

/// What a task changed on this machine.
///
/// **Empty is a real answer and is said.** A task stopped before it did
/// anything changed nothing, and a person is told that rather than shown a
/// blank area they have to interpret. The difference between *nothing changed*
/// and *nobody looked* is the difference this whole type exists for.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WhatChanged {
    /// One line per change, in the order they happened.
    done: Vec<String>,
}

impl WhatChanged {
    /// Nothing, yet.
    #[must_use]
    pub fn nothing() -> Self {
        Self::default()
    }

    /// This happened, said the way a person is told it.
    pub fn and_then(&mut self, done: impl Into<String>) {
        self.done.push(done.into());
    }

    /// Everything that happened, in order.
    #[must_use]
    pub fn each(&self) -> &[String] {
        &self.done
    }

    /// Whether anything happened at all.
    ///
    /// **Named rather than left to `each().is_empty()`**, because the two
    /// readings of an empty list are opposite — *it changed nothing* and *we
    /// have not been told* — and only one of them is this type.
    #[must_use]
    pub fn nothing_happened(&self) -> bool {
        self.done.is_empty()
    }

    /// How many changes, for a sentence that counts them.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.done.len()
    }
}
