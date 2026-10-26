//! The numbers a task window is built out of, and what each of them answers to.
//!
//! **Every figure here was measured off the design's own frames** and is written
//! down with its node in `docs/design/the-ask-alo-task-window.md`. None of it
//! was matched by eye, and none of it may be changed without changing that page
//! — a layout nobody can check against the design is a layout that drifts.
//!
//! # These are logical measures, not pixels
//!
//! `CLAUDE.md`: *nothing is built to one screen size*, and the design's frames
//! are 1440 × 960 because somebody had to draw them at some size. So each
//! constant here is a **logical** measure, declared once and converted for a
//! display through that display's scale, exactly as `alo_dock::measures` is.
//! The conversion happens at the boundary in the crate that knows the display,
//! never here.
//!
//! **The fault to watch is a figure that reaches a display without passing
//! through its scale**, and the usual way it arrives is scaled by the person's
//! text size only — which looks derived and is half the size it should be on a
//! dense screen.
//!
//! # What is a measure here, and what is deliberately not
//!
//! The four rows that keep their height are measures. **The conversation's
//! height is not**: it is whatever is left, which is why it is absent from this
//! file and computed in [`crate::layout`]. A constant for it would be the one
//! number that stopped the window working at any other size.

/// The window's own border, on each side.
pub const BORDER: u32 = 1;

/// The window's corners — `radius/lg` in the design, and new to this
/// repository: `figma-snapshot/variables.toml` carries only `radius/sm` and
/// `radius/md`.
pub const CORNER: u32 = 18;

/// A bubble's and a block's corners — `radius/md`.
pub const A_BLOCKS_CORNER: u32 = 12;

/// A shared-file chip's corners — `radius/sm`.
pub const A_CHIPS_CORNER: u32 = 8;

/// The task heading: the title, what kind of task it is, and the status.
pub const THE_HEADING_IS_TALL: u32 = 76;

/// The row of files a person explicitly shared with this task.
pub const THE_SHARED_FILES_ARE_TALL: u32 = 52;

/// The reply composer, including the padding above its field.
pub const THE_COMPOSER_IS_TALL: u32 = 150;

/// *Permissions and model choices apply to this task.*
pub const THE_SCOPE_NOTE_IS_TALL: u32 = 28;

/// How much of a window's height the four rows that do not stretch take.
///
/// **Derived rather than written down**, so a change to any row above cannot
/// leave this stale — which is the arithmetic a layout file exists to keep
/// honest.
pub const THE_ROWS_THAT_DO_NOT_STRETCH: u32 =
    THE_HEADING_IS_TALL + THE_SHARED_FILES_ARE_TALL + THE_COMPOSER_IS_TALL + THE_SCOPE_NOTE_IS_TALL;

/// The padding down each side of every row.
pub const BESIDE_A_ROW: u32 = 32;

/// The padding inside the conversation, on all four sides.
pub const INSIDE_THE_CONVERSATION: u32 = 32;

/// Between one thing said and the next.
pub const BETWEEN_WHAT_IS_SAID: u32 = 20;

/// The gap used between controls, chips and most neighbours.
pub const GAP: u32 = 8;

/// The padding inside a message bubble, an activity block or an approval card.
pub const INSIDE_A_BLOCK: u32 = 16;

/// How wide a person's own request is drawn.
///
/// **Narrower than alo's reply on purpose**: a person's words are right aligned
/// in a bubble and alo's run the width of the column, which is what tells the
/// two apart without relying on colour.
pub const A_REQUEST_IS_WIDE: u32 = 660;

/// How wide alo's reply and an activity block are drawn.
pub const ALOS_SIDE_IS_WIDE: u32 = 920;

/// The reply field a person types in.
pub const THE_FIELD_IS_TALL: u32 = 128;

/// The reply field's corners. **Not `radius/lg`** although it is the same 18:
/// the design gives the field a literal 18 rather than the token, and the two
/// are recorded separately so that moving the token does not silently move the
/// field.
pub const THE_FIELDS_CORNER: u32 = 18;

/// Above the reply field, inside the composer.
pub const ABOVE_THE_FIELD: u32 = 8;

/// Down each side of the reply field's contents.
pub const INSIDE_THE_FIELD: u32 = 12;

/// Above the reply field's contents.
pub const ABOVE_THE_FIELDS_CONTENTS: u32 = 16;

/// Below the reply field's contents.
pub const BELOW_THE_FIELDS_CONTENTS: u32 = 12;

/// The row of controls at the bottom of the reply field, and so also the height
/// of every control on it.
pub const A_CONTROL_IS: u32 = 48;

/// The round Send-or-Stop control's corner, which makes it a circle.
pub const THE_ONE_BUTTONS_CORNER: u32 = A_CONTROL_IS / 2;

/// A square control's corners — Attach and Permissions.
pub const A_CONTROLS_CORNER: u32 = 12;

/// *Allow once* on an approval card: Primary, 144 wide, `A_CONTROL_IS` high.
///
/// `docs/design/the-ask-alo-task-window.md`, *The approval card*.
pub const ALLOW_ONCE_IS_WIDE: u32 = 144;

/// *Deny* on an approval card: Quiet, 104 wide.
///
/// **Narrower because its label is shorter, not because it is secondary** —
/// the design says so in as many words, and both are 48 high and neither is
/// behind anything.
pub const DENY_IS_WIDE: u32 = 104;

/// How wide the control naming the model is.
pub const THE_MODEL_CONTROL_IS_WIDE: u32 = 176;

/// How wide the text saying where the processing happens is.
pub const THE_DESTINATION_IS_WIDE: u32 = 148;

/// A control's artwork frame.
///
/// **The artwork is not the target.** This is 28 inside a 48 control, and the
/// pressable thing is the 48 — *a glyph is not a target*, which this repository
/// has already got wrong once by using one number for both.
pub const ARTWORK: u32 = 28;

/// The chevron on the model control, which the design draws smaller than the
/// other artwork.
pub const A_CHEVRON: u32 = 24;

/// How thick a control's artwork is stroked.
pub const ARTWORK_IS_STROKED: u32 = 2;

/// A shared-file chip.
pub const A_CHIP_IS_WIDE: u32 = 184;

/// A shared-file chip's height.
pub const A_CHIP_IS_TALL: u32 = 36;

/// The padding before a chip's label.
pub const BEFORE_A_CHIPS_LABEL: u32 = 12;

/// A popover opened from a control on the composer.
pub const A_POPOVER_IS_WIDE: u32 = 420;

/// The padding inside a popover.
pub const INSIDE_A_POPOVER: u32 = 20;

/// Between a popover's rows.
pub const BETWEEN_A_POPOVERS_ROWS: u32 = 12;

/// How far a popover's bottom sits above the control row it opened from.
///
/// **The derived figure is this, not the popover's `top`.** Both popovers in
/// the design end at y 670 above a control row at y 688, and a layout that
/// hard-coded 670 would be wrong on any window that is not 800 tall.
pub const A_POPOVER_STANDS_OFF: u32 = 18;

/// How far a popover's left edge sits right of its control's left edge.
///
/// One, which is the border. Measured twice — the model control at x 100 with
/// its popover at 101, and permissions at 440 with its popover at 441 — and two
/// instances agreeing is what makes it a rule rather than two positions.
pub const A_POPOVER_IS_INSET: u32 = 1;

/// The least a conversation may be and still be a conversation: three lines of
/// body text.
///
/// **A floor, so the window has a smallest honest size** rather than laying
/// itself out with a negative conversation on a short display. Three lines
/// because two is a question and its answer with nowhere for the next thing.
pub const THE_LEAST_CONVERSATION: u32 = 75;
