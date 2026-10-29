//! The window of what a person can install, drawn.
//!
//! One rounded panel with every shape inside it rasterised into the same
//! texture, and the words inked separately on top — the shape
//! `crate::task_raster` uses, for the reasons it gives.
//!
//! # Everything inside the window is in the raster, and nothing is a `Solid`
//!
//! **`crate::desktop_paint` paints every flat shape before any panel.** A
//! `Solid` pushed here for a card or a chip would be painted and then covered
//! by the window it belongs to. That is not a theory: on 2026-10-11 the Dock's
//! popover drew its badges and its search field exactly that way, and
//! `crate::dock_surface`'s header records what it looked like — *"the marks
//! were drawn, the code was right, and a pixel sample under every badge found
//! nothing at all."*
//!
//! The other half of the same rule is that a rounded corner is **partial
//! coverage**: the pixel at a corner is some of the card and some of what is
//! behind it, and a `Solid` is one opaque colour over an axis-aligned
//! rectangle. No number of them makes a radius.
//!
//! So every shape goes to `crate::dock_surface::a_panel`, which already draws
//! the Dock's surface and the Ask alo window's. This window is its **third**
//! caller, which is why there is no rasteriser in this file.
//!
//! # The colours are roles, not tokens
//!
//! `alo_appearance::Role` carries the twelve the design file defines, at the
//! values it defines them. `Token` carries six, which is the older vocabulary
//! and is what most of this crate still draws with — it has no name for the
//! cool ground under a search field, the hairline round a card or the second
//! rank of text, all three of which this window needs.
//!
//! **Deep teal is not among the ones used here.** `Role::AccentDefault` is
//! alo's own (ADR 0067), and an Install button drawn in it would say *alo is
//! doing this* about something the person chose. A primary action in this
//! design is `Role::TextPrimary` with a `Role::BgSurface` label, which is what
//! `Action / Erase and install` is.
//!
//! # It draws what it is given and reads nothing
//!
//! No catalogue is opened here, no machine is asked what is installed, and no
//! sentence is worded: `crate::MarketShows` carries the list,
//! `crate::TheMarketsWords` carries the words already said, and this file turns
//! the two into pixels.

use alo_appearance::{Role, Scheme, TextScale};
use alo_software::Listed;
use cosmic_text::FontSystem;
use smithay::utils::{Physical, Rectangle};

use crate::desktop_look::Measure;
use crate::dock_surface::{APanelsShapes, AWell, a_panel};
use crate::market_measures as m;
use crate::market_window::{Browsing, HowItIsGoing, MarketShows};
use crate::painted::{Inked, Washed};
use crate::painted_text::sentence;
use crate::the_markets_words::TheMarketsWords;

/// The window, rasterised.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarketPicture {
    /// The panel and every rounded shape on it, in one raster.
    pub(crate) panel: Washed,
    /// Where the window sits.
    pub(crate) at: Rectangle<i32, Physical>,
    /// Its words, painted after its panel.
    ///
    /// **Separate from the panel's raster**, for `crate::task_raster`'s
    /// reason: text is inked onto the ground it sits on, so the edges of the
    /// letters blend into that ground rather than into whatever was behind the
    /// window.
    pub(crate) inked: Vec<Inked>,
}

/// One market window open on a display, already placed.
///
/// **Where it sits is the caller's**, as it is for a task window: only the
/// lane knows which display this is and what scale puts a window in its
/// pixels. This file draws what it is handed.
#[derive(Debug, Clone, Copy)]
pub struct AMarketOnTheDisplay<'a> {
    /// Where the window is.
    pub at: Rectangle<i32, Physical>,
    /// What it is showing.
    pub showing: MarketShows<'a>,
    /// Its words, already said.
    pub words: &'a TheMarketsWords,
}

/// How this window is drawn at this moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MarketLook {
    /// Light or dark.
    pub(crate) scheme: Scheme,
    /// The person's text size.
    pub(crate) scale: TextScale,
}

/// Which colour a role is, in this scheme.
///
/// **Dark is not implemented and says so rather than guessing.** `Role` carries
/// the design file's light values; the dark mode is a second set of bindings in
/// that file which `docs/design/palette.toml` does not yet record, and inventing
/// one here would put a colour on a person's screen that no designer chose.
/// Until it is measured, a dark session draws this window in the light roles —
/// which is wrong, and is visibly wrong, which is the better of the two ways to
/// be wrong.
fn colour(role: Role, _scheme: Scheme) -> [u8; 3] {
    let it = role.colour();
    [it.red(), it.green(), it.blue()]
}

/// The colours one card is drawn in.
///
/// **Named rather than five `[u8; 3]` in a row**, which is what clippy asked
/// for and is right: nothing in a five-part tuple says which of them is the
/// ground and which is the second rank of text, and two of the five are a pale
/// grey apiece.
#[derive(Debug, Clone, Copy)]
struct Inks {
    /// Structure and a name.
    ink: [u8; 3],
    /// Metadata — a licence, a version.
    quiet: [u8; 3],
    /// Still text rather than metadata: a second line under a name.
    second: [u8; 3],
    /// The window's own white, which a filled control writes on.
    surface: [u8; 3],
    /// The cooler ground of a well.
    cool: [u8; 3],
}

/// One block of words, where it goes and what it is drawn on.
///
/// **A named thing rather than a four-part tuple**, which is what clippy asked
/// for and is right: the same four values are passed between three functions
/// here, and `(Rectangle, String, [u8; 3], [u8; 3])` says nothing about which
/// colour is the ground and which is the ink.
struct ToWrite {
    /// The band it is centred in.
    at: Rectangle<i32, Physical>,
    /// What it says, already said.
    text: String,
    /// What it sits on, so the letters blend into that and not into whatever
    /// is behind the window.
    ground: [u8; 3],
    /// What it is written in.
    ink: [u8; 3],
}

impl ToWrite {
    /// One block, in the order a reader expects: where, what, on what, in what.
    fn of(at: Rectangle<i32, Physical>, text: String, ground: [u8; 3], ink: [u8; 3]) -> Self {
        Self {
            at,
            text,
            ground,
            ink,
        }
    }
}

/// A rectangle from a corner and a size, in whole pixels.
fn box_at(x: i32, y: i32, wide: i32, tall: i32) -> Rectangle<i32, Physical> {
    Rectangle::new((x, y).into(), (wide.max(0), tall.max(0)).into())
}

/// Draw the window of what a person can install, on a display drawn at
/// `scale`: every measure goes through the person's text size and that scale
/// together, and its corner and shadow through the scale.
///
/// `at` is where the window sits on the output, already placed. Returns [`None`]
/// for a window the rasteriser refuses — one with no room in it — which a
/// caller draws as nothing rather than as a broken frame.
pub(crate) fn picture(
    at: Rectangle<i32, Physical>,
    showing: MarketShows<'_>,
    words: &TheMarketsWords,
    look: MarketLook,
    fonts: &mut FontSystem,
    scale: crate::display_scale::DisplayScale,
) -> Option<MarketPicture> {
    let MarketShows::Browsing(browsing) = showing else {
        return None;
    };
    let measure = Measure::of(look.scale).on(scale);
    let px = |of: u32| measure.px(i32::try_from(of).unwrap_or(i32::MAX));

    // **A window with no room for its own furniture is refused**, not drawn
    // with everything laid out past its edges. The least is the title bar, the
    // search field and the promise, inside both margins.
    let the_least_tall = px(m::the_window_is_tall(0));
    let the_least_wide = px(m::THE_WINDOW_IS_WIDE) / 2;
    if at.size.w < the_least_wide || at.size.h < the_least_tall {
        return None;
    }

    let surface = colour(Role::BgSurface, look.scheme);
    let edge = colour(Role::BorderDefault, look.scheme);
    let cool = colour(Role::BgCool, look.scheme);
    let ink = colour(Role::TextPrimary, look.scheme);
    let quiet = colour(Role::TextMuted, look.scheme);
    let second = colour(Role::TextSecondary, look.scheme);

    // Inside the body: one margin in from the left, under the title bar.
    let left = at.loc.x + px(m::BESIDE);
    let body_wide = px(m::the_body_is_wide());
    let mut down = at.loc.y + px(m::THE_TITLE_BAR_IS_TALL) + px(m::ABOVE);

    let mut wells: Vec<AWell> = Vec::new();
    let mut circles: Vec<Rectangle<i32, Physical>> = Vec::new();
    let mut marks: Vec<Rectangle<i32, Physical>> = Vec::new();
    let mut writing: Vec<ToWrite> = Vec::new();

    // --- the search field ----------------------------------------------------
    let search = box_at(left, down, body_wide, px(m::THE_SEARCH_IS_TALL));
    wells.push(AWell {
        at: search,
        corner: m::A_WELLS_CORNER,
    });
    down += px(m::THE_SEARCH_IS_TALL) + px(m::A_GAP);

    // --- the kinds of work, as chips ----------------------------------------
    //
    // **Each is as wide as its own label**, measured rather than given a fixed
    // width: *Page layout* and *Audio* are not the same length in English and
    // are a different pair of lengths in every other language, so a fixed chip
    // would crop one of them in some translation nobody here reads.
    let chip_tall = px(m::A_CHIP_IS_TALL);
    let mut along = left;
    let chosen = browsing.under();
    for kind in std::iter::once(None).chain(alo_software::Category::EVERY.into_iter().map(Some)) {
        let label = words.the(kind);
        if label.is_empty() {
            continue;
        }
        let shaped = sentence(fonts, label, body_wide, measure.metrics(), cool, ink);
        // **The ink's own width, not the box's.** `Shaped::width_of_box` is
        // the width the text was *wrapped in*, which is the whole body here —
        // sizing a chip by it made the first one 608 pixels wide, overflowed
        // the row, and broke the loop before any chip was drawn at all. The
        // window looked like it had no categories and the fault read as a
        // missing draw call.
        let wide = shaped.width + 2 * px(12);
        if along + wide > left + body_wide {
            break;
        }
        let chip = box_at(along, down, wide, chip_tall);
        if kind == chosen {
            // The chosen one is filled and its label is the surface colour —
            // the primary-action treatment, so selection is carried by
            // contrast and not by hue alone.
            marks.push(chip);
            writing.push(ToWrite::of(chip, label.to_owned(), ink, surface));
        } else {
            wells.push(AWell {
                at: chip,
                corner: m::A_CAPSULE,
            });
            writing.push(ToWrite::of(chip, label.to_owned(), cool, second));
        }
        along += wide + px(m::BETWEEN_CHIPS);
    }
    down += chip_tall + px(m::A_GAP);

    // --- the applications, two across ---------------------------------------
    let card_wide = px(m::a_card_is_wide());
    let card_tall = px(m::A_CARD_IS_TALL);
    let bottom = at.loc.y + at.size.h - px(m::ABOVE) - px(m::THE_PROMISE_IS_TALL);
    for (which, one) in browsing.rows().enumerate() {
        let across = i32::try_from(which as u32 % m::CARDS_ACROSS).unwrap_or(0);
        let row = i32::try_from(which as u32 / m::CARDS_ACROSS).unwrap_or(0);
        let x = left + across * (card_wide + px(m::A_GAP));
        let y = down + row * (card_tall + px(m::A_GAP));
        // A card that would reach past the promise is not drawn, and neither is
        // any after it: half a card says something the list does not.
        if y + card_tall > bottom {
            break;
        }
        draw_one(
            one,
            &browsing,
            box_at(x, y, card_wide, card_tall),
            px(m::INSIDE_A_CARD),
            px(m::A_TILE_IS),
            Inks {
                ink,
                quiet,
                second,
                surface,
                cool,
            },
            words,
            measure,
            &mut wells,
            &mut circles,
            &mut marks,
            &mut writing,
        );
    }

    // --- the promise under the list -----------------------------------------
    writing.push(ToWrite::of(
        box_at(left, bottom, body_wide, px(m::THE_PROMISE_IS_TALL)),
        words.promise.clone(),
        surface,
        quiet,
    ));

    let panel = a_panel(
        at,
        surface,
        edge,
        (at.loc.x + at.size.w, at.loc.y + at.size.h),
        crate::how_a_panel_is_raised::HowItIsRaised::THE_DOCKS.at(scale.factor()),
        APanelsShapes {
            circles: &circles,
            circle: cool,
            marks: &marks,
            mark: ink,
            wells: &wells,
            well_is: cool,
            rings: &[],
            ring_is: edge,
            in_primary: crate::dock_surface::InPrimary::NOTHING,
        },
    )?;

    // --- the words, last ------------------------------------------------------
    let mut writing_out = Vec::with_capacity(writing.len());
    for one in writing {
        let area = one.at;
        let shaped = sentence(
            fonts,
            &one.text,
            area.size.w,
            measure.metrics(),
            one.ground,
            one.ink,
        );
        // Centred in the band it was given, from its own measured height rather
        // than from half a leading: the height depends on the font, the
        // person's text size and how many lines it wrapped to.
        let top = area.loc.y + (area.size.h - shaped.height).max(0) / 2;
        let x = area.loc.x + (area.size.w - shaped.width_of_box).max(0) / 2;
        let left_edge = if area.size.w > shaped.width_of_box * 2 {
            area.loc.x
        } else {
            x
        };
        // `Shaped` already carries the box and its pixels; an `Inked` is that
        // box placed. `painted_text::inked` is a different road -- it takes a
        // `Buffer` and shapes it itself, which `sentence` has already done.
        writing_out.push(Inked {
            area: box_at(left_edge, top, shaped.width_of_box, shaped.height),
            pixels: shaped.pixels,
        });
    }

    Some(MarketPicture {
        panel,
        at,
        inked: writing_out,
    })
}

/// One application's card: its tile, its name, what kind of work it is, its
/// licence and what is to be done about it.
#[expect(
    clippy::too_many_arguments,
    reason = "every colour and measure this draws with is passed in rather than \
              read here, which is the rule this file keeps; bundling them into a \
              struct would hide which of them a card actually uses"
)]
fn draw_one(
    one: &Listed,
    browsing: &Browsing<'_>,
    card: Rectangle<i32, Physical>,
    inside: i32,
    tile_is: i32,
    colours: Inks,
    words: &TheMarketsWords,
    measure: Measure,
    wells: &mut Vec<AWell>,
    circles: &mut Vec<Rectangle<i32, Physical>>,
    marks: &mut Vec<Rectangle<i32, Physical>>,
    writing: &mut Vec<ToWrite>,
) {
    let Inks {
        ink,
        quiet,
        second,
        surface,
        cool,
    } = colours;

    wells.push(AWell {
        at: card,
        corner: m::A_WELLS_CORNER,
    });

    // The artwork tile. A square well rather than a circle, because the design
    // draws a rounded square and `circles` would make it round.
    let tile = box_at(card.loc.x + inside, card.loc.y + inside, tile_is, tile_is);
    wells.push(AWell {
        at: tile,
        corner: m::A_TILES_CORNER,
    });
    let _ = circles;

    // The name and the kind of work, beside the tile.
    let beside = tile.loc.x + tile_is + inside / 2;
    let text_wide = (card.loc.x + card.size.w - inside - beside).max(1);
    writing.push(ToWrite::of(
        box_at(beside, tile.loc.y, text_wide, tile_is / 2),
        one.application().identifier().to_owned(),
        cool,
        ink,
    ));
    // **What kind of work it is, under its name**, said from the vocabulary
    // rather than from `Category::named`, which is the data file's spelling and
    // is not translated.
    writing.push(ToWrite::of(
        box_at(beside, tile.loc.y + tile_is / 2, text_wide, tile_is / 2),
        words.the(Some(one.category())).to_owned(),
        cool,
        second,
    ));

    // What is to be done about it, along the bottom.
    let foot_tall = measure.px(26);
    let foot_y = card.loc.y + card.size.h - inside - foot_tall;
    let licence = box_at(card.loc.x + inside, foot_y, text_wide / 2, foot_tall);
    wells.push(AWell {
        at: licence,
        corner: m::A_CAPSULE,
    });
    writing.push(ToWrite::of(licence, one.licence().to_owned(), cool, quiet));

    // The action, which is a filled capsule for anything not already here.
    let action_wide = measure.px(72);
    let action = box_at(
        card.loc.x + card.size.w - inside - action_wide,
        foot_y,
        action_wide,
        foot_tall,
    );
    match browsing.how_it_is_going(one) {
        HowItIsGoing::Here => {
            wells.push(AWell {
                at: action,
                corner: m::A_CAPSULE,
            });
            writing.push(ToWrite::of(action, words.open.clone(), cool, quiet));
        }
        HowItIsGoing::Asked => {
            // **Not a progress figure.** How far it has got is the rented
            // tool's business; what a person needs is that it is going, and
            // the indicator says what is leaving while it does.
            marks.push(action);
            writing.push(ToWrite::of(action, words.installing.clone(), ink, surface));
        }
        HowItIsGoing::NotAsked | HowItIsGoing::Refused(_) => {
            // **A refusal still offers the control.** `MarketWindow` clears the
            // old reason on the next ask, so a row that was refused is a row a
            // person may try again rather than one that is spent.
            marks.push(action);
            writing.push(ToWrite::of(action, words.install.clone(), ink, surface));
        }
    }
}

#[cfg(test)]
#[path = "market_raster_tests.rs"]
mod tests;
