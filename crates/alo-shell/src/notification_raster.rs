//! Notifications laid out and rasterised for one output: one card for each one
//! `alo-notifying` handed over, and nothing at all when it handed none.
//!
//! # This file cannot show a notification that should be held
//!
//! **It takes `alo_notifying::Shown` and there is no other road to one.** That
//! crate's `deciding::arrives` is the only thing that makes a `Shown`, and it
//! has already asked the two questions that matter: whether the machine is
//! locked — in which case the notification waits behind the lock screen and
//! comes back at the unlock — and whether anything is holding notifications at
//! all, which includes *the screen is being shared or recorded* and *the
//! machine could not tell whether it is*.
//!
//! So *never while locked, shared or recorded* is not a rule this file
//! implements or could get wrong. It is a rule the type carries: a held
//! notification is a `Became::Held`, it has no `Shown` inside it, and there is
//! nothing here that takes anything else.
//!
//! # Where they sit
//!
//! At the end of the dock **opposite** the status area
//! (`crate::egress_status_place::Place::of_the_other_end`). The two indicators
//! own the status corner and are permanent; a notification arrives and goes,
//! and one that covered *what is leaving this machine* or *your camera is on*
//! would be trading a promise for a convenience.
//!
//! # What is drawn
//!
//! Per card: who it is from, its title, its body, and the label of every action
//! its sender offered — each as `alo-notifying` worded it, never shortened or
//! re-worded here. The agent's card is terracotta, and **says *the agent* in so
//! many words** as well (`Notification::sent_by`), which is what carries
//! ADR 0010's signal to somebody who cannot tell terracotta from anything else.

use alo_appearance::{Scheme, TextScale, Token};
use alo_dock::{Dock, Screen};
use alo_notifying::Shown;
use alo_strings::{Direction, Strings};
use smithay::utils::{Physical, Rectangle};

use crate::egress_status_place::{Across, Place, Stacked};
use crate::painted::{Inked, Solid};
use crate::painted_text::sentence;
use crate::status_row::Measure;
use crate::{Contrast, RenderError, WindowControlLabels};

/// How the cards look, as the person's appearance and language decide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotificationLook {
    /// Light or dark, as `alo-appearance` decides it.
    pub scheme: Scheme,
    /// The person's text scale, applied to every measure.
    pub scale: TextScale,
    /// Which way the person reads.
    pub reading: Direction,
    /// The design's palette, or the one high contrast decides.
    pub contrast: Contrast,
}

/// The largest output side, in pixels, a card is laid out for.
const LARGEST_SIDE: i32 = 16_384;

/// One card as drawn, for a test to read back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Card {
    /// The whole card, in output pixels.
    pub(crate) area: Rectangle<i32, Physical>,
    /// Every line of text on it, in the order it is drawn.
    pub(crate) lines: Vec<String>,
    /// Whether it is drawn in the colour that means the agent.
    pub(crate) the_agents: bool,
}

/// The notifications on one output, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NotificationPicture {
    /// The output size it was laid out for.
    pub(crate) size: (i32, i32),
    /// Every card, nearest the dock first.
    pub(crate) cards: Vec<Card>,
    /// Flat shapes, painted first.
    pub(crate) solids: Vec<Solid>,
    /// Words, painted after the shapes.
    pub(crate) inked: Vec<Inked>,
}

impl NotificationPicture {
    /// Whether nothing at all is drawn.
    pub(crate) fn is_empty(&self) -> bool {
        self.cards.is_empty() && self.solids.is_empty() && self.inked.is_empty()
    }
}

/// Lay every notification out for an output of `size` and rasterise it.
///
/// No notifications is an empty picture on any output.
///
/// # Errors
/// [`RenderError::NotificationScene`] when there is one to show and the output
/// is too small for `alo-dock` to lay a dock out on, or larger than any output
/// a card is laid out for.
pub(crate) fn picture(
    showing: &[Shown],
    strings: &Strings,
    dock: &Dock,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
    look: NotificationLook,
) -> Result<NotificationPicture, RenderError> {
    let mut picture = NotificationPicture {
        size,
        cards: Vec::new(),
        solids: Vec::new(),
        inked: Vec::new(),
    };
    if showing.is_empty() {
        return Ok(picture);
    }
    let (width, height) = size;
    if width > LARGEST_SIDE || height > LARGEST_SIDE {
        return Err(RenderError::NotificationScene);
    }
    let screen = Screen::of(
        u32::try_from(width).map_err(|_| RenderError::NotificationScene)?,
        u32::try_from(height).map_err(|_| RenderError::NotificationScene)?,
    )
    .map_err(|_| RenderError::NotificationScene)?;
    let measure = Measure::of(look.scale);
    let place = Place::of_the_other_end(
        dock.layout_on(screen, look.scale, look.reading),
        size,
        measure.px(8),
    );
    let ground = look.contrast.ground(look.scheme);
    let ink = look.contrast.ink(look.scheme);

    let mut at_y = place.stacked;
    let mut used = 0;
    for shown in showing {
        let card = lines_of(shown, strings);
        let widest = (place.room_across - 2 * measure.px(12))
            .min(measure.px(360))
            .max(1);
        let shaped: Vec<_> = card
            .lines
            .iter()
            .map(|line| {
                sentence(
                    &mut labels.fonts,
                    line,
                    widest,
                    measure.metrics(),
                    ground,
                    ink,
                )
            })
            .collect();
        let inside: i32 = shaped.iter().map(|line| line.height).sum();
        let tall = inside + 2 * measure.px(12);
        let wide = shaped
            .iter()
            .map(|line| line.width_of_box)
            .max()
            .unwrap_or(0)
            + 2 * measure.px(12);
        // A card that would not fit is not drawn, and neither is any after it:
        // half a notification says something the sender did not.
        if used + tall > place.room_down {
            break;
        }
        let left = match place.across {
            Across::FromLeft(x) => x,
            Across::FromRight(x) => x - wide,
        };
        let top = match at_y {
            Stacked::Downwards(y) => {
                at_y = Stacked::Downwards(y + tall + measure.px(8));
                y
            }
            Stacked::Upwards(y) => {
                at_y = Stacked::Upwards(y - tall - measure.px(8));
                y - tall
            }
        };
        used += tall + measure.px(8);

        let whole = Rectangle::<i32, Physical>::from_size(size.into());
        let area = Rectangle::new((left.max(0), top.max(0)).into(), (wide, tall).into());
        let edge = if card.the_agents {
            look.contrast
                .accent(look.scheme, Token::Terracotta.colour())
        } else {
            ink
        };
        for solid in [
            Solid { area, colour: edge },
            Solid {
                area: Rectangle::new(
                    (area.loc.x + measure.px(2), area.loc.y + measure.px(2)).into(),
                    (
                        (area.size.w - 2 * measure.px(2)).max(1),
                        (area.size.h - 2 * measure.px(2)).max(1),
                    )
                        .into(),
                ),
                colour: ground,
            },
        ] {
            if let Some(kept) = solid.area.intersection(whole) {
                picture.solids.push(Solid {
                    area: kept,
                    colour: solid.colour,
                });
            }
        }
        let mut line_y = area.loc.y + measure.px(12);
        for line in shaped {
            let line_height = line.height;
            if let Some(inked) = line.placed(area.loc.x + measure.px(12), line_y, height) {
                picture.inked.push(inked);
            }
            line_y += line_height;
        }
        picture.cards.push(Card {
            area: area.intersection(whole).unwrap_or_default(),
            lines: card.lines,
            the_agents: card.the_agents,
        });
    }
    Ok(picture)
}

/// Everything one card says, in the order it is drawn.
struct Lines {
    /// Who it is from, its title, its body when it has one, and every action.
    lines: Vec<String>,
    /// Whether the agent sent it.
    the_agents: bool,
}

/// The sentences on one card, each exactly as `alo-notifying` worded it.
fn lines_of(shown: &Shown, strings: &Strings) -> Lines {
    let notification = shown.notification();
    let mut lines = vec![notification.sent_by(strings).text().to_owned()];
    lines.push(notification.title().to_owned());
    if !notification.body().is_empty() {
        lines.push(notification.body().to_owned());
    }
    lines.extend(
        notification
            .offering()
            .iter()
            .map(|action| action.label().to_owned()),
    );
    Lines {
        lines,
        the_agents: notification.is_the_agents(),
    }
}

#[cfg(test)]
#[path = "notification_raster_tests.rs"]
mod tests;
