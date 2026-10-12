//! Where each block inside the scrolling conversation goes, and what is out of
//! view.
//!
//! [`crate::layout`] answers where the five rows are. This answers what is
//! *inside* the one that scrolls: a person's request, alo's reply, an activity
//! block, an approval card — stacked in the order they were said, with the
//! newest at the bottom, and clipped to the row.
//!
//! # Heights come in; they are not computed here
//!
//! How tall a block is depends on how many lines its text takes, which depends
//! on the font, its size, the person's text-size setting and the width
//! available. **None of that is knowable in this crate**, which holds no font
//! and draws nothing. So each block's height is a parameter, measured by the
//! shell where the font system lives, and what is decided here is the stacking,
//! the widths, the alignment and the clipping.
//!
//! *This is the same split `alo-put-aside` makes and for the same reason: the
//! conversion from a design's figures to a display's pixels happens once, in
//! the crate that knows the display.*
//!
//! # Logical measures, used as pixels for now
//!
//! Every figure is [`crate::measures`]' logical measure. **No display scale is
//! applied**, here or by the shell that draws this, which matches
//! `crate::alo_shell::dock_raster`'s documented decision: every figure in the
//! Dock is a logical measure used directly as a pixel, because scaling one
//! measure of an otherwise unscaled geometry is worse than scaling none.
//! Making the shell scale-aware is one change that moves all of them together,
//! and this joins that change rather than inventing a conversion of its own.

use crate::measures as m;

/// Which side of the conversation a block sits on, and how wide it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhoseBlock {
    /// A person's own request: **right aligned**, in a bubble, narrower than
    /// the column.
    ///
    /// The narrowness and the side are what tell a person's words from alo's
    /// without relying on colour — which matters to anybody who cannot see the
    /// difference between `bg/cool` and the surface behind it.
    ThePersons,
    /// Alo's reply: left aligned, the full column, no bubble.
    Alos,
    /// An activity block or an approval card: left aligned, the full column, in
    /// a bubble.
    AnAccount,
}

impl WhoseBlock {
    /// How wide this kind of block is drawn.
    #[must_use]
    pub const fn how_wide(self) -> u32 {
        match self {
            Self::ThePersons => m::A_REQUEST_IS_WIDE,
            Self::Alos | Self::AnAccount => m::ALOS_SIDE_IS_WIDE,
        }
    }

    /// Whether it is drawn in a bubble with padding inside it.
    #[must_use]
    pub const fn is_in_a_bubble(self) -> bool {
        matches!(self, Self::ThePersons | Self::AnAccount)
    }
}

/// One block, placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct APlacedBlock {
    /// Which kind it is, and so its width and its side.
    pub whose: WhoseBlock,
    /// How far down the conversation's own content this block starts.
    ///
    /// **Content coordinates, not screen ones**: the conversation scrolls, so a
    /// block's place in the content outlives where it happens to be drawn.
    pub from_the_top: u32,
    /// How far from the conversation row's left inside edge it starts.
    pub from_the_left: u32,
    /// Its width.
    pub wide: u32,
    /// Its height, as it was measured.
    pub tall: u32,
}

impl APlacedBlock {
    /// Just past its bottom.
    #[must_use]
    pub const fn past_its_bottom(self) -> u32 {
        self.from_the_top + self.tall
    }

    /// Whether any part of it is inside a view of `how_tall` scrolled to
    /// `scrolled_to`.
    #[must_use]
    pub const fn is_in_view(self, scrolled_to: u32, how_tall: u32) -> bool {
        let top_of_view = scrolled_to;
        let past_the_view = scrolled_to + how_tall;
        self.from_the_top < past_the_view && self.past_its_bottom() > top_of_view
    }
}

/// The conversation's blocks, stacked, and how much they come to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheBlocks {
    /// Every block, oldest first — the order a person reads them in.
    blocks: Vec<APlacedBlock>,
    /// How tall the whole content is.
    how_tall: u32,
    /// The width the blocks were placed in.
    inside: u32,
}

impl TheBlocks {
    /// Stack `each` block, in order, inside a conversation row `row_is_wide`
    /// across.
    ///
    /// Each entry is a kind and the height that kind's content was measured at.
    #[must_use]
    pub fn stacked(each: &[(WhoseBlock, u32)], row_is_wide: u32) -> Self {
        let inside = row_is_wide.saturating_sub(m::INSIDE_THE_CONVERSATION * 2);
        let mut blocks = Vec::with_capacity(each.len());
        let mut down = 0;
        for (which, (whose, tall)) in each.iter().enumerate() {
            // **The gap goes before every block but the first**, so a single
            // block starts flush against the padding rather than one gap down.
            if which > 0 {
                down += m::BETWEEN_WHAT_IS_SAID;
            }
            let wide = whose.how_wide().min(inside);
            blocks.push(APlacedBlock {
                whose: *whose,
                from_the_top: down,
                // **Right aligned means measured from the right**, so a
                // person's request stays against the far edge whatever the
                // window's width is — which is the thing that marks it as
                // theirs.
                from_the_left: match whose {
                    WhoseBlock::ThePersons => inside.saturating_sub(wide),
                    WhoseBlock::Alos | WhoseBlock::AnAccount => 0,
                },
                wide,
                tall: *tall,
            });
            down += tall;
        }
        Self {
            blocks,
            how_tall: down,
            inside,
        }
    }

    /// Every block, oldest first.
    #[must_use]
    pub fn each(&self) -> &[APlacedBlock] {
        &self.blocks
    }

    /// How tall the whole conversation's content is.
    #[must_use]
    pub const fn how_tall(&self) -> u32 {
        self.how_tall
    }

    /// The width the blocks were placed inside, after the row's padding.
    #[must_use]
    pub const fn inside(&self) -> u32 {
        self.inside
    }

    /// How far down the content a view of `how_tall` must be scrolled to sit at
    /// the newest block.
    ///
    /// **Where a conversation opens**, and where it returns to when something
    /// new is said: at the bottom. A conversation that opened at the top would
    /// show a person the request they typed an hour ago and hide what alo just
    /// answered.
    ///
    /// Zero when everything fits, so a short conversation sits against the top
    /// rather than being pushed down by a scroll it does not need.
    #[must_use]
    pub const fn scrolled_to_the_newest(&self, how_tall: u32) -> u32 {
        self.how_tall.saturating_sub(how_tall)
    }

    /// Whether anything is out of view above and below, for a view of
    /// `how_tall` scrolled to `scrolled_to`.
    ///
    /// Both answered together, because they are what a scroll rail is drawn
    /// from and two callers asking separately is two chances to disagree about
    /// whether there is anything to scroll.
    #[must_use]
    pub const fn out_of_view(&self, scrolled_to: u32, how_tall: u32) -> (bool, bool) {
        let above = scrolled_to > 0;
        let below = scrolled_to + how_tall < self.how_tall;
        (above, below)
    }

    /// The blocks with any part of them inside the view.
    pub fn in_view(&self, scrolled_to: u32, how_tall: u32) -> impl Iterator<Item = &APlacedBlock> {
        self.blocks
            .iter()
            .filter(move |block| block.is_in_view(scrolled_to, how_tall))
    }
}

#[cfg(test)]
#[path = "conversation_layout_tests.rs"]
mod tests;
