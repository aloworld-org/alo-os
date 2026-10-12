//! Where everything in a task window goes, for the window's actual size.
//!
//! **Not for 1040 × 800.** That is the size somebody drew the design at, and
//! `CLAUDE.md` is explicit that no machine is obliged to be it: *every layout is
//! derived from the display it is on*. So this takes a size and answers with
//! rows, and the only figures it holds are [`crate::measures`]' logical ones.
//!
//! # Four rows keep their height and one takes what is left
//!
//! The heading, the shared files, the composer and the scope note are each a
//! fixed logical height, because each holds one line or one row of controls and
//! neither grows with the window. **The conversation takes the remainder**,
//! which is why there is no constant for it: it is the scrolling row, and a
//! window is made taller precisely so that more of the conversation fits.
//!
//! # The eight pixels the design does not account for, and what is done with
//! them
//!
//! At the design's own 1040 × 800 the four fixed rows take 306 of the 798 inside
//! the border, leaving **492** — and the design draws the conversation at
//! **484**. The eight are unlabelled. What they are is trailing space at the
//! bottom of the frame: its children stack from the top, so the shortfall falls
//! below the scope note.
//!
//! **This gives them to the conversation instead, deliberately**, and that is a
//! decision rather than a measurement. Dead space along the bottom edge of a
//! window is a bug when a window can be any height — at 800 it is eight pixels
//! and at 2000 it would be eight pixels in a very different-looking place, and
//! nothing in the design says the gap should be there at all. The alternative
//! was to reproduce the shortfall exactly, which reproduces an artefact of how
//! the frame was built.
//!
//! So: at 1040 × 800 this lays the conversation out **eight taller than the
//! frame**, every other row the same height, and
//! `docs/design/the-ask-alo-task-window.md` records the residue so a reader can
//! disagree with the reasoning rather than discover the outcome.
//!
//! **And it has a consequence worth stating, because a test caught it:**
//! everything below the conversation sits **eight lower** than the frame draws
//! it. The composer starts at 620 rather than 612, its control row at 696
//! rather than 688, and a popover above it ends at 678 rather than 670. The
//! design's absolute vertical figures therefore do **not** hold here, by this
//! decision — only the relationships do, which is why
//! [`TheRows::a_popover_from`] is written as *this far above the control row*
//! and never as a `top`. The scope note is flush to the window's inside edge,
//! which is the point: there is no dead strip along the bottom.

use crate::measures as m;

/// Why a window could not be laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TooSmall {
    /// There is not enough height left for a conversation.
    ///
    /// Carries the least a window may be, so a caller can say what it needs
    /// rather than only that it failed.
    NotTallEnough {
        /// The least height that leaves [`m::THE_LEAST_CONVERSATION`].
        the_least: u32,
    },
    /// The composer's controls do not fit across the window.
    NotWideEnough {
        /// The least width the controls fit in.
        the_least: u32,
    },
}

/// One row of the window, in logical measures from the window's own corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ARow {
    /// How far down the window's inside this row starts.
    pub from_the_top: u32,
    /// How tall it is.
    pub tall: u32,
    /// How wide it is — the window less its two borders, for every row.
    pub wide: u32,
}

impl ARow {
    /// Just past its bottom, which is where the next row starts.
    #[must_use]
    pub const fn past_its_bottom(self) -> u32 {
        self.from_the_top + self.tall
    }
}

/// The five rows of a task window, top to bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheRows {
    /// The title, what kind of task it is, and the status.
    pub heading: ARow,
    /// The files a person explicitly shared.
    pub shared_files: ARow,
    /// What was said and what was done. **The row that stretches.**
    pub conversation: ARow,
    /// The reply field and every control.
    pub composer: ARow,
    /// *Permissions and model choices apply to this task.*
    pub scope_note: ARow,
}

/// Where each control on the composer sits, across the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheControls {
    /// How far down the window the row of controls starts.
    pub from_the_top: u32,
    /// Attach.
    pub attach: u32,
    /// The model's name and its chevron.
    pub model: u32,
    /// Where the processing happens, which is text and not a control.
    pub destination: u32,
    /// Permissions.
    pub permissions: u32,
    /// Send, or Stop. **One slot**, because the design draws one button.
    pub the_one_button: u32,
    /// The space between the permissions control and the one button, which is
    /// what pushes the button to the right-hand end.
    pub flexible_space: u32,
}

/// The least width the composer's controls fit across.
///
/// Derived from the controls and the paddings rather than stated, so a control
/// that changes width moves the floor with it.
#[must_use]
pub const fn the_least_width() -> u32 {
    let controls = m::A_CONTROL_IS
        + m::THE_MODEL_CONTROL_IS_WIDE
        + m::THE_DESTINATION_IS_WIDE
        + m::A_CONTROL_IS
        + m::A_CONTROL_IS;
    // Five gaps: one after each of the first four controls, and one before the
    // button, which is where the flexible space sits when there is room for it.
    let gaps = m::GAP * 5;
    // Two borders for the window and two for the reply field inside it.
    controls + gaps + (m::INSIDE_THE_FIELD * 2) + (m::BESIDE_A_ROW * 2) + (m::BORDER * 4)
}

/// The least height that leaves a conversation worth the name.
#[must_use]
pub const fn the_least_height() -> u32 {
    m::THE_ROWS_THAT_DO_NOT_STRETCH + m::THE_LEAST_CONVERSATION + (m::BORDER * 2)
}

impl TheRows {
    /// Lay a window of this logical size out.
    ///
    /// Both sizes are **logical**: a caller converts for its display's scale at
    /// the boundary, and never passes pixels in here.
    pub const fn of(wide: u32, tall: u32) -> Result<Self, TooSmall> {
        if wide < the_least_width() {
            return Err(TooSmall::NotWideEnough {
                the_least: the_least_width(),
            });
        }
        if tall < the_least_height() {
            return Err(TooSmall::NotTallEnough {
                the_least: the_least_height(),
            });
        }
        let across = wide - (m::BORDER * 2);
        let down = tall - (m::BORDER * 2);
        // The conversation takes what the other four do not. Subtraction is
        // safe: the height floor above is this sum plus the conversation's own
        // floor.
        let conversation = down - m::THE_ROWS_THAT_DO_NOT_STRETCH;

        let heading = ARow {
            from_the_top: 0,
            tall: m::THE_HEADING_IS_TALL,
            wide: across,
        };
        let shared_files = ARow {
            from_the_top: heading.past_its_bottom(),
            tall: m::THE_SHARED_FILES_ARE_TALL,
            wide: across,
        };
        let conversation = ARow {
            from_the_top: shared_files.past_its_bottom(),
            tall: conversation,
            wide: across,
        };
        let composer = ARow {
            from_the_top: conversation.past_its_bottom(),
            tall: m::THE_COMPOSER_IS_TALL,
            wide: across,
        };
        let scope_note = ARow {
            from_the_top: composer.past_its_bottom(),
            tall: m::THE_SCOPE_NOTE_IS_TALL,
            wide: across,
        };
        Ok(Self {
            heading,
            shared_files,
            conversation,
            composer,
            scope_note,
        })
    }

    /// Where the composer's controls sit.
    #[must_use]
    pub const fn controls(&self) -> TheControls {
        // The field's contents, inside the composer's padding and the field's.
        // **The row starts without the field's border and is two narrower than
        // the space it starts in**, which is asymmetric and is what the design
        // draws. At 1040 the field is 974 with 12 of padding, so its contents
        // begin at 44 — the design's own figure for Attach — and that leaves
        // 950; but the drawn row is **948**. The two are slack on the right,
        // unexplained in the design exactly as the vertical eight is.
        //
        // Reproduced rather than tidied, because the figure that matters is
        // where the one button lands: 944, which is the design's. Taking the
        // two off the left instead would put Attach at 45 and move every
        // control.
        let left = m::BESIDE_A_ROW + m::INSIDE_THE_FIELD;
        let row_is_wide = self.composer.wide
            - (m::BESIDE_A_ROW * 2)
            - (m::INSIDE_THE_FIELD * 2)
            - (m::BORDER * 2);

        let fixed = m::A_CONTROL_IS
            + m::THE_MODEL_CONTROL_IS_WIDE
            + m::THE_DESTINATION_IS_WIDE
            + m::A_CONTROL_IS
            + m::A_CONTROL_IS;
        let flexible_space = row_is_wide - fixed - (m::GAP * 5);

        let attach = left;
        let model = attach + m::A_CONTROL_IS + m::GAP;
        let destination = model + m::THE_MODEL_CONTROL_IS_WIDE + m::GAP;
        let permissions = destination + m::THE_DESTINATION_IS_WIDE + m::GAP;
        let the_one_button = permissions + m::A_CONTROL_IS + m::GAP + flexible_space + m::GAP;

        TheControls {
            // The row sits at the bottom of the field, inside its padding.
            from_the_top: self.composer.from_the_top + m::ABOVE_THE_FIELD + m::THE_FIELD_IS_TALL
                - m::BELOW_THE_FIELDS_CONTENTS
                - m::A_CONTROL_IS,
            attach,
            model,
            destination,
            permissions,
            the_one_button,
            flexible_space,
        }
    }

    /// Where a popover opened from the control at `control_left` goes.
    ///
    /// Answers the popover's top-left, given how tall it is. **Opens upward**:
    /// its bottom sits [`m::A_POPOVER_STANDS_OFF`] above the control row, so a
    /// taller popover grows away from the composer rather than over it.
    #[must_use]
    pub const fn a_popover_from(&self, control_left: u32, popover_is_tall: u32) -> (u32, u32) {
        let controls = self.controls();
        let bottom = controls.from_the_top - m::A_POPOVER_STANDS_OFF;
        (
            control_left + m::A_POPOVER_IS_INSET,
            bottom.saturating_sub(popover_is_tall),
        )
    }
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod tests;
