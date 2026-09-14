//! What this machine has for showing a file: the kinds it opens as they are,
//! and the conversions it can make into one of those.
//!
//! **Handed in, never assumed.** Which viewers and converters a machine has is
//! a fact about what is installed on it, and whoever knows that — the shell, and
//! the conversion `docs/autonomy/v0-5-documents-and-paper-plan.md` task 2
//! builds — says so by building a [`ThisMachine`]. This crate does not ship a
//! list that says a machine can open something, because a list written here
//! would be true of no machine in particular.
//!
//! # What cannot be said about a machine
//!
//! - **That it opens a program, a locked document or an empty file.** None of
//!   them is a [`Kind`], so there is nothing to hand.
//! - **That it converts into a kind it does not open**, which would be an
//!   answer ending in a copy nothing can show.
//! - **That it both opens a kind and converts it**, or converts one kind into
//!   two: either would leave which of two answers a person gets to the order
//!   somebody registered them in.
//! - **That it converts a kind into itself.**
//!
//! Each is refused with a [`NotAnAbility`] naming the kinds involved.

use crate::kind::Kind;

/// What this machine has for showing a file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ThisMachine {
    /// Every kind it opens as it is.
    opens: Vec<Kind>,
    /// Every conversion it can make, from a kind into one it opens.
    converts: Vec<(Kind, Kind)>,
}

/// Why something cannot be said about this machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotAnAbility {
    /// A conversion into a kind the machine does not open.
    IntoAKindItDoesNotOpen {
        /// The kind converted from.
        from: Kind,
        /// The kind nothing opens.
        into: Kind,
    },
    /// A conversion from a kind the machine already opens as it is.
    FromAKindItAlreadyOpens {
        /// The kind.
        from: Kind,
    },
    /// A second conversion from a kind that already has one.
    AlreadyConverted {
        /// The kind.
        from: Kind,
        /// What it is already converted into.
        into: Kind,
    },
    /// A kind to open as it is that the machine already converts.
    AlreadyConvertedInstead {
        /// The kind.
        kind: Kind,
    },
    /// A conversion into the kind it is already.
    IntoItself {
        /// The kind.
        kind: Kind,
    },
}

/// What this machine does with one kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Handles {
    /// Opens it as it is.
    Opens,
    /// Converts a copy into this, which it opens.
    Converts(Kind),
    /// Nothing.
    Nothing,
}

impl ThisMachine {
    /// A machine that opens nothing and converts nothing.
    #[must_use]
    pub fn with_nothing() -> Self {
        Self::default()
    }

    /// The same machine, which also opens this kind as it is.
    ///
    /// Saying it twice is saying it once.
    ///
    /// # Errors
    /// [`NotAnAbility::AlreadyConvertedInstead`] when the machine already
    /// converts this kind.
    pub fn opens(mut self, kind: Kind) -> Result<Self, NotAnAbility> {
        if self.converts.iter().any(|(from, _)| *from == kind) {
            return Err(NotAnAbility::AlreadyConvertedInstead { kind });
        }
        if !self.opens.contains(&kind) {
            self.opens.push(kind);
        }
        Ok(self)
    }

    /// The same machine, which also converts a copy of one kind into another
    /// that it opens.
    ///
    /// # Errors
    /// [`NotAnAbility`], naming which of the things this module's
    /// documentation lists it would have said.
    pub fn converts(mut self, from: Kind, into: Kind) -> Result<Self, NotAnAbility> {
        if from == into {
            return Err(NotAnAbility::IntoItself { kind: from });
        }
        if !self.opens.contains(&into) {
            return Err(NotAnAbility::IntoAKindItDoesNotOpen { from, into });
        }
        if self.opens.contains(&from) {
            return Err(NotAnAbility::FromAKindItAlreadyOpens { from });
        }
        if let Some((_, already)) = self.converts.iter().find(|(had, _)| *had == from) {
            return Err(NotAnAbility::AlreadyConverted {
                from,
                into: *already,
            });
        }
        self.converts.push((from, into));
        Ok(self)
    }

    /// What this machine does with one kind.
    pub(crate) fn handles(&self, kind: Kind) -> Handles {
        if self.opens.contains(&kind) {
            return Handles::Opens;
        }
        self.converts
            .iter()
            .find(|(from, _)| *from == kind)
            .map_or(Handles::Nothing, |(_, into)| Handles::Converts(*into))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A machine that says what it has handles exactly that.
    #[test]
    fn a_machine_handles_what_it_says_it_has() {
        let machine = ThisMachine::with_nothing()
            .opens(Kind::Pdf)
            .unwrap()
            .opens(Kind::Pdf)
            .unwrap()
            .converts(Kind::WordDocument, Kind::Pdf)
            .unwrap();
        assert_eq!(machine.handles(Kind::Pdf), Handles::Opens);
        assert_eq!(
            machine.handles(Kind::WordDocument),
            Handles::Converts(Kind::Pdf)
        );
        assert_eq!(machine.handles(Kind::PngImage), Handles::Nothing);
        assert_eq!(
            ThisMachine::with_nothing().handles(Kind::Text),
            Handles::Nothing
        );
    }

    /// **Nothing contradictory can be said about a machine**, one refusal at a
    /// time.
    #[test]
    fn nothing_contradictory_can_be_said_about_a_machine() {
        let opens_pdf = ThisMachine::with_nothing().opens(Kind::Pdf).unwrap();

        assert_eq!(
            opens_pdf.clone().converts(Kind::Pdf, Kind::Pdf),
            Err(NotAnAbility::IntoItself { kind: Kind::Pdf })
        );
        assert_eq!(
            opens_pdf.clone().converts(Kind::WordDocument, Kind::Text),
            Err(NotAnAbility::IntoAKindItDoesNotOpen {
                from: Kind::WordDocument,
                into: Kind::Text
            })
        );
        let opens_both = opens_pdf.clone().opens(Kind::Text).unwrap();
        assert_eq!(
            opens_both.clone().converts(Kind::Text, Kind::Pdf),
            Err(NotAnAbility::FromAKindItAlreadyOpens { from: Kind::Text })
        );

        let converts = opens_both.converts(Kind::RichText, Kind::Pdf).unwrap();
        assert_eq!(
            converts.clone().converts(Kind::RichText, Kind::Text),
            Err(NotAnAbility::AlreadyConverted {
                from: Kind::RichText,
                into: Kind::Pdf
            })
        );
        assert_eq!(
            converts.opens(Kind::RichText),
            Err(NotAnAbility::AlreadyConvertedInstead {
                kind: Kind::RichText
            })
        );
    }
}
