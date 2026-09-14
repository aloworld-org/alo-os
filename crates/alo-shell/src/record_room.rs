//! Where things go in the record window, and in which colours: the panel's
//! room on one output, the person's text scale, and the two tokens it is drawn
//! in.
//!
//! Apart from `crate::record_raster` because it changes for a different reason:
//! this is how the window looks and is measured, and that file is what goes in
//! it and in which order.

use alo_appearance::{Scheme, TextScale, Token};
use alo_strings::Direction;
use cosmic_text::{FontSystem, Metrics};
use smithay::utils::{Physical, Rectangle};

use crate::RenderError;
use crate::painted::Solid;
use crate::painted_text::{Shaped, sentence};

/// How the record window looks, as the person's appearance and language
/// decide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordLook {
    /// Light or dark, as `alo-appearance` decides it.
    pub scheme: Scheme,
    /// The person's text scale, applied to every measure.
    pub scale: TextScale,
    /// Which way the person reads, which decides which edge the rail is on.
    pub reading: Direction,
}

/// The largest output side, in pixels, the window is laid out for.
const LARGEST_SIDE: i32 = 16_384;

/// Measures scaled by the person's text scale, never below one pixel.
#[derive(Clone, Copy)]
pub(crate) struct Measure {
    /// The scale, in percent.
    percent: i32,
}

impl Measure {
    /// Measures for this scale.
    pub(crate) fn of(scale: TextScale) -> Self {
        Self {
            percent: i32::from(scale.as_percent()),
        }
    }

    /// `at_one` pixels at the ordinary scale, scaled.
    pub(crate) fn px(self, at_one: i32) -> i32 {
        (at_one * self.percent / 100).max(1)
    }

    /// The text metrics every line in the window uses.
    pub(crate) fn metrics(self) -> Metrics {
        let factor = self.percent as f32 / 100.0;
        Metrics::new(16.0 * factor, 24.0 * factor)
    }
}

/// The window's two colours for one scheme.
#[derive(Clone, Copy)]
pub(crate) struct Palette {
    /// The panel's ground.
    pub(crate) ground: [u8; 3],
    /// Words, edges, rules and the rail.
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

/// Where things go inside the panel for one output.
#[derive(Clone, Copy)]
pub(crate) struct Room {
    /// The panel's left edge.
    pub(crate) left: i32,
    /// The panel's width.
    pub(crate) width: i32,
    /// Where text starts across.
    pub(crate) text_x: i32,
    /// How wide text may be.
    pub(crate) inner: i32,
    /// How far an entry's words after its clause are set in.
    pub(crate) indent: i32,
    /// The rail's left edge.
    pub(crate) rail_x: i32,
    /// The rail's width.
    pub(crate) rail: i32,
    /// The panel's inside padding.
    pub(crate) pad: i32,
    /// The space between the output's edge and the panel.
    pub(crate) margin: i32,
    /// The space between two pieces of text.
    pub(crate) gap: i32,
    /// The text measures.
    pub(crate) measure: Measure,
    /// The two colours.
    pub(crate) palette: Palette,
}

impl Room {
    /// The room on an output of `size`, or the refusal for one too small or
    /// larger than any this window is laid out for.
    pub(crate) fn on(size: (i32, i32), look: RecordLook) -> Result<Self, RenderError> {
        let (width, height) = size;
        if width > LARGEST_SIDE || height > LARGEST_SIDE {
            return Err(RenderError::RecordScene);
        }
        let measure = Measure::of(look.scale);
        let margin = measure.px(16);
        let pad = measure.px(20);
        let rail = measure.px(10);
        let rail_gap = measure.px(10);
        let panel_width = (width - 2 * margin).min(measure.px(760));
        let inner = panel_width - 2 * pad - rail - rail_gap;
        if inner < measure.px(200) || height - 2 * margin < measure.px(120) {
            return Err(RenderError::RecordScene);
        }
        let left = (width - panel_width) / 2;
        let (text_x, rail_x) = match look.reading {
            Direction::RightToLeft => (left + pad + rail + rail_gap, left + pad),
            Direction::LeftToRight => (left + pad, left + panel_width - pad - rail),
        };
        Ok(Self {
            left,
            width: panel_width,
            text_x,
            inner,
            indent: measure.px(20),
            rail_x,
            rail,
            pad,
            margin,
            gap: measure.px(6),
            measure,
            palette: Palette::of(look.scheme),
        })
    }

    /// Text shaped at `width`, in the window's type and colours.
    pub(crate) fn shaped(self, fonts: &mut FontSystem, text: &str, width: i32) -> Shaped {
        sentence(
            fonts,
            text,
            width,
            self.measure.metrics(),
            self.palette.ground,
            self.palette.ink,
        )
    }

    /// Where an entry's words after its clause start, and how wide they are:
    /// set in from the side the person starts reading at.
    pub(crate) fn after_box(self, reading: Direction) -> (i32, i32) {
        match reading {
            Direction::RightToLeft => (self.text_x, self.inner - self.indent),
            Direction::LeftToRight => (self.text_x + self.indent, self.inner - self.indent),
        }
    }
}

/// A rule one pixel high across the text, at `y`.
pub(crate) fn rule(solids: &mut Vec<Solid>, room: Room, y: i32) {
    solids.push(Solid {
        area: Rectangle::new((room.text_x, y).into(), (room.inner, 1).into()),
        colour: room.palette.ink,
    });
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
