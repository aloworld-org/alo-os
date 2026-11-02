//! **The question closing the terminal asks**, drawn on the terminal's own
//! window: *A command is still running. Close anyway?*, with *Close anyway*
//! and *Cancel* under it, and a ring round the one the keyboard is on.
//!
//! # What it looks like, and where that came from
//!
//! **The design file draws no such question.** The owner decided what it says
//! and what its buttons do on 2026-11-01; how it looks is a reading, recorded
//! in `docs/design/the-terminal-window.md` so it can be disagreed with: it is
//! drawn as the record window's own confirmation is (`560:763`, approved the
//! same day) — a rounded box in `bg/canvas` with a `border/default` hairline,
//! the question in Body/Medium SemiBold, a filled button for the action and an
//! outlined one for Cancel, 48 tall — so a person meets one kind of *are you
//! sure* on this machine and not two. Every figure is
//! `crate::record_measures`', through the person's text size and the display's
//! scale.
//!
//! **Where:** across the top of the transcript, inset as the transcript's own
//! text is, so it sits over what the command was printing and under the strip
//! that names the folder. It is drawn into the terminal's one raster, so it
//! pans and zooms with the window.
//!
//! # Where a press finds it
//!
//! The two buttons are kept in the raster's pixels with where the raster was
//! drawn ([`WhereTheQuestionIs`]), and a press is carried back through that
//! into the raster before it is tested — the same arithmetic the camera used
//! to put the raster on the screen, run the other way.

use cosmic_text::FontSystem;
use smithay::utils::{Physical, Point, Rectangle};

use crate::closing_the_terminal::OnTheQuestion;
use crate::desktop_look::Measure;
use crate::painted::Washed;
use crate::record_measures as m;
use crate::settings_canvas::{Area, Canvas};
use crate::settings_type::{TheType, written};

/// The colours it is drawn in: the record confirmation's.
#[derive(Debug, Clone, Copy)]
struct ItsColours {
    /// The box's ground, `bg/canvas`.
    ground: [u8; 3],
    /// The hairlines, `border/default`.
    edge: [u8; 3],
    /// The question, the filled button and the ring, `text/primary`.
    ink: [u8; 3],
    /// The outlined button's ground and the filled button's words, `bg/surface`.
    surface: [u8; 3],
}

impl ItsColours {
    /// From the palette's roles.
    fn now() -> Self {
        let rgb = |role: alo_appearance::Role| {
            let it = role.colour();
            [it.red(), it.green(), it.blue()]
        };
        Self {
            ground: rgb(alo_appearance::Role::BgCanvas),
            edge: rgb(alo_appearance::Role::BorderDefault),
            ink: rgb(alo_appearance::Role::TextPrimary),
            surface: rgb(alo_appearance::Role::BgSurface),
        }
    }
}

/// The question, drawn: its raster, in the terminal picture's own pixels,
/// and where its two buttons are in them.
#[derive(Debug, Clone)]
pub(crate) struct TheQuestionDrawn {
    /// The box and what is in it, placed at its own corner in the picture.
    pub(crate) raster: Washed,
    /// *Close anyway*.
    pub(crate) close_anyway: Area,
    /// *Cancel*.
    pub(crate) cancel: Area,
}

/// The words it says, in the person's language.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ItsWords<'a> {
    /// The question.
    pub(crate) question: &'a str,
    /// *Close anyway*.
    pub(crate) close_anyway: &'a str,
    /// *Cancel*.
    pub(crate) cancel: &'a str,
}

impl ItsWords<'_> {
    /// The terminal's own words, from `says`.
    pub(crate) fn the_terminals_own(says: &alo_strings::Strings) -> [String; 3] {
        let say = |word: alo_strings::Word| {
            says.say(&word.key(), &alo_strings::Filling::nothing())
                .text()
                .to_owned()
        };
        [
            say(alo_terminal::words::A_COMMAND_IS_STILL_RUNNING),
            say(alo_terminal::words::CLOSE_ANYWAY),
            say(alo_terminal::words::CANCEL),
        ]
    }
}

/// Draw the question across the top of `transcript`, in the terminal
/// picture's pixels, with the keyboard `on` one button.
///
/// `measure` is the person's text size on this display and `hairline` one
/// logical pixel through the display's scale alone. [`None`] for a transcript
/// with no room across for it.
pub(crate) fn the_question(
    fonts: &mut FontSystem,
    words: ItsWords<'_>,
    on: OnTheQuestion,
    transcript: Area,
    (measure, hairline): (Measure, i32),
) -> Option<TheQuestionDrawn> {
    let px = |at_one: i32| measure.px(at_one);
    let colours = ItsColours::now();
    let (corner, above, side, below, between) = m::THE_CONFIRMATION;
    let (tall, pad, button_corner, buttons_between, buttons_above) = m::ITS_BUTTONS;
    let inset = px(i32::try_from(alo_terminal::measures::AN_INSET).unwrap_or(24));
    let wide = transcript.size.w - 2 * inset;
    let inside = wide - 2 * px(side);
    if inside <= 0 {
        return None;
    }
    let question = written(
        fonts,
        words.question,
        TheType::Name,
        inside,
        measure,
        (colours.ground, colours.ink),
    );
    let label = |fonts: &mut FontSystem, text: &str, (ground, ink): ([u8; 3], [u8; 3])| {
        written(fonts, text, TheType::Label, inside, measure, (ground, ink)).cropped_to_its_ink()
    };
    let anyway = label(fonts, words.close_anyway, (colours.ink, colours.surface));
    let cancel = label(fonts, words.cancel, (colours.surface, colours.ink));
    let height =
        px(above) + question.height + px(between) + px(buttons_above) + px(tall) + px(below);
    // **Room for the keyboard's ring** round the buttons, inside the raster.
    let (ring, off) = m::THE_KEYBOARDS_RING;
    let reach = px(ring) + px(off);
    let card = Rectangle::<i32, Physical>::new(
        (transcript.loc.x + inset, transcript.loc.y + inset).into(),
        (wide, height).into(),
    );
    let mut canvas = Canvas::over(Rectangle::new(
        (card.loc.x - reach, card.loc.y - reach).into(),
        (card.size.w + 2 * reach, card.size.h + 2 * reach).into(),
    ));
    canvas.fill(card, px(corner) as f32, colours.ground, 1.0);
    canvas.border(card, px(corner) as f32, hairline as f32, colours.edge);
    let left = card.loc.x + px(side);
    canvas.words(left, card.loc.y + px(above), &question);
    let row = card.loc.y + px(above) + question.height + px(between) + px(buttons_above);
    let mut x = left;
    let mut a_button = |canvas: &mut Canvas,
                        shaped: &crate::painted_text::Shaped,
                        (fill, edge): ([u8; 3], [u8; 3]),
                        chosen: bool| {
        let it = Rectangle::<i32, Physical>::new(
            (x, row).into(),
            (shaped.width_of_box + 2 * px(pad), px(tall)).into(),
        );
        canvas.fill(it, px(button_corner) as f32, fill, 1.0);
        canvas.border(it, px(button_corner) as f32, hairline as f32, edge);
        canvas.words(x + px(pad), row + (px(tall) - shaped.height) / 2, shaped);
        if chosen {
            let ring_at = Rectangle::<i32, Physical>::new(
                (it.loc.x - reach, it.loc.y - reach).into(),
                (it.size.w + 2 * reach, it.size.h + 2 * reach).into(),
            );
            canvas.border(
                ring_at,
                (px(button_corner) + reach) as f32,
                px(ring) as f32,
                colours.ink,
            );
        }
        x += it.size.w + px(buttons_between);
        it
    };
    let close_anyway = a_button(
        &mut canvas,
        &anyway,
        (colours.ink, colours.ink),
        on == OnTheQuestion::CloseAnyway,
    );
    let cancel = a_button(
        &mut canvas,
        &cancel,
        (colours.surface, colours.edge),
        on == OnTheQuestion::Cancel,
    );
    Some(TheQuestionDrawn {
        raster: canvas.washed(),
        close_anyway,
        cancel,
    })
}

/// Where the question was drawn on the screen, for a press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WhereTheQuestionIs {
    /// Where the terminal's whole picture landed on the display.
    pub(crate) drawn_at: Rectangle<i32, Physical>,
    /// The picture's own size, in its pixels.
    pub(crate) picture: (i32, i32),
    /// *Close anyway*, in the picture's pixels.
    pub(crate) close_anyway: Area,
    /// *Cancel*, in the picture's pixels.
    pub(crate) cancel: Area,
}

impl WhereTheQuestionIs {
    /// The button under `at`, a point on the display, if either is.
    ///
    /// The point is carried back into the picture through the scale the
    /// camera drew it at, so a press finds a button where it is seen at any
    /// pan and zoom.
    pub(crate) fn the_button_under(&self, at: Point<i32, Physical>) -> Option<OnTheQuestion> {
        let (across, down) = self.picture;
        if self.drawn_at.size.w <= 0 || self.drawn_at.size.h <= 0 {
            return None;
        }
        let back = |from: i32, start: i32, seen: i32, picture: i32| -> i32 {
            let it = i64::from(from - start) * i64::from(picture) / i64::from(seen);
            i32::try_from(it).unwrap_or(i32::MIN)
        };
        let point = Point::<i32, Physical>::from((
            back(at.x, self.drawn_at.loc.x, self.drawn_at.size.w, across),
            back(at.y, self.drawn_at.loc.y, self.drawn_at.size.h, down),
        ));
        if self.close_anyway.contains(point) {
            Some(OnTheQuestion::CloseAnyway)
        } else if self.cancel.contains(point) {
            Some(OnTheQuestion::Cancel)
        } else {
            None
        }
    }
}

impl crate::Server {
    /// Where this frame drew the question, or that it drew none.
    pub(crate) fn the_question_was_drawn(&mut self, at: Option<WhereTheQuestionIs>) {
        self.where_the_question_was_drawn = at;
    }

    /// A press on the display at `at`, offered to the question. Answers
    /// whether it took it.
    ///
    /// **Anywhere on the terminal while the question is open is not taken
    /// here**: a press beside the buttons brings the terminal in front as any
    /// press on it does, and the question stays until it is answered.
    pub(crate) fn a_press_on_the_question(&mut self, at: Point<i32, Physical>) -> bool {
        if self.closing_the_terminal.is_none() {
            return false;
        }
        let Some(button) = self
            .where_the_question_was_drawn
            .and_then(|drawn| drawn.the_button_under(at))
        else {
            return false;
        };
        self.the_question_is_answered(button);
        true
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_terminals_question_drawn_tests.rs"]
mod tests;
