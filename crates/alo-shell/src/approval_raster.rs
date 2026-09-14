//! The approval surface laid out and rasterised for one output: a panel with
//! the agent's name, the sentence, and two answers under it.
//!
//! # What is drawn
//!
//! While nothing is open, **nothing**. While a question is open, one panel in
//! the middle of the output: whose question it is, the sentence exactly as
//! `alo-approving` handed it, and the two answers — *no* first in reading
//! order, the answer that changes the machine last, mirrored for a language
//! read right to left. While a refusal is open, the panel holds that sentence
//! and no answer at all.
//!
//! # The sentence is drawn whole or not at all
//!
//! It is wrapped at word boundaries to the panel's width, which changes where
//! its lines break and nothing else: it is never shortened, never given an
//! ellipsis and never cut at the bottom of the output. A panel that cannot
//! hold the whole sentence on this output **refuses the frame**
//! ([`RenderError::ApprovalScene`]), because half a sentence is a different
//! claim from the one the person would be approving.
//!
//! # A selected answer is never told apart by colour alone
//!
//! Neither answer is selected when a question goes up, and the two are then
//! drawn identically. The selected one gets a thicker edge **and** a bar under
//! its words — two shapes — in the same two colours as everything else on the
//! panel, so a person who cannot tell one hue from another can still tell which
//! answer Enter will give.
//!
//! # What it reads
//!
//! `crate::ApprovalShows`, the person's vocabulary for the two answers, and
//! `alo-appearance`'s tokens. Terracotta is not among the colours: it means the
//! agent acting, and this surface is the moment it has not.

use alo_appearance::{Scheme, TextScale, Token};
use alo_strings::{Direction, Strings};
use cosmic_text::Metrics;
use smithay::utils::{Physical, Rectangle};

use crate::approval_answers::Answers;
use crate::painted::{Inked, Solid};
use crate::painted_text::{Shaped, sentence};
use crate::{ApprovalAnswer, ApprovalShows, RenderError, WindowControlLabels};

/// How the approval surface looks, as the person's appearance and language
/// decide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApprovalLook {
    /// Light or dark, as `alo-appearance` decides it.
    pub scheme: Scheme,
    /// The person's text scale, applied to every measure.
    pub scale: TextScale,
    /// Which way the person reads, which decides which side the first answer
    /// is on.
    pub reading: Direction,
}

/// The largest output side, in pixels, the surface is laid out for.
const LARGEST_SIDE: i32 = 16_384;

/// One answer as drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AnswerDrawn {
    /// Which answer it is.
    pub(crate) answer: ApprovalAnswer,
    /// Its whole box, edge included.
    pub(crate) area: Rectangle<i32, Physical>,
    /// The words inked in it, exactly as the vocabulary answered them.
    pub(crate) words: String,
    /// Whether it is the answer Enter gives.
    pub(crate) selected: bool,
}

/// The surface for one output size, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ApprovalPicture {
    /// The output size it was laid out for.
    pub(crate) size: (i32, i32),
    /// The panel, when anything is drawn.
    pub(crate) panel: Option<Rectangle<i32, Physical>>,
    /// The agent's name as drawn, when a question is.
    pub(crate) agent: Option<String>,
    /// The sentence as drawn, and where: the question's, or a refusal's.
    pub(crate) sentence: Option<(String, Rectangle<i32, Physical>)>,
    /// The answers, in the order they are placed across the panel.
    pub(crate) answers: Vec<AnswerDrawn>,
    /// Flat shapes, painted first.
    pub(crate) solids: Vec<Solid>,
    /// Text, painted after the shapes.
    pub(crate) inked: Vec<Inked>,
}

impl ApprovalPicture {
    /// Whether nothing at all is drawn.
    pub(crate) fn is_empty(&self) -> bool {
        self.panel.is_none() && self.solids.is_empty() && self.inked.is_empty()
    }
}

/// Measures scaled by the person's text scale, never below one pixel.
#[derive(Clone, Copy)]
pub(crate) struct Measure {
    /// The scale, in percent.
    percent: i32,
}

impl Measure {
    /// Measures for this scale.
    fn of(scale: TextScale) -> Self {
        Self {
            percent: i32::from(scale.as_percent()),
        }
    }

    /// `at_one` pixels at the ordinary scale, scaled.
    pub(crate) fn px(self, at_one: i32) -> i32 {
        (at_one * self.percent / 100).max(1)
    }

    /// The text metrics every line on the panel uses.
    pub(crate) fn metrics(self) -> Metrics {
        let factor = self.percent as f32 / 100.0;
        Metrics::new(16.0 * factor, 24.0 * factor)
    }
}

/// The panel's two colours for one scheme.
#[derive(Clone, Copy)]
pub(crate) struct Palette {
    /// The panel's ground, inside every edge.
    pub(crate) ground: [u8; 3],
    /// Words, edges and the selection bar.
    pub(crate) ink: [u8; 3],
}

impl Palette {
    /// `alo-appearance`'s tokens for this scheme. Terracotta is not among them.
    pub(crate) fn of(scheme: Scheme) -> Self {
        let rgb = |token: Token| {
            let colour = token.colour();
            [colour.red(), colour.green(), colour.blue()]
        };
        match scheme {
            Scheme::Light => Self {
                ground: rgb(Token::Cream),
                ink: rgb(Token::Navy),
            },
            Scheme::Dark => Self {
                ground: rgb(Token::Charcoal),
                ink: rgb(Token::Cream),
            },
        }
    }
}

/// Lay the surface out for an output of `size` and rasterise it.
///
/// # Errors
/// [`RenderError::ApprovalScene`] when something is open and the output is
/// larger than any this surface is laid out for, too narrow for a panel, or
/// too short to hold the whole sentence and both answers.
pub(crate) fn picture(
    shows: ApprovalShows<'_>,
    strings: &Strings,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
    look: ApprovalLook,
) -> Result<ApprovalPicture, RenderError> {
    let mut drawn = ApprovalPicture {
        size,
        panel: None,
        agent: None,
        sentence: None,
        answers: Vec::new(),
        solids: Vec::new(),
        inked: Vec::new(),
    };
    let (asked, selected, said) = match shows {
        ApprovalShows::Nothing => return Ok(drawn),
        ApprovalShows::Question { asked, selected } => (Some(asked), selected, asked.sentence()),
        ApprovalShows::Refusal(said) => (None, None, said),
    };
    let (width, height) = size;
    if width > LARGEST_SIDE || height > LARGEST_SIDE {
        return Err(RenderError::ApprovalScene);
    }
    let measure = Measure::of(look.scale);
    let palette = Palette::of(look.scheme);
    let margin = measure.px(16);
    let pad = measure.px(20);
    let panel_width = (width - 2 * margin).min(measure.px(560));
    let inner = panel_width - 2 * pad;
    if inner < measure.px(160) {
        return Err(RenderError::ApprovalScene);
    }
    let fonts = &mut labels.fonts;
    let line = |fonts: &mut cosmic_text::FontSystem, text: &str| {
        sentence(
            fonts,
            text,
            inner,
            measure.metrics(),
            palette.ground,
            palette.ink,
        )
    };

    let agent = asked.map(|asked| asked.agent().as_str().to_owned());
    let agent_shaped = agent.as_deref().map(|name| line(fonts, name));
    let sentence_text = said.text().to_owned();
    let sentence_shaped = line(fonts, &sentence_text);
    let answers = match asked {
        Some(asked) => {
            let mut words = Vec::new();
            for answer in ApprovalAnswer::IN_READING_ORDER {
                let said = match answer {
                    ApprovalAnswer::No => asked.no_said(strings),
                    ApprovalAnswer::Approve => asked.approve_said(strings),
                };
                words.push((answer, said.text().to_owned()));
            }
            Some(Answers::laid_out(fonts, words, inner, measure, palette))
        }
        None => None,
    };

    let gap = measure.px(8);
    let agent_height = agent_shaped
        .as_ref()
        .map_or(0, |shaped| shaped.height + gap);
    let answers_height = answers
        .as_ref()
        .map_or(0, |answers| measure.px(20) + answers.height);
    let panel_height = pad + agent_height + sentence_shaped.height + answers_height + pad;
    if panel_height > height - 2 * margin {
        return Err(RenderError::ApprovalScene);
    }
    let left = (width - panel_width) / 2;
    let top = (height - panel_height) / 2;
    let panel = Rectangle::new((left, top).into(), (panel_width, panel_height).into());
    framed(&mut drawn.solids, panel, measure.px(2), palette);
    drawn.panel = Some(panel);

    let mut y = top + pad;
    if let (Some(shaped), Some(name)) = (agent_shaped, agent) {
        let at = shaped.height;
        drawn.inked.extend(placed(shaped, left + pad, y));
        drawn.agent = Some(name);
        y += at + gap;
    }
    let sentence_area = Rectangle::new(
        (left + pad, y).into(),
        (sentence_shaped.width_of_box, sentence_shaped.height).into(),
    );
    y += sentence_shaped.height;
    drawn
        .inked
        .extend(placed(sentence_shaped, left + pad, sentence_area.loc.y));
    drawn.sentence = Some((sentence_text, sentence_area));

    if let Some(answers) = answers {
        y += measure.px(20);
        answers.placed(
            &mut drawn,
            (left + pad, y),
            inner,
            selected,
            look.reading,
            measure,
            palette,
        );
    }
    Ok(drawn)
}

/// Text placed whole. The panel was measured to hold it, so nothing is cut:
/// the limit handed on is the text's own bottom.
pub(crate) fn placed(shaped: Shaped, x: i32, y: i32) -> Option<Inked> {
    let bottom = y + shaped.height;
    shaped.placed(x, y, bottom)
}

/// An edge `edge` pixels thick in ink, and the ground inside it.
pub(crate) fn framed(
    solids: &mut Vec<Solid>,
    area: Rectangle<i32, Physical>,
    edge: i32,
    palette: Palette,
) {
    solids.push(Solid {
        area,
        colour: palette.ink,
    });
    solids.push(Solid {
        area: Rectangle::new(
            (area.loc.x + edge, area.loc.y + edge).into(),
            (area.size.w - 2 * edge, area.size.h - 2 * edge).into(),
        ),
        colour: palette.ground,
    });
}

#[cfg(test)]
#[path = "approval_raster_tests.rs"]
mod tests;
