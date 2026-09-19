//! The one value that says a session may end, and the only two ways there are
//! to get one.
//!
//! A session ends when the thing that started it stops, and that is not this
//! crate's to do: `alo-sessiond` opens sessions and the base's own session
//! manager ends them. What is decided here is **what has to have happened
//! first**, and it is carried as a value rather than written in a comment,
//! because a comment is not something a compositor has to hold in its hand.
//!
//! [`MayEnd`] has no public constructor. There are exactly two:
//!
//! - [`crate::logging_out::asked`] answering [`crate::LoggingOut::Ready`],
//!   which happens when every application was asked to close and every one of
//!   them closed; and
//! - [`crate::WouldNotClose::even_so`], which is the person — having been shown
//!   the name of every application that would not close — saying *log out
//!   anyway*.
//!
//! ```compile_fail
//! let ends = alo_leaving::MayEnd { after: alo_leaving::AfterWhat::EverythingClosed };
//! ```
//!
//! So *never kills one silently* is the shape of this file. There is no third
//! road, no `MayEnd::now()`, and nothing in this crate that ends or signals a
//! process at all: the forced end is the session ending over an application
//! that said no, and the person asked for it by name.

/// A session may end: every application was asked first.
///
/// Held by whoever ends it, and made only by the two roads named at the top of
/// this file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MayEnd {
    /// Which of the two roads it came by.
    after: AfterWhat,
}

impl MayEnd {
    /// Every application closed when it was asked.
    pub(crate) const fn everything_closed() -> Self {
        Self {
            after: AfterWhat::EverythingClosed,
        }
    }

    /// The person read the names and said *log out anyway*.
    pub(crate) const fn the_person_insisted() -> Self {
        Self {
            after: AfterWhat::ThePersonInsisted,
        }
    }

    /// How this log-out got here, for whoever is writing down what happened.
    #[must_use]
    pub const fn after(self) -> AfterWhat {
        self.after
    }
}

/// What had happened by the time a session was allowed to end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AfterWhat {
    /// Every application was asked to close, and every one of them closed.
    EverythingClosed,
    /// Some would not close, the person was shown which, and they said to log
    /// out anyway.
    ThePersonInsisted,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Both roads say which one they were**, because *the person insisted* is
    /// the one a support engineer wants to be able to tell apart from an
    /// ordinary log-out.
    #[test]
    fn a_session_that_may_end_says_how_it_got_there() {
        assert_eq!(
            MayEnd::everything_closed().after(),
            AfterWhat::EverythingClosed
        );
        assert_eq!(
            MayEnd::the_person_insisted().after(),
            AfterWhat::ThePersonInsisted
        );
        assert_ne!(MayEnd::everything_closed(), MayEnd::the_person_insisted());
    }
}
