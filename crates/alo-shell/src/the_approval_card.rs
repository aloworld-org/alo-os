//! **The approval card, as the owner approved it on 2026-10-31**: frame
//! `549:135` *alo window · Waiting for your permission* on page `544:30910`,
//! card `549:193`. Measured with the Figma connector on 2026-11-01 and recorded
//! in `docs/design/the-ask-alo-task-window.md`, *The approval card, as
//! approved*.
//!
//! From the top, inside a white card with a hairline and an 18 corner:
//!
//! 1. **Who is asking** — alo's 12 mark, 8, then *alo is asking to* in
//!    Label/Small and alo's teal; 16 above it and 20 in from each side.
//! 2. **The action**, in Heading/Medium — the very sentence the person
//!    approves (ADR 0001 §5), 6 under that line and 16 above what follows.
//! 3. **A row for each thing it touches** (`549:200`), 12 under what is above
//!    it: `bg/cool`, corner 12, 12 inside; a 40 white tile at corner 10 with
//!    the file icon; then the name in SemiBold 14/22 and, 2 under it, where it
//!    is in Geist Mono 12/18 and `text/muted`. Added 2026-11-01 by the owner's
//!    decision, which put what a change touches on the wire
//!    (`alo_protocol::Touched`).
//! 4. **The choices**, on a row of their own in `bg/canvas` under a hairline:
//!    14 above and below, *Allow once* (Primary) and *Deny* (Quiet) **both 144
//!    wide**, 8 apart, and *Allows this one action. alo asks again next time.*
//!    in Body/Medium and `text/muted` at the far end.
//!
//! Everything here is a logical measure, converted through the display's scale
//! where it is drawn (`crate::task_raster`).

/// The card's corner.
pub(crate) const THE_CARDS_CORNER: u32 = 18;
/// In from each side of the card, for every line in it.
pub(crate) const BESIDE: u32 = 20;
/// Above *alo is asking to*.
pub(crate) const ABOVE_WHO_IS_ASKING: u32 = 16;
/// The line *alo is asking to* is on: Label/Small's 16.
pub(crate) const WHO_IS_ASKING_IS_TALL: u32 = 16;
/// alo's mark.
pub(crate) const THE_MARK_IS: u32 = 12;
/// Between the mark and *alo is asking to*.
pub(crate) const AFTER_THE_MARK: u32 = 8;
/// Between *alo is asking to* and the action.
pub(crate) const ABOVE_THE_ACTION: u32 = 6;
/// Below the action, before the row of choices.
pub(crate) const BELOW_THE_ACTION: u32 = 16;
/// The hairlines: round the card, and over the row of choices.
pub(crate) const A_HAIRLINE: u32 = 1;
/// Above and below the two answers, inside their row.
pub(crate) const ABOVE_AND_BELOW_THE_ANSWERS: u32 = 14;
/// Each answer's width: **the same for both**, as the approved frame draws
/// them — *equally reachable* drawn as equally sized.
pub(crate) const AN_ANSWER_IS_WIDE: u32 = 144;
/// Between the two answers, and between Deny and the line beside them.
pub(crate) const BETWEEN: u32 = 8;
/// The least room the line beside the answers is written in. Narrower than
/// this, a window drops the line and keeps the answers: a sentence broken
/// into a word a row would be read as nothing.
pub(crate) const THE_LEAST_ROOM_BESIDE: u32 = 160;

/// Where everything on one card is, from the card's own top-left corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TheCard {
    /// How wide the card is.
    pub(crate) wide: u32,
    /// How tall the action's words are, as measured in Heading/Medium at
    /// [`Self::the_actions_room`].
    pub(crate) the_action_is_tall: u32,
    /// How many rows naming what the change touches it has.
    pub(crate) rows: u32,
    /// How tall the lines saying what the change does and leaves alone are
    /// (`549:209`), and nought where its verb says neither.
    pub(crate) what_it_does_is_tall: u32,
}

/// Above the lines saying what the change does: the column's 12.
pub(crate) const ABOVE_WHAT_IT_DOES: u32 = 12;
/// The tick before each of those lines.
pub(crate) const A_TICK: u32 = 16;
/// Between the tick and its words.
pub(crate) const AFTER_A_TICK: u32 = 8;
/// Between the two, side by side, as the frame draws them.
pub(crate) const BETWEEN_WHAT_IT_DOES: u32 = 20;
/// Between the two when they do not fit side by side and are drawn one under
/// the other. **A departure the frame does not draw**: it shows two short
/// English sentences on one line, and a translation or a longer sentence may
/// not fit there.
pub(crate) const UNDER_WHAT_IT_DOES: u32 = 4;

/// The most rows a card draws. No verb on the list names more than three
/// things; the bound is for a message that claims more, which must not make a
/// card taller than any window.
pub(crate) const THE_MOST_ROWS: usize = 4;
/// Above each row naming what the change touches.
pub(crate) const ABOVE_A_ROW: u32 = 12;
/// A row naming what the change touches: 12, the 40 tile, 12.
pub(crate) const A_ROW_IS_TALL: u32 = 64;
/// Inside a row, round its tile and its words.
pub(crate) const INSIDE_A_ROW: u32 = 12;
/// A row's corner.
pub(crate) const A_ROWS_CORNER: u32 = 12;
/// The white tile at the start of a row.
pub(crate) const A_TILE_IS: u32 = 40;
/// The tile's corner.
pub(crate) const A_TILES_CORNER: u32 = 10;
/// The file icon in the tile.
pub(crate) const THE_ICON_IN_A_TILE: u32 = 20;
/// The name's line: SemiBold 14/22.
pub(crate) const A_NAME_IS_TALL: u32 = 22;
/// Between the name and where it is.
pub(crate) const UNDER_THE_NAME: u32 = 2;
/// Where it is: Geist Mono 12/18.
pub(crate) const WHERE_IT_IS_TALL: u32 = 18;

impl TheCard {
    /// Where row `which` starts, down, counted from nought.
    pub(crate) const fn a_row(self, which: u32) -> u32 {
        self.the_action()
            + self.the_action_is_tall
            + ABOVE_A_ROW
            + which * (A_ROW_IS_TALL + ABOVE_A_ROW)
    }

    /// Where the words in a row start, across, and how much room they have.
    pub(crate) const fn a_rows_words(self) -> (u32, u32) {
        let starts = BESIDE + INSIDE_A_ROW + A_TILE_IS + INSIDE_A_ROW;
        (
            starts,
            self.wide.saturating_sub(starts + INSIDE_A_ROW + BESIDE),
        )
    }

    /// The room the action's words have across: the card less its sides.
    pub(crate) const fn the_actions_room(wide: u32) -> u32 {
        wide.saturating_sub(BESIDE * 2)
    }

    /// Where *alo is asking to* and the mark's line starts, down.
    pub(crate) const fn who_is_asking(self) -> u32 {
        ABOVE_WHO_IS_ASKING
    }

    /// Where the action starts, down.
    pub(crate) const fn the_action(self) -> u32 {
        ABOVE_WHO_IS_ASKING + WHO_IS_ASKING_IS_TALL + ABOVE_THE_ACTION
    }

    /// Where the row of choices starts, down — its hairline.
    pub(crate) const fn the_choices(self) -> u32 {
        let what_it_does = if self.what_it_does_is_tall == 0 {
            0
        } else {
            ABOVE_WHAT_IT_DOES + self.what_it_does_is_tall
        };
        self.the_action()
            + self.the_action_is_tall
            + self.rows * (ABOVE_A_ROW + A_ROW_IS_TALL)
            + what_it_does
            + BELOW_THE_ACTION
    }

    /// Where the lines saying what it does start, down.
    pub(crate) const fn what_it_does(self) -> u32 {
        self.the_action()
            + self.the_action_is_tall
            + self.rows * (ABOVE_A_ROW + A_ROW_IS_TALL)
            + ABOVE_WHAT_IT_DOES
    }

    /// Where the two answers start, down.
    pub(crate) const fn the_answers(self) -> u32 {
        self.the_choices() + A_HAIRLINE + ABOVE_AND_BELOW_THE_ANSWERS
    }

    /// How tall the whole card is.
    pub(crate) const fn tall(self) -> u32 {
        self.the_answers() + alo_tasks::measures::A_CONTROL_IS + ABOVE_AND_BELOW_THE_ANSWERS
    }

    /// Where *Deny* starts, across.
    pub(crate) const fn deny(self) -> u32 {
        BESIDE + AN_ANSWER_IS_WIDE + BETWEEN
    }

    /// Where the line beside the answers may start, across, and how much room
    /// it has: it ends 20 in from the card's right, as the frame draws it.
    pub(crate) const fn the_line_beside(self) -> (u32, u32) {
        let starts = self.deny() + AN_ANSWER_IS_WIDE + BETWEEN;
        (starts, self.wide.saturating_sub(starts + BESIDE))
    }
}

#[cfg(test)]
#[path = "the_approval_card_tests.rs"]
mod tests;
