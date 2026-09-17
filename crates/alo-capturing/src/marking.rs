//! **What a person can mark on a screenshot, as values.**
//!
//! Task 3 of `docs/autonomy/v0-5-capture-and-the-room-plan.md`. Somebody takes
//! a screenshot to send to a colleague, and before they send it they point at
//! the thing that matters and hide the thing that does not. That is the whole
//! of annotation, and it must happen **without opening anything else**: an
//! image editor to draw an arrow is the moment a person decides the machine is
//! not worth the trouble.
//!
//! # Marks are data; nothing here draws
//!
//! A [`Mark`] is where it is and what it says. The shell renders it, and the
//! same values are what a saved file is rendered from, so what somebody saw
//! while marking and what the file holds cannot be two different pictures.
//!
//! # The blur is not a mark like the others
//!
//! Every other mark sits **over** the picture: an arrow can be moved, a
//! rectangle recoloured, text rewritten. A blur cannot, because the reason
//! somebody reaches for it is that what is underneath must not be seen — a
//! password, an address, somebody else's name.
//!
//! A blur kept as a layer is a blur that comes off. The file travels: into a
//! chat, onto a ticket, through somebody's mail, and it is opened by tools
//! nobody here chose. So **[`Marks::saving`] destroys the pixels under a blur**,
//! and what was under it is not in the saved bytes at all —
//! `tests/a_blur_cannot_be_taken_off.rs` reads the saved pixels back and fails
//! if the original can be recovered.
//!
//! # The original is kept until they save
//!
//! Marking is a decision in progress. [`Marks`] holds the picture as it was
//! taken and the marks beside it; [`Marks::as_taken`] is still the untouched
//! screenshot, and nothing is destroyed until a person says they are finished.
//! A blur applied by mistake is undone by removing the mark, right up to the
//! moment they save — and never afterwards, which is the point.

use crate::picture::Picture;
use crate::region::Region;
use crate::words::{self, Word};

/// **Everything a person can mark on a screenshot**, and nothing else.
///
/// Closed, like every list in this repository something acts on. A tool that
/// could be any shape is a tool nothing can be tested against, and *the marks
/// are data the shell renders* would stop being true the moment one of them
/// carried code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mark {
    /// Point at something.
    Arrow {
        /// Where it starts.
        from: (u32, u32),
        /// What it points at.
        to: (u32, u32),
    },
    /// Put a box round something.
    Rectangle {
        /// The box.
        over: Region,
    },
    /// A line drawn by hand, as the points it passed through.
    Freehand {
        /// Where the hand went, in order.
        through: Vec<(u32, u32)>,
    },
    /// Words on the picture.
    Text {
        /// What it says — the person's own words, never translated.
        saying: String,
        /// Where it sits.
        at: (u32, u32),
    },
    /// **Hide what is underneath, for good.**
    Blur {
        /// What is hidden.
        over: Region,
    },
}

impl Mark {
    /// What this tool is called, in the person's own language.
    #[must_use]
    pub fn word(&self) -> Word {
        match self {
            Self::Arrow { .. } => words::AN_ARROW,
            Self::Rectangle { .. } => words::A_BOX,
            Self::Freehand { .. } => words::DRAWN_BY_HAND,
            Self::Text { .. } => words::WORDS_ON_IT,
            Self::Blur { .. } => words::HIDE_WHAT_IS_THERE,
        }
    }

    /// Whether this mark destroys what is under it when the file is saved.
    #[must_use]
    pub const fn destroys_what_is_under_it(&self) -> bool {
        matches!(self, Self::Blur { .. })
    }

    /// The area a blur hides, or [`None`] for every other mark.
    #[must_use]
    pub const fn hides(&self) -> Option<Region> {
        match self {
            Self::Blur { over } => Some(*over),
            _ => None,
        }
    }
}

/// **A screenshot, and what somebody has marked on it so far.**
#[derive(Debug, Clone, PartialEq)]
pub struct Marks {
    /// The picture as it was taken. Untouched until [`Marks::saving`].
    as_taken: Picture,
    /// What has been marked, in the order it was marked.
    marks: Vec<Mark>,
}

impl Marks {
    /// Nothing marked yet.
    #[must_use]
    pub fn on(as_taken: Picture) -> Self {
        Self {
            as_taken,
            marks: Vec::new(),
        }
    }

    /// Mark something.
    #[must_use]
    pub fn and(mut self, mark: Mark) -> Self {
        self.marks.push(mark);
        self
    }

    /// **Take a mark off**, which is possible for every mark including a blur
    /// — until the file is saved.
    #[must_use]
    pub fn without_the_last(mut self) -> Self {
        self.marks.pop();
        self
    }

    /// The picture as it was taken, still.
    #[must_use]
    pub fn as_taken(&self) -> &Picture {
        &self.as_taken
    }

    /// What has been marked.
    #[must_use]
    pub fn marks(&self) -> &[Mark] {
        &self.marks
    }

    /// Whether anything here hides something.
    #[must_use]
    pub fn hides_anything(&self) -> bool {
        self.marks.iter().any(Mark::destroys_what_is_under_it)
    }

    /// **The picture to save: every blur burnt into the pixels.**
    ///
    /// `flatten` is handed the picture and every area a blur covers, and gives
    /// back the bytes to write. It is the shell's renderer — this crate holds
    /// no pixels of its own — and what comes back is what a person's colleague
    /// will open.
    ///
    /// The marks that sit *over* the picture are not this function's business:
    /// they are rendered with it, and a person can still move them. **The blur
    /// is here because it is the one mark that must not be a layer.**
    ///
    /// # Errors
    /// Whatever `flatten` answers when it cannot render, unchanged — this crate
    /// does not turn somebody else's failure into a picture.
    pub fn saving<E>(
        &self,
        flatten: impl Fn(&Picture, &[Region]) -> Result<Picture, E>,
    ) -> Result<Picture, E> {
        let hidden: Vec<Region> = self.marks.iter().filter_map(Mark::hides).collect();
        flatten(&self.as_taken, &hidden)
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    fn a_picture() -> Picture {
        Picture::of(vec![1, 2, 3, 4, 5, 6, 7, 8]).expect("bytes are a picture")
    }

    fn somewhere() -> Region {
        Region::of(10, 10, 40, 20).expect("a region on the screen")
    }

    /// **Every tool has a name a person reads**, and no two share one.
    #[test]
    fn every_mark_is_named_in_the_vocabulary() {
        let marks = [
            Mark::Arrow {
                from: (0, 0),
                to: (5, 5),
            },
            Mark::Rectangle { over: somewhere() },
            Mark::Freehand {
                through: vec![(1, 1), (2, 2)],
            },
            Mark::Text {
                saying: "here".to_owned(),
                at: (3, 3),
            },
            Mark::Blur { over: somewhere() },
        ];
        let mut keys: Vec<String> = marks
            .iter()
            .map(|mark| mark.word().key().to_string())
            .collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), marks.len(), "two tools share a name");
    }

    /// **The original is untouched until saving**, and a mark comes off again.
    #[test]
    fn the_picture_is_as_it_was_taken_until_somebody_saves() {
        let taken = a_picture();
        let marking = Marks::on(taken.clone())
            .and(Mark::Blur { over: somewhere() })
            .and(Mark::Arrow {
                from: (0, 0),
                to: (9, 9),
            });
        assert_eq!(marking.as_taken().bytes(), taken.bytes());
        assert_eq!(marking.marks().len(), 2);

        let undone = marking.without_the_last().without_the_last();
        assert!(!undone.hides_anything(), "a blur could not be taken off");
        assert_eq!(undone.as_taken().bytes(), taken.bytes());
    }

    /// **Saving hands the renderer every area a blur covers**, and nothing else.
    #[test]
    fn saving_asks_for_every_blurred_area_and_no_other_mark() {
        let marking = Marks::on(a_picture())
            .and(Mark::Blur { over: somewhere() })
            .and(Mark::Rectangle { over: somewhere() })
            .and(Mark::Blur {
                over: Region::of(80, 80, 10, 10).expect("another region"),
            });

        let saved: Result<Picture, std::convert::Infallible> = marking.saving(|picture, hidden| {
            assert_eq!(
                hidden.len(),
                2,
                "a rectangle was treated as something to hide"
            );
            Ok(picture.clone())
        });
        assert!(saved.is_ok());

        // And a picture nobody blurred asks for nothing.
        let nothing_hidden = Marks::on(a_picture()).and(Mark::Arrow {
            from: (0, 0),
            to: (1, 1),
        });
        let saved: Result<Picture, std::convert::Infallible> =
            nothing_hidden.saving(|picture, hidden| {
                assert!(hidden.is_empty());
                Ok(picture.clone())
            });
        assert!(saved.is_ok());
    }

    /// **A blur says it destroys; nothing else does.**
    #[test]
    fn only_the_blur_destroys_what_is_under_it() {
        assert!(Mark::Blur { over: somewhere() }.destroys_what_is_under_it());
        for over_the_top in [
            Mark::Rectangle { over: somewhere() },
            Mark::Arrow {
                from: (0, 0),
                to: (1, 1),
            },
            Mark::Text {
                saying: "x".to_owned(),
                at: (0, 0),
            },
            Mark::Freehand { through: vec![] },
        ] {
            assert!(
                !over_the_top.destroys_what_is_under_it(),
                "{over_the_top:?}"
            );
            assert_eq!(over_the_top.hides(), None);
        }
    }
}
