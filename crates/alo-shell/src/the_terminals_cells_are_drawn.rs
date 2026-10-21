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
use crate::painted_text::{ALOS_OWN_MONO, alos_own_mono, alos_own_mono_at, alos_own_text, inked};

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

/// The design's `Label/Medium`, on this display.
///
/// Manrope SemiBold **13** on a line of **18**, where the transcript is Geist
/// Mono 14 on 22. Separate from [`the_designs_metrics`] because they are
/// different type styles answering different questions, and a label drawn at
/// the transcript's size is a label that reads as transcript.
fn the_labels_metrics(scale: f64) -> Metrics {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a two-digit logical measure through a display's scale is a size in pixels"
    )]
    let metrics = Metrics::new(
        (f64::from(measures::THE_LABEL) * scale) as f32,
        (f64::from(measures::THE_LABELS_LINE) * scale) as f32,
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

/// The note under the transcript, saying what this terminal is.
///
/// The design's `Task scope note` — `zsh · UTF-8`, the shell and the encoding —
/// measured off node `479:29034`: the band is 1038×28 at the body's row 763,
/// and the text sits at **(24, 0)** inside it, which is the transcript's own
/// inset and the **top** of the band rather than its middle.
///
/// Drawn in `Label/Small`: Manrope SemiBold 11 on a line of 16, in
/// `text/muted`. Three sizes for three jobs — the transcript is what a program
/// said, the gutter says who said it, and this says what the terminal is. A
/// person reads them in that order.
///
/// **No vocabulary entry, and that is deliberate.** `zsh · UTF-8` holds no
/// English: a program's own name, a separator, and the name of an encoding.
/// `CLAUDE.md` makes hardcoded English a bug, and there is none here to
/// externalise — a translator handed `UTF-8` would have nothing to do with it.
///
/// Answers [`None`] where there is no room or no name to give.
fn the_note_under_the_transcript(
    fonts: &mut FontSystem,
    said: &str,
    window: Rectangle<i32, Physical>,
    scale: f64,
    colours: ([u8; 3], [u8; 3]),
) -> Option<Inked> {
    let (ground, ink) = colours;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "two-digit logical measures through a display's scale"
    )]
    let (inset, from_the_top, tall, size, line) = (
        (f64::from(measures::AN_INSET) * scale).round() as i32,
        (f64::from(measures::BELOW_THE_NOTE) * scale).round() as i32,
        (f64::from(measures::A_NOTE) * scale).round() as i32,
        (f64::from(measures::THE_NOTE) * scale) as f32,
        (f64::from(measures::THE_NOTES_LINE) * scale) as f32,
    );
    if tall <= 0 || size <= 0.0 || line <= 0.0 {
        return None;
    }

    // The band's own top: the window, less the rows under the note, less the
    // note. `measures::the_note_begins_at` is the same arithmetic on a body,
    // and reproduces the design's 763 at 800.
    let top = window.loc.y + window.size.h - from_the_top - tall;
    let room = window.size.w - inset * 2;
    if room <= 0 {
        return None;
    }

    let mut buffer = Buffer::new(fonts, Metrics::new(size, line));
    buffer.set_wrap(fonts, Wrap::None);
    buffer.set_size(fonts, None, None);
    buffer.set_text(fonts, said, &alos_own_text(), Shaping::Advanced);
    buffer.shape_until_scroll(fonts, false);
    let shaped = inked(fonts, &buffer, (room, tall), 0, ground, ink);
    shaped.placed(window.loc.x + inset, top, window.loc.y + window.size.h)
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
    #[expect(
        clippy::cast_possible_truncation,
        reason = "as elsewhere in this file: a two-digit logical measure through a scale"
    )]
    let gutter = (f64::from(measures::A_GUTTER) * scale).round() as i32;
    // **The columns are counted in the program's room, not the region's.** The
    // gutter is the terminal's, so a grid sized to the whole region would hand
    // the program columns that land in it — and the drawing would then refuse
    // them one frame at a time rather than never offering them.
    let columns = cell.columns_in(transcript.size.w - inset_across * 2 - gutter);
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
    /// The terminal's own column, between the inset and the first cell.
    ///
    /// **Room taken from the cells before they are laid out**, never an
    /// overlay drawn on top of them — the design's annotation says the gutter
    /// is *outside clipped program content*, and ADR 0113 makes that the
    /// difference between a promise and a decoration.
    gutter: i32,
    /// The cell the face advances by.
    cell: ACell,
    /// The size and line height the cells are drawn at.
    metrics: Metrics,
    /// This display's scale, which the gutter's own measures go through.
    scale: f64,
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
            self.transcript.loc.x + self.inset.0 + self.gutter + self.cell.across(column) as i32,
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

    /// The room the cells have: the transcript, less its insets, less the
    /// terminal's own gutter.
    ///
    /// **The gutter is taken off both the start and the width**, so a cell
    /// cannot reach it from either side. `a_run` refuses anything that is not
    /// wholly inside this rectangle, which is what makes *no program writes in
    /// the gutter* a thing the drawing cannot do rather than a thing it
    /// chooses not to.
    fn the_room(self) -> Rectangle<i32, Physical> {
        Rectangle::new(
            (
                self.transcript.loc.x + self.inset.0 + self.gutter,
                self.transcript.loc.y + self.inset.1,
            )
                .into(),
            (
                self.transcript.size.w - self.inset.0 * 2 - self.gutter,
                self.transcript.size.h - self.inset.1,
            )
                .into(),
        )
    }

    /// A logical measure in this display's pixels.
    ///
    /// **The one road a figure from `alo_terminal::measures` takes to a
    /// screen.** `CLAUDE.md`: a measure is declared once with what it answers
    /// to and converted for each display through that display's scale, and a
    /// figure that reaches a display without passing through here is the fault
    /// that rule names.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a logical measure of this terminal is three digits at most, and a scale is \
                  small and positive on every display this product lays out for"
    )]
    fn scaled(self, measure: u32) -> i32 {
        (f64::from(measure) * self.scale).round() as i32
    }

    /// The type the gutter's label is drawn at, on this display.
    fn label_metrics(self) -> Metrics {
        the_labels_metrics(self.scale)
    }

    /// Where the gutter's label goes for a region starting at this line.
    ///
    /// The region's own left edge — the inset, before the gutter — and the top
    /// of the line it labels. The design puts the label text node at (0, 0) of
    /// the region, which is this point.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a line's offset inside a transcript is bounded by the transcript"
    )]
    fn the_label_at(self, line: usize) -> (i32, i32) {
        (
            self.transcript.loc.x + self.inset.0,
            self.transcript.loc.y + self.inset.1 + (self.metrics.line_height * line as f32) as i32,
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

/// The gutter's label for one region, drawn in alo's own face.
///
/// **The one place in the terminal a program cannot write**
/// ([ADR 0113](../../../docs/decisions/0113-the-terminal-owns-a-gutter-that-says-who-is-speaking.md)),
/// so what it says is decided by [`alo_terminal::WhoIsSpeaking`] and never by
/// anything on the screen. The word arrives here already chosen; this function
/// cannot see a single byte the program wrote, which is the guarantee expressed
/// as a signature rather than as a comment.
///
/// **Drawn in alo's own face at the design's `Label/Medium`** — Manrope
/// SemiBold 13 on a line of 18 — where the transcript is Geist Mono 14. Two
/// families and two sizes, so the difference is visible before a word is read.
/// That is ADR 0113's *distinct boundary shape*, and the words carry it in any
/// case: *colour is supplementary*, in the design's own annotation.
///
/// Answers [`None`] where the label would not fit wholly inside the gutter,
/// for the same reason a run reaching past the transcript draws nothing.
fn a_regions_label(
    fonts: &mut FontSystem,
    said: &str,
    goes: WhereTheCellsGo,
    line: usize,
    colours: ([u8; 3], [u8; 3]),
) -> Option<Inked> {
    let (ground, ink) = colours;
    let (across, down) = goes.the_label_at(line);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "two logical measures through a display's scale, each two digits"
    )]
    let (room, tall) = (
        (f64::from(measures::THE_LABELS_ROOM) * goes.scale).round() as i32,
        goes.metrics.line_height.ceil() as i32,
    );
    if room <= 0 || tall <= 0 {
        return None;
    }

    let mut buffer = Buffer::new(fonts, goes.label_metrics());
    buffer.set_wrap(fonts, Wrap::None);
    buffer.set_size(fonts, None, None);
    buffer.set_text(fonts, said, &alos_own_text(), Shaping::Advanced);
    buffer.shape_until_scroll(fonts, false);
    let shaped = inked(fonts, &buffer, (room, tall), 0, ground, ink);
    let bottom = goes.transcript.loc.y + goes.transcript.size.h;
    shaped.placed(across, down, bottom)
}

/// How many transcript lines one benchmark row takes.
///
/// The design's row is 40 tall and a transcript line is 22, so two lines is 44
/// — the closest the line grid comes to the design's row, and the alternative
/// was a 16-tall bar crowded into a 22-tall line. **A deliberate fit, not a
/// measurement**, recorded here because the next person will compare it with
/// `479:28967` and find 40.
const A_BENCHMARKS_ROW_IN_LINES: usize = 2;

/// One row of a benchmark: its label, its bar and its value.
///
/// The design's rule, from `docs/design/the-terminal-window.md`:
///
/// ```text
/// label(110) | 16 | bar(value x scale) | 16 | value(110)
/// ```
///
/// **The value follows the bar and is not at a fixed place.** A value pinned
/// where the design's own data put it would be right for 240 ms and wrong for
/// every other number, and would look right in review.
fn a_benchmarks_row(
    fonts: &mut FontSystem,
    run: &alo_terminal::ARun,
    longest: u64,
    goes: WhereTheCellsGo,
    line: usize,
    colours: ([u8; 3], [u8; 3]),
) -> (Vec<Inked>, Option<Solid>) {
    let (ground, ink) = colours;
    let scaled = |measure: u32| goes.scaled(measure);
    let (across, down) = goes.the_cell_at(0, line);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a line height is two digits of pixels on any display this product lays out for"
    )]
    let a_line = goes.metrics.line_height.ceil() as i32;
    let tall = a_line.saturating_mul(i32::try_from(A_BENCHMARKS_ROW_IN_LINES).unwrap_or(1));
    let label_wide = scaled(alo_terminal::measures::A_BENCHMARKS_LABEL);
    let value_wide = scaled(alo_terminal::measures::A_BENCHMARKS_VALUE);
    let gutter = scaled(alo_terminal::measures::A_BENCHMARKS_GUTTER);
    let room = goes.transcript.loc.x + goes.transcript.size.w - goes.inset.0 - across;

    // **The track is what the row has left**, which is why it is not a measure:
    // it depends on the display, not on the design.
    let track = room
        .saturating_sub(label_wide)
        .saturating_sub(value_wide)
        .saturating_sub(gutter.saturating_mul(2));
    if room <= 0 || tall <= 0 || track <= 0 {
        return (Vec::new(), None);
    }

    let mut pieces = Vec::new();
    let bottom = goes.transcript.loc.y + goes.transcript.size.h;
    let text_at = |fonts: &mut FontSystem, what: &str, x: i32, wide: i32| -> Option<Inked> {
        let mut buffer = Buffer::new(fonts, goes.label_metrics());
        buffer.set_wrap(fonts, Wrap::None);
        buffer.set_size(fonts, None, None);
        buffer.set_text(fonts, what, &alos_own_text(), Shaping::Advanced);
        buffer.shape_until_scroll(fonts, false);
        let shaped = inked(fonts, &buffer, (wide, a_line), 0, ground, ink);
        shaped.placed(x, down, bottom)
    };

    if let Some(label) = text_at(fonts, run.it_said_called(), across, label_wide) {
        pieces.push(label);
    }

    let bar_wide = i32::try_from(alo_terminal::measures::a_bars_length(
        run.it_said_microseconds(),
        longest,
        u32::try_from(track).unwrap_or(0),
    ))
    .unwrap_or(0);
    let bar_tall = scaled(alo_terminal::measures::A_BENCHMARKS_BAR);
    let bar_at = across.saturating_add(label_wide).saturating_add(gutter);
    let bar_down = down.saturating_add((tall.saturating_sub(bar_tall)) / 2);
    let bar = (bar_wide > 0 && bar_tall > 0).then(|| Solid {
        area: Rectangle::new((bar_at, bar_down).into(), (bar_wide, bar_tall).into()),
        colour: ink,
    });

    // **The value follows the bar**, one gutter after where it ends.
    let value_at = bar_at.saturating_add(bar_wide).saturating_add(gutter);
    let said = a_duration_in_words(run.it_said_microseconds());
    if let Some(value) = text_at(fonts, &said, value_at, value_wide) {
        pieces.push(value);
    }

    (pieces, bar)
}

/// A duration as the design writes one: a whole number and `ms`.
///
/// **Not a vocabulary string.** `ms` is the symbol for a millisecond and is the
/// same in every language this ships in, the way `·` is — and the number is a
/// number. A sentence *about* the duration would be a string; this is a
/// quantity with its unit.
fn a_duration_in_words(microseconds: u64) -> String {
    format!("{} ms", microseconds / 1_000)
}

/// One line alo said, drawn where the transcript's own text goes.
///
/// **Not [`a_regions_label`], which drew these until 2026-10-17.** That one
/// places at `WhereTheCellsGo::the_label_at` and crops to the label's room, so
/// alo's sentence was drawn *inside the gutter's 80-pixel box* and then painted
/// over by the gutter word for the same region. The picture held both runs, at
/// one rectangle, and a person pressing `Ctrl`+`Alt`+`W` saw `alo` beside
/// nothing. Every check of it counted runs and passed.
///
/// Found by drawing the terminal on a compositor and reading the pixels back —
/// `crates/alo-shell/examples/support/a_terminal_check.rs` — and held here by
/// `saying_something_draws_the_sentence_and_not_only_its_gutter_word`, which
/// asserts no two inked runs share a rectangle.
fn alos_own_line(
    fonts: &mut FontSystem,
    said: &str,
    goes: WhereTheCellsGo,
    line: usize,
    colours: ([u8; 3], [u8; 3]),
) -> Option<Inked> {
    shaped_into_alos_row(fonts, said, goes, line, colours, Wrap::None, 1).map(|(drawn, _)| drawn)
}

/// One of alo's own lines, **wrapped at the room it has**, and how many rows it
/// took.
///
/// **Only alo's remark uses this, and the reason is layout rather than taste.**
/// Everything else drawn through [`alos_own_line`] — a diff's rows, a
/// benchmark's, a result's summary and its provenance — is one row per item by
/// construction, and each of those callers works out its own row from a count it
/// already holds. A line that silently became two would be drawn over the next
/// one. A model's prose has no such bound, so it is the one that must wrap, and
/// its caller is the one written to advance by rows.
fn alos_own_wrapped_line(
    fonts: &mut FontSystem,
    said: &str,
    goes: WhereTheCellsGo,
    line: usize,
    colours: ([u8; 3], [u8; 3]),
    at_most: usize,
) -> Option<(Inked, usize)> {
    shaped_into_alos_row(fonts, said, goes, line, colours, Wrap::Word, at_most)
}

/// alo's own line, shaped and inked, answering what it drew and how many rows
/// it took.
///
/// **Until 2026-10-22 there was no wrapping here at all.** The buffer was laid
/// out unbounded and `inked` cropped whatever overran `room`, so a long
/// explanation was cut off **mid-word with its tail lost** — not wrapped, not
/// scrolled, not marked as shortened. Measured on the nested terminal fixture at
/// 1440x960: 137 characters drew whole, and 286 drew as
/// `...so nothing can be restored afterwar` and stopped, losing *it can reach
/// files that are not yours*, which is the half a person deciding whether to
/// press Enter most needs.
fn shaped_into_alos_row(
    fonts: &mut FontSystem,
    said: &str,
    goes: WhereTheCellsGo,
    line: usize,
    colours: ([u8; 3], [u8; 3]),
    wrapping: Wrap,
    at_most: usize,
) -> Option<(Inked, usize)> {
    let (ground, ink) = colours;
    let (across, down) = goes.the_cell_at(0, line);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a line height is two digits of pixels on any display this product lays out for"
    )]
    let tall = goes.metrics.line_height.ceil() as i32;
    // **What the transcript leaves after the gutter**, rather than a measure of
    // its own: the sentence starts where the cells start and runs to the
    // transcript's far inset, so it is the same room a line of output has.
    //
    // **It is also what a wrapped line wraps to**, which is why this stays a
    // measure read off the display's own rectangle rather than a figure
    // somebody chose: a wrapped sentence then wraps to the screen it is on.
    let room = goes.transcript.loc.x + goes.transcript.size.w - goes.inset.0 - across;
    if room <= 0 || tall <= 0 {
        return None;
    }

    let mut buffer = Buffer::new(fonts, goes.label_metrics());
    buffer.set_wrap(fonts, wrapping);
    #[expect(
        clippy::cast_precision_loss,
        reason = "a width in pixels is four digits at most on any display this product lays out for"
    )]
    let bounded = if matches!(wrapping, Wrap::None) {
        None
    } else {
        Some(room as f32)
    };
    // **Bounded in height as well as width**, so the buffer does not lay out
    // rows it has no room for. Clipping them in `inked` instead left a
    // **one-pixel smear** of the next row's ascenders — measured at y=737 on a
    // 1440x960 frame, where the row below began at 741 — and that smear was
    // enough to make a fixture counting rows of ink believe a line had wrapped.
    #[expect(
        clippy::cast_precision_loss,
        reason = "a height in pixels is four digits at most on any display this product lays out for"
    )]
    let ceiling =
        bounded.map(|_| tall.saturating_mul(i32::try_from(at_most.max(1)).unwrap_or(1)) as f32);
    buffer.set_size(fonts, bounded, ceiling);
    buffer.set_text(fonts, said, &alos_own_text(), Shaping::Advanced);
    buffer.shape_until_scroll(fonts, false);
    // **How many rows it really took**, asked of the shaped buffer rather than
    // guessed from the text's length: the caller advances by this, and the
    // gutter's own region is counted from it, so a wrap nobody predicted would
    // caption the wrong row.
    // **At most the rows it was given.** A wrapped sentence can want more rows
    // than alo's block has left, and the lines beneath it — each mark, and the
    // disclaimer — are the ones that must not be the casualty:
    // `WhatACommandWillDo::and_not_a_guarantee` has no shape that omits it, so a
    // drawing that dropped it would break an invariant the type states.
    //
    // What this costs is named rather than hidden: prose longer than the room
    // **is still shortened, and still says nothing about it** — but it is now cut
    // at a word rather than mid-word, and the marks below it survive. What a
    // too-tall explanation should really do is a question
    // `docs/design/the-terminal-window.md` leaves open, in the same words it
    // uses for a benchmark bar that overruns: *left open deliberately; this is
    // exactly the sort of thing that gets invented once and then cited as the
    // design.*
    let rows = buffer.layout_runs().count().clamp(1, at_most.max(1));
    let shaped = inked(
        fonts,
        &buffer,
        (room, tall.saturating_mul(i32::try_from(rows).unwrap_or(1))),
        0,
        ground,
        ink,
    );
    let bottom = goes.transcript.loc.y + goes.transcript.size.h;
    shaped
        .placed(across, down, bottom)
        .map(|drawn| (drawn, rows))
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

/// What a terminal has on it: the program's grid, and anything alo has said
/// beneath it.
///
/// **Two things that arrive together and are drawn together**, grouped because
/// they are one answer to *what is on this terminal* rather than two arguments
/// that happen to be adjacent. `picture` was at clippy's limit of seven and this
/// is the honest way under it: the alternative was passing the terminal itself,
/// which the drawing's own tests cannot build without a pseudoterminal.
#[derive(Clone, Copy)]
pub(crate) struct Shows<'a> {
    /// What the program has written, as a grid of cells.
    pub(crate) screen: &'a TerminalOutput,
    /// What alo has said into this transcript, beneath it.
    pub(crate) what_alo_said: &'a alo_terminal::WhatAloSaid,
    /// A run of tests recognised in what the program printed, if there is one.
    ///
    /// **Drawn in the program's own region and labelled `program`**, not alo's.
    /// The design's annotation on `485-28999` says so outright — *PROGRAM
    /// includes parsed rich results* — and it is the right call: a summary read
    /// out of a stranger's output is still that stranger's claim, which is also
    /// why the line under it says nobody checked it.
    pub(crate) a_test_result: Option<&'a alo_terminal::ATestResult>,
    /// A benchmark comparison recognised in what the program printed.
    ///
    /// **In the program's own region, for the same reason the test result is**
    /// — *PROGRAM includes parsed rich results*. A chart drawn from a
    /// stranger's numbers is still that stranger's claim.
    pub(crate) a_benchmark: Option<&'a alo_terminal::ABenchmark>,
    /// A file diff recognised in what the program printed.
    ///
    /// In the program's own region, under the same rule as the two above.
    pub(crate) a_diff: Option<&'a alo_terminal::ADiff>,
}

/// What joins the parts of a recognised result's summary.
///
/// The design draws `Tests · 23 passed · 1 failed`, so the separator is a
/// middle dot with a space either side, read off the frame. **Punctuation and
/// not language**: it is the same mark in every language this ships in, which
/// is why it is here rather than in a vocabulary.
const BETWEEN_THE_SUMMARYS_PARTS: &str = " · ";

/// Draw alo's own terminal onto a display this size.
///
/// # Errors
/// [`NothingToDrawWith`] when the face is absent, was not used, or the display
/// has no room for a terminal.
pub(crate) fn picture(
    fonts: &mut FontSystem,
    shows: Shows<'_>,
    size: (i32, i32),
    scale: f64,
    scheme: Scheme,
    says: &alo_strings::Strings,
    the_shell_is_called: Option<&str>,
) -> Result<TerminalPicture, NothingToDrawWith> {
    let Shows {
        screen,
        what_alo_said,
        a_test_result,
        a_benchmark,
        a_diff,
    } = shows;
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
    // **The note is `text/muted`**, which the design asks for by name and which
    // `alo-appearance` already holds at the design file's own `#596B78`. Read
    // from the role rather than written here, so the one place that decides
    // what muted means stays the one place.
    let muted = alo_appearance::Role::TextMuted.colour();
    let muted = [muted.red(), muted.green(), muted.blue()];

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
    #[expect(
        clippy::cast_possible_truncation,
        reason = "as above: a two-digit logical measure through a display's scale"
    )]
    let gutter = (f64::from(measures::A_GUTTER) * scale).round() as i32;
    let room_across = transcript.size.w - inset_across * 2 - gutter;
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
        gutter,
        cell,
        metrics,
        scale,
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

    // **The gutter, once per region.** `regions_of` is given a count of lines
    // and nothing else — it cannot see what the program wrote, so a program
    // printing `alo` or `system` changes nothing here. That is ADR 0113's
    // promise, and it is kept by what this call is unable to receive rather
    // than by anything it checks.
    //
    // The word is resolved through the vocabulary, because `CLAUDE.md` makes
    // hardcoded English a bug and these four are read by every person who uses
    // the machine.
    // **A recognised test result, in the program's own region.**
    //
    // Two lines, in the order the design draws them: the summary from the
    // numbers, then the sentence saying nobody checked them. The design's frame
    // `Rich test result / untrusted program data` has a third thing in it — the
    // program's own text at length — and that is already on the screen above,
    // because it is what the program printed and this never replaces it. *Text
    // reveals original bytes*, in the frame's own words.
    //
    // **The ignored count is shown only when there is one.** The design's
    // summary reads `Tests · 23 passed · 1 failed` over a run with none, and a
    // trailing `· 0 ignored` on every result would be three words of noise in
    // the ordinary case.
    let the_results_lines: Vec<String> = a_test_result.map_or_else(Vec::new, |result| {
        let counted = |which: &alo_terminal::words::Counted, how_many: usize| {
            says.count(
                &which.key(),
                &alo_strings::Counting::of(how_many as u64),
                &alo_strings::Filling::nothing(),
            )
            .text()
            .to_owned()
        };
        let mut summary = vec![
            says.say(
                &alo_terminal::words::TESTS.key(),
                &alo_strings::Filling::nothing(),
            )
            .text()
            .to_owned(),
            counted(
                &alo_terminal::words::SO_MANY_PASSED,
                result.it_said_passed(),
            ),
            counted(
                &alo_terminal::words::SO_MANY_FAILED,
                result.it_said_failed(),
            ),
        ];
        if result.it_said_ignored() > 0 {
            summary.push(counted(
                &alo_terminal::words::SO_MANY_IGNORED,
                result.it_said_ignored(),
            ));
        }
        let printed_it = says.say(
            &result.what_printed_it().key(),
            &alo_strings::Filling::nothing(),
        );
        let provenance = says.say(
            &result.and_nobody_checked().key(),
            &alo_strings::Filling::of("what", printed_it.text()),
        );
        vec![
            summary.join(BETWEEN_THE_SUMMARYS_PARTS),
            provenance.text().to_owned(),
        ]
    });
    for (which, line) in the_results_lines.iter().enumerate() {
        let beneath = screen.size().lines() + which;
        if let Some(drawn) = alos_own_line(fonts, line, goes, beneath, (ground, ink)) {
            placed.push(drawn);
        }
    }

    // **A recognised benchmark, in the program's own region**, under the same
    // rule as the test result above and drawn in the order `479:28967` draws
    // it: the summary, the sentence saying nobody checked it, the rows, and the
    // note saying how to read them.
    //
    // **The three `Ask alo / Action` buttons the frame also holds are not
    // drawn.** Their widths disagree between the design's two nodes and what
    // they do is not built, so a button here would look live and reach nothing
    // — which is worse than one that is plainly absent, because somebody would
    // later measure it and conclude the road is broken rather than missing.
    // `docs/design/the-terminal-window.md` records both.
    let a_benchmarks_first_line = screen.size().lines() + the_results_lines.len();
    let how_tall_the_benchmark_is = a_benchmark.map_or(0, |benchmark| {
        let Some(longest) = benchmark.the_longest() else {
            return 0;
        };
        let slowest = longest.it_said_microseconds();
        let fastest = benchmark
            .it_said_runs()
            .iter()
            .min_by_key(|run| run.it_said_microseconds());
        let mut line = a_benchmarks_first_line;

        if let Some(fastest) = fastest
            && slowest > 0
        {
            // **The design's own sample checks this**: 240 ms against 180 ms is
            // 25% lower, and that is the number its summary draws.
            let lower = (slowest - fastest.it_said_microseconds()).saturating_mul(100) / slowest;
            let summary = says.say(
                &alo_terminal::words::LOWER_IN_THIS_COMPARISON.key(),
                &alo_strings::Filling::of("what", fastest.it_said_called())
                    .and("percent", lower.to_string()),
            );
            if let Some(drawn) = alos_own_line(fonts, summary.text(), goes, line, (ground, ink)) {
                placed.push(drawn);
            }
            line += 1;
        }

        let printed_it = says.say(
            &benchmark.what_printed_it().key(),
            &alo_strings::Filling::nothing(),
        );
        let provenance = says.say(
            &benchmark.and_nobody_checked().key(),
            &alo_strings::Filling::of("what", printed_it.text()),
        );
        if let Some(drawn) = alos_own_line(fonts, provenance.text(), goes, line, (ground, ink)) {
            placed.push(drawn);
        }
        line += 1;

        for run in benchmark.it_said_runs() {
            let (pieces, bar) = a_benchmarks_row(fonts, run, slowest, goes, line, (ground, ink));
            placed.extend(pieces);
            behind.extend(bar);
            line += A_BENCHMARKS_ROW_IN_LINES;
        }

        let note = says.say(
            &alo_terminal::words::SAME_LINEAR_SCALE.key(),
            &alo_strings::Filling::nothing(),
        );
        if let Some(drawn) = alos_own_line(fonts, note.text(), goes, line, (ground, ink)) {
            placed.push(drawn);
        }
        line += 1;

        line - a_benchmarks_first_line
    });

    // **A recognised file diff**, in the order `479:28986` draws it: the
    // summary from the counts, the sentence saying nothing is applied, and the
    // body with each line's mark in a gutter left of its number.
    //
    // **The body is drawn a line at a time** where the design draws it as one
    // text node 110 tall. The design's node renders as five rows; these are
    // those rows, in the line grid everything else beneath the transcript uses.
    let a_diffs_first_line = a_benchmarks_first_line + how_tall_the_benchmark_is;
    let how_tall_the_diff_is = a_diff.map_or(0, |diff| {
        let mut line = a_diffs_first_line;
        let counted = |which: &alo_terminal::words::Counted, how_many: usize| {
            says.count(
                &which.key(),
                &alo_strings::Counting::of(how_many as u64),
                &alo_strings::Filling::nothing(),
            )
            .text()
            .to_owned()
        };
        // **The file, then the two counts** — `src/auth.ts · 1 removal, 1
        // addition`. The dot separates the file from the counts and a comma
        // separates the counts, which is what the design draws and is
        // punctuation rather than language.
        let summary = format!(
            "{}{BETWEEN_THE_SUMMARYS_PARTS}{}, {}",
            diff.it_said_file(),
            counted(
                &alo_terminal::words::SO_MANY_REMOVALS,
                diff.it_said_removals()
            ),
            counted(
                &alo_terminal::words::SO_MANY_ADDITIONS,
                diff.it_said_additions()
            ),
        );
        if let Some(drawn) = alos_own_line(fonts, &summary, goes, line, (ground, ink)) {
            placed.push(drawn);
        }
        line += 1;

        let printed_it = says.say(
            &diff.what_printed_it().key(),
            &alo_strings::Filling::nothing(),
        );
        let provenance = says.say(
            &diff.and_nothing_is_applied().key(),
            &alo_strings::Filling::of("what", printed_it.text()),
        );
        if let Some(drawn) = alos_own_line(fonts, provenance.text(), goes, line, (ground, ink)) {
            placed.push(drawn);
        }
        line += 1;

        for its in diff.it_said_lines() {
            let row = format!(
                "{} {}  {}",
                its.it_said_how().the_mark(),
                its.it_said_number(),
                its.it_said_text()
            );
            if let Some(drawn) = alos_own_line(fonts, &row, goes, line, (ground, ink)) {
                placed.push(drawn);
            }
            line += 1;
        }

        line - a_diffs_first_line
    });

    // **alo's own lines, beneath what the program wrote.** Drawn in the
    // transcript's own type at the transcript's own inset, so they read as part
    // of the conversation rather than as chrome bolted beside it. The gutter
    // below labels them `alo`, which is the design's own word for
    // interpretation.
    //
    // **Two kinds of line, and the difference is not cosmetic.** One of alo's
    // own words arrives as a key and is resolved through the vocabulary, because
    // `CLAUDE.md` makes hardcoded English a bug and what is kept is a key rather
    // than a sentence. A model's explanation of a command arrives as prose and
    // is drawn as it came back: there is no key for a sentence nobody here
    // wrote, and looking one up would be claiming authorship of it.
    //
    // `alo_terminal::AloSLine` carries that distinction all the way here rather
    // than flattening to text, which is why this matches rather than resolving
    // everything. An explanation is its words, then each mark, then *an
    // explanation, not a guarantee* — and the count below is the same count the
    // gutter's regions are worked out from, which `what_alo_said_tests.rs`
    // asserts cannot disagree with these lines.
    // **Rows, not lines.** One of alo's lines can wrap, so what the next sits
    // beneath is how many rows the last one really took — and that same running
    // total is what the gutter's region is counted from below.
    //
    // `WhatAloSaid::lines()` still answers how many *lines* alo said, which is
    // what it is for and what its own tests hold it to. It cannot answer rows:
    // that needs a width and a font, and `alo-terminal` has neither.
    // **How many rows alo's block has**, so a wrapped line cannot eat the rows
    // its own marks and disclaimer need. The room is the transcript's, less its
    // inset, in lines — the same arithmetic the grid is sized by.
    let rows_in_all = lines_in(goes.the_room().size.h, goes.metrics.line_height);
    let alos_first_row = a_diffs_first_line + how_tall_the_diff_is;
    let alos_lines = what_alo_said.the_lines();
    let how_many = alos_lines.len();
    let mut alos_rows = 0_usize;
    for (which, line) in alos_lines.into_iter().enumerate() {
        let text = match &line {
            alo_terminal::AloSLine::InOurWords(key) => says
                .say(key, &alo_strings::Filling::nothing())
                .text()
                .to_owned(),
            alo_terminal::AloSLine::AsTheModelSaidIt(prose) => prose.clone(),
        };
        let beneath = alos_first_row + alos_rows;
        // One row kept back for each line still to come, so the last of them —
        // the disclaimer — always has somewhere to go.
        let still_to_come = how_many.saturating_sub(which).saturating_sub(1);
        let at_most = rows_in_all
            .saturating_sub(beneath)
            .saturating_sub(still_to_come)
            .max(1);
        if let Some((drawn, rows)) =
            alos_own_wrapped_line(fonts, &text, goes, beneath, (ground, ink), at_most)
        {
            placed.push(drawn);
            alos_rows = alos_rows.saturating_add(rows);
        } else {
            // A line with no room to draw still takes its row, so the one after
            // it does not climb on top of it.
            alos_rows = alos_rows.saturating_add(1);
        }
    }

    // **The program's region includes the recognised result**, which is the
    // design's own rule and not a convenience: *PROGRAM includes parsed rich
    // results*. A summary drawn in alo's region would label a stranger's
    // arithmetic as alo's interpretation.
    for region in alo_terminal::regions_of(a_diffs_first_line + how_tall_the_diff_is, alos_rows) {
        let said = says.say(
            &region.who.the_word().key(),
            &alo_strings::Filling::nothing(),
        );
        if let Some(label) = a_regions_label(fonts, said.text(), goes, region.from, (ground, ink)) {
            placed.push(label);
        }
    }

    // **The note under the transcript**, which says what this terminal is:
    // the shell actually running on it and the encoding its bytes are read as.
    // Left out rather than guessed at when the kernel will not name the shell
    // — a note naming a shell that is not the one running would be worse than
    // no note.
    if let Some(called) = the_shell_is_called {
        let said = format!("{called} \u{b7} UTF-8");
        if let Some(note) =
            the_note_under_the_transcript(fonts, &said, window, scale, (ground, muted))
        {
            placed.push(note);
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
