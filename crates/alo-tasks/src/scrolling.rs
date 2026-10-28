//! How far a person has scrolled a task's conversation back from its newest
//! words.
//!
//! *Conversation: scrolls on y, clipped on x* — `docs/design/the-ask-alo-task-window.md`,
//! measured off the design's own frame. [`crate::TheBlocks`] already says where
//! a conversation opens and where it returns when something new is said: **at
//! the newest**. This holds the one thing a person adds to that, which is how far
//! back they went to read.
//!
//! # Held as a distance back from the newest, not down from the top
//!
//! Because the newest is where a conversation lives. Nothing here can know how
//! tall the content is — that is measured where the font is, in the shell — so
//! a position counted from the top could not say where *the newest* is, and a
//! window would open on the request somebody typed an hour ago.
//!
//! # Something new being said brings the view back
//!
//! [`crate::TheBlocks::scrolled_to_the_newest`]'s own rule. So the count of
//! turns at the moment a person last scrolled is kept beside the distance, and
//! a conversation that has grown since is read at its newest again.
//!
//! # How far back it can go is told, not worked out
//!
//! The most a view can scroll is the content's height less the view's, and only
//! a frame that measured both knows it. So the frame says it here, through
//! [`TheScroll::the_frame_measured`], and every scroll after that stops at the
//! oldest words rather than running on into nothing a person would then have to
//! scroll back out of.

/// How far one task's conversation is scrolled back from its newest words.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TheScroll {
    /// How far back from the newest, in the conversation's own pixels.
    back: u32,
    /// How far back it can go, as the last frame measured.
    the_most: u32,
    /// How many turns the conversation had when the person last scrolled.
    turns_then: usize,
}

impl TheScroll {
    /// At the newest words, where every conversation opens.
    #[must_use]
    pub const fn at_the_newest() -> Self {
        Self {
            back: 0,
            the_most: 0,
            the_view: 0,
            turns_then: 0,
        }
    }

    /// The person scrolled `back` pixels further back — or forward toward the
    /// newest, for a negative `back` — in a conversation of `turns_now` turns.
    ///
    /// Answers whether the view moved, so a scroll that moved nothing — at the
    /// oldest words, or at the newest — can be handed on rather than swallowed.
    pub fn scrolled(&mut self, back: i32, turns_now: usize) -> bool {
        let from = self.where_it_stands(turns_now);
        let to = if back >= 0 {
            from.saturating_add(back.unsigned_abs()).min(self.the_most)
        } else {
            from.saturating_sub(back.unsigned_abs())
        };
        self.back = to;
        self.turns_then = turns_now;
        to != from
    }

    /// A frame measured the conversation and found it can scroll back at most
    /// `the_most` pixels, in a view `the_view` tall.
    pub fn the_frame_measured(&mut self, the_most: u32, the_view: u32) {
        self.the_most = the_most;
        self.the_view = the_view;
        self.back = self.back.min(the_most);
    }

    /// How far one page moves: the view, less one line kept in sight.
    #[must_use]
    pub const fn a_page(&self) -> u32 {
        self.the_view
            .saturating_sub(crate::measures::A_LINE_OF_WHAT_WAS_SAID)
    }

    /// How far down the content the view starts, for a conversation that would
    /// sit at `the_newest` when not scrolled and that has `turns_now` turns.
    #[must_use]
    pub fn the_view_starts(&self, the_newest: u32, turns_now: usize) -> u32 {
        the_newest.saturating_sub(self.where_it_stands(turns_now).min(the_newest))
    }

    /// How far back it stands now: where the person left it, or at the newest
    /// once something new has been said.
    const fn where_it_stands(&self, turns_now: usize) -> u32 {
        if turns_now == self.turns_then {
            self.back
        } else {
            0
        }
    }
}

#[cfg(test)]
#[path = "scrolling_tests.rs"]
mod tests;
