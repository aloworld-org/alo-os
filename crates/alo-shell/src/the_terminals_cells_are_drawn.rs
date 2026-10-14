//! alo's own terminal, drawn: a window, a transcript, and the cells in it.
//!
//! The third of the terminal's parts and the first about a screen.
//! `alo-terminal` owns the pty and the grid and knows no display;
//! [ADR 0114](../../docs/decisions/0114-the-terminals-engine-is-pinned-and-its-face-is-bundled.md)
//! says *the compositor draws it, as it draws everything else*, and this is
//! where that happens — in the crate that knows the display's scale.
//!
//! # Every figure comes from the design, and none of them is a pixel
//!
//! `alo_terminal::measures` holds what was measured off the design file, and
//! `docs/design/the-terminal-window.md` records how each was established.
//! Everything there is a **logical** measure, so it reaches this display only
//! through this display's own scale.
//!
//! **The window's own size is the one thing not taken from there**, because
//! 1040×800 is the size the design was *drawn at* and not the size a window
//! *is*. What the design does settle is the **proportions**: 1040 of 1440
//! across and 800 of 1040 down, centred — the design leaves exactly 200 on each
//! side. So the window is that share of whatever display it is on, which is
//! what *nothing is built to one screen size* asks for.
//!
//! # The failure this file is built around, and it has two faces
//!
//! **Asking cosmic-text for a family no database holds is not an error.** It
//! has been seen two ways in this product, and a design handling only one of
//! them ships the other:
//!
//! - **It lays out nothing.** A booted machine showed that on 2026-10-12 —
//!   wallpaper, monogram, entry field and not one word, including the person's
//!   own name.
//! - **It substitutes another face, silently.** Measured 2026-10-14: a font
//!   system holding Manrope and Inter, asked for `Geist Mono`, laid out glyphs
//!   **from Inter**, proportionally. A terminal drawn that way has every column
//!   in the wrong place and **is full of letters**, so nothing about it looks
//!   like a font fault.
//!
//! The second is worse in this repository's recurring way: *a verdict that
//! agrees with itself while being wrong is invisible.* So the face is looked
//! for in the **font database**, never asked of the shaper — the shaper answers
//! *here are your glyphs* either way and cannot be the witness.
//!
//! # What is not here yet
//!
//! Colour per cell, bold, the cursor's own block, the selection, the
//! scrollback's position, the identity strip, the note under the transcript and
//! the gutter ADR 0113 describes. The grid carries attributes and this draws
//! one ground and one ink. `docs/design/the-terminal-window.md` records what
//! each is measured as, and the sessions strip is recorded there as **a
//! silence**: the design places it over the transcript's own rows and does not
//! say which is drawn when.

use alo_appearance::Scheme;
use alo_terminal::{TerminalOutput, measures};
use cosmic_text::{Buffer, FontSystem, Metrics, Shaping, Wrap};
use smithay::backend::renderer::Frame;
use smithay::utils::{Physical, Rectangle};

use crate::painted::{Inked, Solid};
use crate::painted_text::{ALOS_OWN_MONO, alos_own_mono, alos_own_mono_at, inked};

/// How much of the display's width the window takes: 1040 of 1440.
const ACROSS_THE_DISPLAY: f32 = 1040.0 / 1440.0;

/// How much of the display's height the window takes: 800 of 1040.
const DOWN_THE_DISPLAY: f32 = 800.0 / 1040.0;

/// alo's own terminal, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TerminalPicture {
    /// The display size it was laid out for.
    ///
    /// **Carried rather than recomputed**, so `validate` can refuse a frame the
    /// terminal was laid out for another display on — which is the check every
    /// other native picture in this crate makes, and for the same reason.
    pub(crate) size: (i32, i32),
    /// Flat shapes — the window and the transcript's ground — painted first.
    pub(crate) solids: Vec<Solid>,
    /// The cells, painted after the shapes.
    pub(crate) inked: Vec<Inked>,
}

impl TerminalPicture {
    /// Whether there is nothing to paint.
    pub(crate) fn is_empty(&self) -> bool {
        self.solids.is_empty() && self.inked.is_empty()
    }

    /// Refuse a picture that would draw outside the output it was laid out for.
    ///
    /// The same check every other native picture makes before a frame opens,
    /// and for the same reason: a rectangle outside the output is a drawing
    /// fault, and finding it here names the picture rather than the renderer.
    ///
    /// # Errors
    /// [`crate::RenderError::Unsupported`] when anything falls outside.
    pub(crate) fn validate(
        &self,
        size: smithay::utils::Size<i32, Physical>,
    ) -> Result<(), crate::RenderError> {
        let output: Rectangle<i32, Physical> = Rectangle::from_size(size);
        let inside = |area: Rectangle<i32, Physical>| output.contains_rect(area);
        if self.size == (size.w, size.h)
            && self.solids.iter().all(|solid| inside(solid.area))
            && self.inked.iter().all(|ink| inside(ink.area))
        {
            return Ok(());
        }
        Err(crate::RenderError::TerminalScene)
    }

    /// Paint the shapes, then the cells.
    ///
    /// # Errors
    /// Whatever the renderer refused.
    pub(crate) fn paint(&self, frame: &mut impl Frame) -> Result<(), crate::RenderError> {
        crate::painted::paint(frame, &self.solids, &self.inked)
    }
}

/// Why a terminal could not be drawn at all.
///
/// **All three are refusals rather than fallbacks.** *Never a silent fallback*:
/// a terminal drawn in a proportional face has crooked columns, and one drawn
/// in no face at all is the empty rectangle this module's header is about.
/// Either is worse than a caller being told.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum NothingToDrawWith {
    /// No face in the family a terminal is drawn in is loaded.
    #[error(
        "the font system has no face in the {ALOS_OWN_MONO} family, and cosmic-text would \
         silently draw a terminal in whatever it substituted"
    )]
    TheFaceIsMissing,
    /// The family is loaded and the shaper used something else anyway.
    #[error(
        "the shaper laid out a terminal's text in a face outside the {ALOS_OWN_MONO} family, so \
         its columns would be crooked"
    )]
    TheFaceWasNotUsed,
    /// The display is too small to put a terminal on.
    #[error("this display is too small for a terminal window")]
    TheresNoRoom,
}

/// One cell, as the face itself advances at the size it is drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ACell {
    /// How far the face moves along for one character. **Fractional on
    /// purpose**: rounding is what makes a line of eighty cells end up
    /// somewhere other than eighty advances along.
    advance: f32,
}

impl ACell {
    /// This cell in whole pixels, as the child is told it.
    ///
    /// **Rounded, and the rounding is why the drawing does not use this.** The
    /// advance is fractional on purpose so that column *n* lands at *n*
    /// advances; `TIOCSWINSZ` carries whole `u16`s, so the figure handed to a
    /// child is the nearest whole pixel. The two are different answers to
    /// different questions and neither is a rounding of the other's job.
    pub(crate) fn in_pixels(self, line_height: f32) -> alo_terminal::ACellInPixels {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a cell is two digits of pixels on any display this product lays out for,                       and both figures are positive by construction"
        )]
        let answer = alo_terminal::ACellInPixels {
            width: self.advance.round() as u16,
            height: line_height.round() as u16,
        };
        answer
    }

    /// How far along column `which` begins.
    ///
    /// **Fractional, deliberately.** The advance is the face's own, so column
    /// *n* sits at *n* advances — rounding each one would drift visibly across
    /// a wide screen, which is the whole reason this is not an integer.
    pub(crate) fn across(self, which: usize) -> f32 {
        self.advance * which as f32
    }

    /// How many whole columns fit in this many pixels.
    ///
    /// **A count, never a constant.** How many columns a terminal has is the
    /// transcript's room divided by the face's own advance, measured at the
    /// size this display is drawing.
    pub(crate) fn columns_in(self, width: i32) -> usize {
        if self.advance <= 0.0 || width <= 0 {
            return 0;
        }
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a count of columns is bounded by the width it was divided out of"
        )]
        let columns = (width as f32 / self.advance) as usize;
        columns
    }
}

/// Which faces in this font system are in the family a terminal is drawn in.
///
/// **Asked of the database, not of the shaper**, which is this file's
/// hardest-won lesson. Measured 2026-10-14: a font system holding only Manrope
/// and Inter, asked for `Geist Mono`, laid out glyphs **from Inter** with
/// advances of 14.45, 3.88 and 15.77 for `M`, `i` and `W`. Not an error, not an
/// empty box — a **substitution**, in a proportional face. So *did the shaper
/// give me glyphs* cannot answer *is my face here*: it answers yes either way.
fn the_faces_in_that_family(fonts: &FontSystem) -> Vec<cosmic_text::fontdb::ID> {
    fonts
        .db()
        .faces()
        .filter(|face| {
            face.families
                .iter()
                .any(|(family, _)| family == ALOS_OWN_MONO)
        })
        .map(|face| face.id)
        .collect()
}

/// Whether the face the shaper actually drew with is one of ours.
///
/// A function of its own so the refusal it guards can be tested at all: on
/// cosmic-text 0.14.2 a family that *is* loaded is the family that gets used,
/// so no font system this file can build reaches
/// [`NothingToDrawWith::TheFaceWasNotUsed`] through [`how_big_a_cell_is`]. The
/// guard stays because *presence is not use* and a later matcher may not agree
/// with this one.
fn the_shaper_used_one_of(
    ours: &[cosmic_text::fontdb::ID],
    drawn_in: cosmic_text::fontdb::ID,
) -> bool {
    ours.contains(&drawn_in)
}

/// The metrics the design asks for, at this display's scale.
fn the_designs_metrics(scale: f64) -> Metrics {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a two-digit logical measure through a display's scale is a size in pixels"
    )]
    let metrics = Metrics::new(
        (f64::from(measures::A_LETTER) * scale) as f32,
        (f64::from(measures::A_LINE) * scale) as f32,
    );
    metrics
}

/// Measure one cell, and refuse if the face is not there to measure.
///
/// Three things in order, and the order is load-bearing:
///
/// 1. **The metrics, before a buffer exists.** `Buffer::new` *asserts*
///    internally that a line height is not zero, so a check after it is a panic
///    and this workspace denies panics. Found by a test rather than by reading
///    cosmic-text.
/// 2. **The database holds the family.** See [`the_faces_in_that_family`].
/// 3. **The shaper actually used it.** Presence is not use, and *two fields
///    that must agree* is the only check in this repository with a hit rate.
///
/// # Errors
/// As [`NothingToDrawWith`]'s variants describe.
pub(crate) fn how_big_a_cell_is(
    fonts: &mut FontSystem,
    metrics: Metrics,
) -> Result<ACell, NothingToDrawWith> {
    if metrics.font_size <= 0.0 || metrics.line_height <= 0.0 {
        return Err(NothingToDrawWith::TheresNoRoom);
    }
    let ours = the_faces_in_that_family(fonts);
    if ours.is_empty() {
        return Err(NothingToDrawWith::TheFaceIsMissing);
    }

    // **`M`, and the choice does not matter in a monospace face** — which is
    // itself asserted in this module's tests, so this comment is checked rather
    // than believed.
    let mut buffer = Buffer::new(fonts, metrics);
    buffer.set_wrap(fonts, Wrap::None);
    buffer.set_size(fonts, None, None);
    buffer.set_text(fonts, "M", &alos_own_mono(), Shaping::Advanced);
    buffer.shape_until_scroll(fonts, false);

    let (advance, drawn_in) = buffer
        .layout_runs()
        .flat_map(|run| run.glyphs.iter())
        .map(|glyph| (glyph.w, glyph.font_id))
        .next()
        .ok_or(NothingToDrawWith::TheFaceIsMissing)?;
    if !the_shaper_used_one_of(&ours, drawn_in) {
        return Err(NothingToDrawWith::TheFaceWasNotUsed);
    }
    if advance <= 0.0 {
        return Err(NothingToDrawWith::TheresNoRoom);
    }
    Ok(ACell { advance })
}

/// Where the terminal's window sits on a display this size, and its transcript.
///
/// Answers the window and the transcript inside it, both in this display's
/// pixels. Every figure is either a share of the display or a logical measure
/// put through `scale`.
pub(crate) fn where_the_window_goes(
    size: (i32, i32),
    scale: f64,
) -> Option<(Rectangle<i32, Physical>, Rectangle<i32, Physical>)> {
    let (width, height) = size;
    if width <= 0 || height <= 0 || scale <= 0.0 {
        return None;
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a two-digit logical measure through a display's scale is a whole number of \
                  pixels, and a share of a display's own extent is bounded by that extent"
    )]
    let (border, strip, note, under, window_width, window_height) = (
        (f64::from(measures::A_BORDER) * scale).round() as i32,
        (f64::from(measures::A_STRIP) * scale).round() as i32,
        (f64::from(measures::A_NOTE) * scale).round() as i32,
        (f64::from(measures::BELOW_THE_NOTE) * scale).round() as i32,
        (width as f32 * ACROSS_THE_DISPLAY) as i32,
        (height as f32 * DOWN_THE_DISPLAY) as i32,
    );
    let window = Rectangle::new(
        ((width - window_width) / 2, (height - window_height) / 2).into(),
        (window_width, window_height).into(),
    );

    // The transcript: inside the border, below the identity strip, above the
    // note and the rows under it. The same arithmetic as
    // `measures::transcript_height_in`, which reproduces the design's own 706
    // at a body of 800.
    let transcript_height = window_height - border - strip - note - under;
    let transcript_width = window_width - border * 2;
    if transcript_height <= 0 || transcript_width <= 0 {
        return None;
    }
    let transcript = Rectangle::new(
        (window.loc.x + border, window.loc.y + border + strip).into(),
        (transcript_width, transcript_height).into(),
    );
    Some((window, transcript))
}

/// How many lines fit in this many pixels.
fn lines_in(height: i32, line_height: f32) -> usize {
    if line_height <= 0.0 || height <= 0 {
        return 0;
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a count of lines is bounded by the height it was divided out of"
    )]
    let lines = (height as f32 / line_height) as usize;
    lines
}

/// The grid a terminal window on this display has room for, and the cell to
/// tell the child about.
///
/// **This is the one source of truth for the size**, and the fault it exists to
/// end is three parties each holding their own: the drawing measured the
/// window, the engine's grid was built at 80 by 24, and the child was told 80
/// by 24 once at startup. Measured 2026-10-14: at 1440x1040 the window has room
/// for **117 by 31** and all three believed 80 by 24, so programs broke their
/// lines a third of the way across and anything drawing a full screen drew it
/// at the wrong size.
///
/// The room is the transcript less its insets — the same arithmetic the drawing
/// uses, because a grid worked out any other way would be a fourth answer.
///
/// # Errors
/// [`NothingToDrawWith::TheresNoRoom`] when not one whole cell fits, which is
/// a refusal rather than a clamp: a child told it has no columns divides by
/// zero or draws nothing, and a child told it has one when it has none has
/// been lied to.
pub(crate) fn the_grid_that_fits(
    fonts: &mut FontSystem,
    size: (i32, i32),
    scale: f64,
    remembered: usize,
) -> Result<(alo_terminal::HowBig, alo_terminal::ACellInPixels), NothingToDrawWith> {
    let (_, transcript) =
        where_the_window_goes(size, scale).ok_or(NothingToDrawWith::TheresNoRoom)?;
    let metrics = the_designs_metrics(scale);
    let cell = how_big_a_cell_is(fonts, metrics)?;

    #[expect(
        clippy::cast_possible_truncation,
        reason = "as elsewhere in this file: a two-digit logical measure through a scale"
    )]
    let (inset_across, inset_down) = (
        (f64::from(measures::AN_INSET) * scale).round() as i32,
        (f64::from(measures::CONTENT_BELOW_THE_TOP) * scale).round() as i32,
    );
    let columns = cell.columns_in(transcript.size.w - inset_across * 2);
    let lines = lines_in(transcript.size.h - inset_down, metrics.line_height);
    let grid = alo_terminal::HowBig::of(columns, lines, remembered)
        .map_err(|_| NothingToDrawWith::TheresNoRoom)?;
    Ok((grid, cell.in_pixels(metrics.line_height)))
}

/// Where the transcript's cells go, on one display.
///
/// **A type rather than four arguments, and it is a thing rather than a call
/// site.** `painted_text::line` carries an exemption for its own argument count
/// with the reason *bundling them into a struct would name the struct after a
/// call site*; this is the case that reason does not cover. The transcript, its
/// insets, the cell and the metrics are **one answer to one question** — where
/// does character *n* of line *m* sit on this display — and both the text and
/// the cursor need exactly that answer. Two callers of one thing is what makes
/// it a thing.
#[derive(Debug, Clone, Copy)]
struct WhereTheCellsGo {
    /// The transcript, in this display's pixels.
    transcript: Rectangle<i32, Physical>,
    /// How far the content is inset from its left and its top.
    inset: (i32, i32),
    /// The cell the face advances by.
    cell: ACell,
    /// The size and line height the cells are drawn at.
    metrics: Metrics,
}

impl WhereTheCellsGo {
    /// The top-left of the cell at this column and line.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a cell's offset inside a transcript is bounded by the transcript, which is \
                  bounded by the display"
    )]
    fn the_cell_at(self, column: usize, line: usize) -> (i32, i32) {
        (
            self.transcript.loc.x + self.inset.0 + self.cell.across(column) as i32,
            self.transcript.loc.y + self.inset.1 + (self.metrics.line_height * line as f32) as i32,
        )
    }

    /// One cell in whole pixels, rounded up so a stem is never cut off.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a cell is two digits of pixels on any display this product lays out for"
    )]
    fn a_cells_box(self) -> (i32, i32) {
        (
            self.cell.advance.ceil() as i32,
            self.metrics.line_height.ceil() as i32,
        )
    }

    /// The room the cells have: the transcript less its insets.
    fn the_room(self) -> Rectangle<i32, Physical> {
        Rectangle::new(
            (
                self.transcript.loc.x + self.inset.0,
                self.transcript.loc.y + self.inset.1,
            )
                .into(),
            (
                self.transcript.size.w - self.inset.0 * 2,
                self.transcript.size.h - self.inset.1,
            )
                .into(),
        )
    }
}

/// One run of cells, drawn in its own colours.
///
/// **A buffer per run rather than per screen or per cell.** Per screen cannot
/// carry two colours; per cell is a buffer and a rounding for every character
/// on the display. A run is the unit the grid already groups by, and it is what
/// every terminal shapes.
///
/// Answers the run's ground, where it has one, and its ink — the ground first
/// because it is painted under the characters.
fn a_run(
    fonts: &mut FontSystem,
    run: &alo_terminal::ACellRun,
    goes: WhereTheCellsGo,
    line: usize,
    default: ([u8; 3], [u8; 3]),
) -> (Option<Solid>, Option<Inked>) {
    let (ground, ink) = default;
    let its_ink = a_colour(run.looks.ink, ink);
    let its_ground = a_colour(run.looks.ground, ground);
    let (across, down) = goes.the_cell_at(run.from_column, line);
    let (cell_wide, tall) = goes.a_cells_box();
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a run's width is a count of cells times a cell, both bounded by the transcript"
    )]
    let wide = (goes.cell.advance * run.text.chars().count() as f32).ceil() as i32;
    if wide <= 0 || tall <= 0 || cell_wide <= 0 {
        return (None, None);
    }
    let whole = Rectangle::new((across, down).into(), (wide, tall).into());
    // **A run reaching past the transcript draws nothing.** True for a frame
    // while a window is shrinking, before the grid has been laid out again —
    // and a rectangle outside the output is what `validate` refuses, so this is
    // what stops a true state becoming a refused frame.
    if goes.the_room().intersection(whole) != Some(whole) {
        return (None, None);
    }

    // **Only a run with its own ground gets a solid.** A run in the default
    // ground already has the transcript's, painted once behind everything.
    let behind = (run.looks.ground != alo_terminal::what_a_cell_looks_like::AColour::TheDefault)
        .then_some(Solid {
            area: whole,
            colour: its_ground,
        });

    let mut buffer = Buffer::new(fonts, goes.metrics);
    buffer.set_wrap(fonts, Wrap::None);
    buffer.set_size(fonts, None, None);
    buffer.set_text(
        fonts,
        &run.text,
        &alos_own_mono_at(run.looks),
        Shaping::Advanced,
    );
    buffer.shape_until_scroll(fonts, false);
    let shaped = inked(fonts, &buffer, (wide, tall), 0, its_ground, its_ink);
    let bottom = goes.transcript.loc.y + goes.transcript.size.h;
    (behind, shaped.placed(across, down, bottom))
}

/// A colour a cell asked for, or the one this machine uses when it asked for
/// nothing.
///
/// **The default pair is the shell's**, which is why `alo-terminal` answers
/// `TheDefault` rather than a value: a program asking for *the* foreground
/// colour is asking for whatever this terminal uses, and that is the design's
/// `text/primary` on `bg/surface`.
fn a_colour(asked: alo_terminal::what_a_cell_looks_like::AColour, default: [u8; 3]) -> [u8; 3] {
    match asked {
        alo_terminal::what_a_cell_looks_like::AColour::TheDefault => default,
        alo_terminal::what_a_cell_looks_like::AColour::Exactly { red, green, blue } => {
            [red, green, blue]
        }
    }
}

/// The cursor, drawn over the cell it sits on.
///
/// **A block cursor has to put the character back**, and that is the whole of
/// why this is more than one rectangle. Filling the cell hides what is there,
/// so the character is shaped again in the **ground** colour and drawn on top —
/// which is what inverting a cell means, and what every terminal does. Without
/// it, the letter a person is sitting on disappears.
///
/// **No blink.** A blinking cursor needs a clock, and nothing in this drawing
/// has one: a frame is drawn when something changed, not on a timer. A cursor
/// that blinked would need the compositor to redraw on its own account, which
/// is a decision about frames and not about terminals. The program's `blinking`
/// flag is read and ignored, and this says so rather than leaving a reader to
/// wonder.
///
/// Answers the shapes and the re-inked character, or nothing when the cursor is
/// outside the room the transcript has — which happens for one frame while a
/// window is shrinking, before the grid has been laid out again.
fn the_cursor(
    fonts: &mut FontSystem,
    screen: &TerminalOutput,
    at: alo_terminal::WhereTheCursorIs,
    goes: WhereTheCellsGo,
    ground: [u8; 3],
    ink: [u8; 3],
) -> (Vec<Solid>, Vec<Inked>) {
    let (across, down) = goes.the_cell_at(at.column, at.line);
    let (wide, tall) = goes.a_cells_box();
    if wide <= 0 || tall <= 0 {
        return (Vec::new(), Vec::new());
    }
    // **Outside the transcript draws nothing.** True for one frame while a
    // window is shrinking, before the grid has been laid out again — and a
    // rectangle outside the output is what `validate` refuses, so this is the
    // check that stops a true state becoming a refused frame.
    let whole = Rectangle::new((across, down).into(), (wide, tall).into());
    if goes.the_room().intersection(whole) != Some(whole) {
        return (Vec::new(), Vec::new());
    }

    // **The thickness of a line cursor is a measure, not a pixel.** A beam two
    // pixels wide on a dense screen is a beam nobody can see, so it is the
    // design's own border scaled the same way everything else is — and at least
    // one, because a cursor rounded away is a cursor that is not there.
    let thin = (tall / 12).max(1);
    let shapes = match at.shape {
        alo_terminal::ACursorShape::Block => vec![Solid {
            area: whole,
            colour: ink,
        }],
        alo_terminal::ACursorShape::Underline => vec![Solid {
            area: Rectangle::new((across, down + tall - thin).into(), (wide, thin).into()),
            colour: ink,
        }],
        alo_terminal::ACursorShape::Beam => vec![Solid {
            area: Rectangle::new((across, down).into(), (thin, tall).into()),
            colour: ink,
        }],
        // Four edges rather than a fill, which is what makes it hollow.
        alo_terminal::ACursorShape::HollowBlock => vec![
            Solid {
                area: Rectangle::new((across, down).into(), (wide, thin).into()),
                colour: ink,
            },
            Solid {
                area: Rectangle::new((across, down + tall - thin).into(), (wide, thin).into()),
                colour: ink,
            },
            Solid {
                area: Rectangle::new((across, down).into(), (thin, tall).into()),
                colour: ink,
            },
            Solid {
                area: Rectangle::new((across + wide - thin, down).into(), (thin, tall).into()),
                colour: ink,
            },
        ],
    };

    // **Only a filled block hides the character**, so only a filled block puts
    // it back. An underline and a beam sit beside it.
    if at.shape != alo_terminal::ACursorShape::Block {
        return (shapes, Vec::new());
    }
    let Some(character) = screen.the_character_under(at) else {
        return (shapes, Vec::new());
    };
    if character == ' ' {
        // Nothing to put back, which is the ordinary case: the cursor usually
        // sits after the last character of a line.
        return (shapes, Vec::new());
    }
    let mut buffer = Buffer::new(fonts, goes.metrics);
    buffer.set_wrap(fonts, Wrap::None);
    buffer.set_size(fonts, None, None);
    buffer.set_text(
        fonts,
        &character.to_string(),
        &alos_own_mono(),
        Shaping::Advanced,
    );
    buffer.shape_until_scroll(fonts, false);
    // **Ground on ink**, the two swapped, which is the inversion.
    let shaped = inked(fonts, &buffer, (wide, tall), 0, ink, ground);
    (
        shapes,
        shaped
            .placed(across, down, goes.transcript.loc.y + goes.transcript.size.h)
            .into_iter()
            .collect(),
    )
}

/// Draw alo's own terminal onto a display this size.
///
/// # Errors
/// [`NothingToDrawWith`] when the face is absent, was not used, or the display
/// has no room for a terminal.
pub(crate) fn picture(
    fonts: &mut FontSystem,
    screen: &TerminalOutput,
    size: (i32, i32),
    scale: f64,
    scheme: Scheme,
) -> Result<TerminalPicture, NothingToDrawWith> {
    let (window, transcript) =
        where_the_window_goes(size, scale).ok_or(NothingToDrawWith::TheresNoRoom)?;
    let metrics = the_designs_metrics(scale);
    let cell = how_big_a_cell_is(fonts, metrics)?;

    let (ground, ink) = match scheme {
        Scheme::Light => (
            alo_appearance::Token::Cream.colour(),
            alo_appearance::Token::Navy.colour(),
        ),
        Scheme::Dark => (
            alo_appearance::Token::Charcoal.colour(),
            alo_appearance::Token::Cream.colour(),
        ),
    };
    let ground = [ground.red(), ground.green(), ground.blue()];
    let ink = [ink.red(), ink.green(), ink.blue()];

    // The content is inset from the transcript's own edges, both figures the
    // design's and both measured — `AN_INSET` at two different widths.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "as above: a two-digit logical measure through a display's scale"
    )]
    let (inset_across, inset_down) = (
        (f64::from(measures::AN_INSET) * scale).round() as i32,
        (f64::from(measures::CONTENT_BELOW_THE_TOP) * scale).round() as i32,
    );
    let room_across = transcript.size.w - inset_across * 2;
    let room_down = transcript.size.h - inset_down;
    if room_across <= 0 || room_down <= 0 || cell.columns_in(room_across) == 0 {
        return Err(NothingToDrawWith::TheresNoRoom);
    }

    // **A buffer per run of cells that share an appearance**, which is what
    // lets a line hold two colours at all. One buffer for the screen could
    // not; one per cell would be a buffer and a rounding for every character
    // on the display. The grid already groups by run, and a run is what every
    // terminal shapes.
    //
    // **This replaced one buffer for the whole screen**, which joined the lines
    // with `\n` and inked them in a single colour — so `cargo test` had no red
    // or green and a diff was unreadable.
    let goes = WhereTheCellsGo {
        transcript,
        inset: (inset_across, inset_down),
        cell,
        metrics,
    };
    let mut behind: Vec<Solid> = Vec::new();
    let mut placed: Vec<Inked> = Vec::new();
    for line in 0..screen.size().lines() {
        for run in screen.runs_on(line) {
            let (its_ground, its_ink) = a_run(fonts, &run, goes, line, (ground, ink));
            behind.extend(its_ground);
            placed.extend(its_ink);
        }
    }

    // The window's ground, then the transcript's. Two solids rather than one,
    // because the identity strip and the note between them are a later change
    // and will need the window's ground behind them.
    let mut solids = vec![
        Solid {
            area: window,
            colour: ground,
        },
        Solid {
            area: transcript,
            colour: ground,
        },
    ];
    let mut cells: Vec<Inked> = placed;
    // **Each run's own ground goes under the cells**, after the transcript's
    // and before the characters.
    solids.extend(behind);

    // **The cursor last**, because it is drawn *over* the cell it sits on —
    // both the shape and, for a block, the character put back on top of it.
    // Nothing is drawn when the program has hidden it, which every full-screen
    // program does on every redraw.
    if let Some(at) = screen.where_the_cursor_is() {
        let (shapes, put_back) = the_cursor(fonts, screen, at, goes, ground, ink);
        solids.extend(shapes);
        cells.extend(put_back);
    }
    Ok(TerminalPicture {
        size,
        solids,
        inked: cells,
    })
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_terminals_cells_are_drawn_tests.rs"]
mod tests;
