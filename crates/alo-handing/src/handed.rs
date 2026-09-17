//! A drop that happened: who received it, in which form, and how.
//!
//! The completed half of a drag, and the shape of `alo_clipboard::Pasted` for
//! the reason the whole crate is shaped that way — a drop is a paste a person
//! aimed with their hand, and what a compositor does with the answer is the
//! same work in both cases.
//!
//! One is added here that a paste has no use for: **which application received
//! it.** A paste goes to whoever asked; a drop goes where a person let go, and
//! the delivery is decided by which window that was.

use alo_clipboard::{Kind, Taking};

use crate::delivery::Delivery;

/// What was handed over, and to whom.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handed {
    /// The application that received it.
    to: String,
    /// The form it was handed over in — one the source offered and the target
    /// takes.
    form: Kind,
    /// Whether the source keeps what it had.
    taking: Taking,
    /// How it reached the window.
    delivery: Delivery,
}

impl Handed {
    /// The drop that just happened. Made by [`crate::Drag`] and by nothing
    /// else: a value anybody could write would be a transfer nobody made.
    pub(crate) fn of(to: &str, form: Kind, taking: Taking, delivery: Delivery) -> Self {
        Self {
            to: to.to_owned(),
            form,
            taking,
            delivery,
        }
    }

    /// Which application received it.
    #[must_use]
    pub fn to(&self) -> &str {
        &self.to
    }

    /// The form it was handed over in.
    #[must_use]
    pub fn form(&self) -> &Kind {
        &self.form
    }

    /// Whether the window it came from keeps what it had.
    ///
    /// [`Taking::Cut`] is a move: the source gives its own copy up now that the
    /// transfer has completed. It can only happen once, because a [`crate::Drag`]
    /// is consumed by letting go of it — a move that happened twice is not a
    /// move, and here that is the compiler's answer rather than a rule somebody
    /// remembers.
    #[must_use]
    pub fn taking(&self) -> Taking {
        self.taking
    }

    /// How it reached the window: bytes on the spot, or files to export.
    #[must_use]
    pub fn delivery(&self) -> &Delivery {
        &self.delivery
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Everything a compositor needs to carry the drop out, and nothing else.
    #[test]
    fn a_completed_drop_says_who_got_what_and_how() {
        let handed = Handed::of(
            "org.alo.Notes",
            Kind::text(),
            Taking::Copied,
            Delivery::OnTheSpot("Müller".as_bytes().to_vec()),
        );
        assert_eq!(handed.to(), "org.alo.Notes");
        assert_eq!(handed.form(), &Kind::text());
        assert_eq!(handed.taking(), Taking::Copied);
        assert_eq!(handed.delivery().bytes(), Some("Müller".as_bytes()));
    }
}
