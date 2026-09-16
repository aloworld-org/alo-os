//! The shape drawn beside a line, so the indicator is never colour alone.
//!
//! [ADR 0010](../../../docs/decisions/0010-terracotta-is-reserved-and-never-alone.md)
//! settled that a signal carried by hue alone fails for anybody who cannot
//! distinguish that hue, and it measured the reason: terracotta on cream is
//! 2.87:1, under the 3.0:1 WCAG 2.1 §1.4.11 asks of a shape carrying meaning
//! and well under the 4.5:1 EN 301 549 asks of ordinary text. So the agent
//! never appears without a **mark** and a **word** beside its colour.
//!
//! This indicator takes the same rule one step further, because it has three
//! things to distinguish rather than one: each of the screen, the camera and
//! the microphone has a mark of its own, so *which* of them is in use is
//! answerable without reading either a colour or a sentence.
//!
//! # Nothing here draws
//!
//! A [`Mark`] says which shape, not how big, where, in what stroke, or with
//! what animation. The shell draws it; `docs/features.md` puts the indicator's
//! eventual home in the dock's status area, and none of that is this value's to
//! know. The names are the shapes rather than the meanings — a reader of the
//! drawing code should be able to tell what to draw without looking up what it
//! stands for.

/// The shape drawn beside one line of the in-use indicator.
///
/// One per thing a machine can watch or listen with. The three are deliberately
/// unalike in outline rather than in detail: a person distinguishing them at
/// the size of a status-area glyph is reading silhouette, not strokes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Mark {
    /// A wide rectangle: the screen.
    Rectangle,
    /// A circle inside a ring: the camera's lens.
    Lens,
    /// An upright capsule on a stem: the microphone.
    Capsule,
}

impl Mark {
    /// All three, in the order the lines are ordered.
    pub const EVERY: [Self; 3] = [Self::Rectangle, Self::Lens, Self::Capsule];

    /// The name this shape is written down by, where a drawing has to name one.
    ///
    /// Not something a person reads: it is the shape's name for whoever is
    /// wiring a glyph to it, which is why it is a plain `&str` and not a
    /// `Said`. What a person reads is the line's own sentence.
    #[must_use]
    pub const fn named(self) -> &'static str {
        match self {
            Self::Rectangle => "rectangle",
            Self::Lens => "lens",
            Self::Capsule => "capsule",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// **No two marks are the same shape.** Two lines sharing a silhouette
    /// would leave colour as the only thing telling a camera from a microphone,
    /// which is the failure ADR 0010 is about.
    #[test]
    fn no_two_marks_are_the_same_shape() {
        let shapes: BTreeSet<Mark> = Mark::EVERY.into_iter().collect();
        assert_eq!(shapes.len(), Mark::EVERY.len());
        let names: BTreeSet<&str> = Mark::EVERY.iter().map(|mark| mark.named()).collect();
        assert_eq!(names.len(), Mark::EVERY.len());
    }

    /// Every mark has a name for whoever draws it, and none of them is empty.
    #[test]
    fn every_mark_can_be_named_by_whoever_draws_it() {
        for mark in Mark::EVERY {
            assert!(!mark.named().is_empty(), "{mark:?}");
        }
    }
}
