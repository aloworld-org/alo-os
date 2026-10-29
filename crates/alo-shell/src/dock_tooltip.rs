//! **A control's name, shown beside it on the Dock when a person points at it
//! or the keyboard is on it.**
//!
//! # The owner's decisions
//!
//! 2026-10-10: *no permanent application names beneath icons. Show names on
//! hover and keyboard focus, outside the bar without changing its height.*
//! 2026-10-28: every Dock button gets a tooltip. 2026-10-29, the look and the
//! timing, recorded in `docs/design/the-docks-tooltip.md`:
//!
//! - **It appears straight away**, as the window title's tooltip does. There is
//!   no timer anywhere in this file, so nothing here can make reaching a name
//!   depend on how fast somebody moves or how long they hold still.
//! - **It looks like the selection bar's tooltip** (`166:8846`): navy, a 7
//!   corner, white SemiBold 11/18 text, 8 across and 4 above and below.
//! - **At most the owner's width for a name**: the words wrap at
//!   `alo_dock::measures::A_NAME_BESIDE_AN_ICON`, two lines and then an
//!   ellipsis. A screen reader is given the whole name whatever is drawn.
//!
//! # Only where the name is not already on the screen
//!
//! A control that draws its own words — the composer, *Show all*, History where
//! it is a word — gets no tooltip, for the reason `crate::window_edge_tooltip`
//! gives for a title that fits: a panel repeating what a person can already
//! read is a panel over their work saying nothing. Every control that draws an
//! icon or a mark gets one.
//!
//! # Toward the canvas, outside the bar
//!
//! Above a bottom Dock, below a top one, and beside a Dock down either side, a
//! gap clear of the bar, centred on the control and moved to stay on the
//! display. Every figure is logical and goes through the display's scale; the
//! words also follow the person's text size, because a name is a sentence.

use alo_appearance::TextScale;
use alo_dock::Edge;
use alo_dock::a_dock_entry::ANativeAction;
use alo_dock::places::WhatIsHere;
use cosmic_text::{FontSystem, Metrics};
use smithay::utils::{Physical, Rectangle};

use crate::dock_raster::DockPicture;
use crate::painted::{Inked, Washed};
use crate::the_docks_words::TheDocksWords;

/// The words' size at the ordinary text size: the design's 11.
pub(crate) const THE_WORDS_ARE: u32 = 11;

/// One line, at the ordinary text size: the design's 18.
pub(crate) const A_LINE_IS: u32 = 18;

/// Between the words and the panel's side edges: the design's 8.
pub(crate) const BESIDE_THE_WORDS: u32 = 8;

/// Between the words and the panel's top and bottom: the design's 4.
pub(crate) const ABOVE_AND_BELOW_THE_WORDS: u32 = 4;

/// How many lines a name may take before it ends in an ellipsis.
///
/// The owner's figure for a name beside an icon, recorded in
/// `alo_dock::labels::Labels::Beside`: *wrap to two lines then ellipsize*.
pub(crate) const AT_MOST_LINES: usize = 2;

/// The gap between the bar and the panel.
///
/// **The design does not give one**: its only tooltip sits on a bar of its own
/// and the Dock's has never been drawn. So this is the Dock's own spacing unit,
/// `alo_dock::measures::GAP`, rather than a new number — written down in
/// `docs/design/the-docks-tooltip.md` as the lane's choice.
pub(crate) const CLEAR_OF_THE_BAR: u32 = alo_dock::measures::GAP;

/// What the tooltip on `what` says, or [`None`] where the control already
/// draws its own words.
#[must_use]
pub(crate) fn what_it_says<'a>(what: &'a WhatIsHere, words: &'a TheDocksWords) -> Option<&'a str> {
    match what {
        WhatIsHere::Application(app) => Some(app.name()),
        WhatIsHere::ANativeAction(ANativeAction::Settings) => Some(&words.settings),
        WhatIsHere::ANativeAction(ANativeAction::Applications) => Some(&words.applications),
        WhatIsHere::TheOverflow => Some(&words.show_more_apps),
        WhatIsHere::TheComposer
        | WhatIsHere::ShowAll
        | WhatIsHere::History
        | WhatIsHere::TheDivider => None,
    }
}

/// The panel and its words, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TheNameShown {
    /// The panel, in output pixels.
    pub(crate) area: Rectangle<i32, Physical>,
    /// Its rounded ground.
    pub(crate) panel: Washed,
    /// One per line.
    pub(crate) words: Vec<Inked>,
}

/// Show the name of the control at `which` (counted as
/// `crate::the_bars_pressable_slots` counts) on `picture`, if it has one to
/// show.
///
/// The panel goes on after the Dock's own panels and the words after its
/// words, which is the order `crate::desktop_paint` paints them in, so the
/// tooltip is over everything the Dock drew.
pub(crate) fn show_the_name(
    picture: &mut DockPicture,
    which: Option<usize>,
    words: &TheDocksWords,
    text: TextScale,
    fonts: &mut FontSystem,
) {
    let Some(shown) = which.and_then(|which| the_name_shown(picture, which, words, text, fonts))
    else {
        return;
    };
    picture.panels.push(shown.panel);
    picture.inked.extend(shown.words);
}

/// The name of the control at `which` on `picture`, laid out and drawn.
///
/// [`None`] for a slot that is not there, a control with no tooltip, a name
/// with nothing to draw, or a display too small to hold the panel at all.
#[must_use]
pub(crate) fn the_name_shown(
    picture: &DockPicture,
    which: usize,
    words: &TheDocksWords,
    text: TextScale,
    fonts: &mut FontSystem,
) -> Option<TheNameShown> {
    let dock = &picture.standing;
    let place = crate::the_bars_pressable_slots::the_one_at(&dock.places, which)?;
    let name = what_it_says(place.what(), words)?;
    let scale = dock.scale;
    let (width, height) = picture.size;

    let percent = u32::from(text.as_percent());
    let size = scale.px(THE_WORDS_ARE * percent / 100);
    let line = scale.px(A_LINE_IS * percent / 100);
    #[expect(
        clippy::cast_precision_loss,
        reason = "a type size and a line height in pixels are a few dozen, well inside an f32"
    )]
    let metrics = Metrics::new(size as f32, line as f32);
    let room = scale.px(alo_dock::measures::A_NAME_BESIDE_AN_ICON);
    let (mut lines, cut) =
        crate::painted_text_wrapping::wrapped(name, room, fonts, metrics, AT_MOST_LINES);
    if cut && let Some(last) = lines.pop() {
        lines.push(crate::painted_text_wrapping::ending_in_an_ellipsis(
            &last, room, fonts, metrics,
        ));
    }
    if lines.is_empty() {
        return None;
    }
    let wide = lines
        .iter()
        .map(|each| crate::painted_text::how_wide(fonts, each, metrics))
        .max()
        .unwrap_or(room)
        .min(room);
    let (beside, above) = (
        scale.px(BESIDE_THE_WORDS),
        scale.px(ABOVE_AND_BELOW_THE_WORDS),
    );
    let across = wide + 2 * beside;
    let tall = i32::try_from(lines.len()).ok()? * line + 2 * above;
    if across > width || tall > height {
        return None;
    }

    let icon = scale.px(alo_dock::measures::ICON);
    let along = scale.px(place.from_the_start());
    let (left, top) = crate::dock_raster::a_slot_at(dock.band, dock.layout.edge(), along, icon);
    let slot = Rectangle::new((left, top).into(), (scale.px(place.across()), icon).into());
    let area = placed(
        dock.band,
        slot,
        (across, tall),
        dock.layout.edge(),
        scale.px(CLEAR_OF_THE_BAR),
        (width, height),
    );

    let fill = alo_appearance::Role::TextPrimary.colour();
    let fill = [fill.red(), fill.green(), fill.blue()];
    let ink = alo_appearance::Role::BgSurface.colour();
    let ink = [ink.red(), ink.green(), ink.blue()];
    let panel = crate::dock_surface::a_panel(
        area,
        fill,
        // **No hairline**: the design's tooltip has none, so its edge is its fill.
        fill,
        (width, height),
        crate::how_a_panel_is_raised::HowItIsRaised::A_TOOLTIP.at(scale.factor()),
        crate::dock_surface::APanelsShapes {
            circles: &[],
            circle: fill,
            marks: &[],
            mark: fill,
            wells: &[],
            well_is: fill,
            rings: &[],
            ring_is: fill,
            in_primary: crate::dock_surface::InPrimary::NOTHING,
        },
    )?;
    let words = lines
        .iter()
        .zip(0_i32..)
        .filter_map(|(each, down)| {
            crate::painted_text::sentence(fonts, each, room, metrics, fill, ink)
                .cropped_to_its_ink()
                .placed(
                    area.loc.x + beside,
                    area.loc.y + above + down * line,
                    height,
                )
        })
        .collect();
    Some(TheNameShown { area, panel, words })
}

/// Where a panel of `size` goes beside `slot`: toward the canvas from `band`,
/// `gap` clear of it, centred on the slot, and moved to stay on a display of
/// `display`.
#[must_use]
pub(crate) fn placed(
    band: Rectangle<i32, Physical>,
    slot: Rectangle<i32, Physical>,
    size: (i32, i32),
    edge: Edge,
    gap: i32,
    display: (i32, i32),
) -> Rectangle<i32, Physical> {
    let (across, tall) = size;
    let centred_across = slot.loc.x + slot.size.w / 2 - across / 2;
    let centred_down = slot.loc.y + slot.size.h / 2 - tall / 2;
    let (x, y) = match edge {
        Edge::Bottom => (centred_across, band.loc.y - gap - tall),
        Edge::Top => (centred_across, band.loc.y + band.size.h + gap),
        Edge::Left => (band.loc.x + band.size.w + gap, centred_down),
        Edge::Right => (band.loc.x - gap - across, centred_down),
    };
    let x = x.clamp(0, (display.0 - across).max(0));
    let y = y.clamp(0, (display.1 - tall).max(0));
    Rectangle::new((x, y).into(), (across, tall).into())
}

impl crate::Server {
    /// Which control on `display`'s Dock the pointer is on, counted as
    /// `crate::the_bars_pressable_slots` counts.
    ///
    /// **The same hit test a press uses** (`Server::the_dock_takes_this_press`),
    /// on the same Dock as it was last drawn, so the name a person reads is the
    /// name of the control a click would press.
    #[must_use]
    pub(crate) fn the_slot_the_pointer_is_on(&self, display: &str) -> Option<usize> {
        let at = self.where_the_pointer_is_in_pixels()?;
        let dock = self.dock_as_drawn.get(display)?;
        let place = dock.places.at(dock.how_far_along(at)?)?;
        crate::the_bars_pressable_slots::which_one_is(&dock.places, place)
    }
}

#[cfg(test)]
#[path = "dock_tooltip_tests.rs"]
mod tests;
