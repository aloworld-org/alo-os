//! How the ordinary desktop looks: light or dark, the accent, the text size and
//! the reading direction — every one of them `alo-appearance`'s answer, asked at
//! the moment of drawing.
//!
//! The dock, the window of what is running and the window of what is filling
//! the disk are drawn from one [`DesktopLook`], and a [`DesktopLook`] can only
//! be made by asking a person's `alo_appearance::Appearance` what it is at a
//! time of day. There is no constructor taking a scheme or a colour, so nothing
//! in this crate can decide that the desktop is dark, or what its accent is: a
//! machine that turns dark at six turns its desktop with it, and an accent a
//! person chose is the one drawn.
//!
//! # Terracotta is never offered
//!
//! Terracotta means the agent and nothing else (ADR 0010). The accent reaches
//! a [`DesktopPalette`] only through `alo_appearance::Accent::of_colour`, the
//! door that refuses terracotta in words, and the palette's other colours are
//! the grounds and structure of the design brief. A palette that would offer
//! terracotta is refused rather than drawn, and a test says so.

use alo_access::TurnedOn;
use alo_appearance::{Accent, AccentError, Appearance, Colour, Scheme, TextScale, TimeOfDay};
use alo_strings::Direction;
use cosmic_text::Metrics;

use crate::{Contrast, EgressStatusLook};

/// How the ordinary desktop looks, as the person's appearance decides it at
/// one moment and as their language is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopLook {
    /// Light or dark, as `alo-appearance` answered it.
    scheme: Scheme,
    /// The accent colour, as `alo-appearance` answered it.
    accent: Colour,
    /// The person's text size.
    scale: TextScale,
    /// Which way the person reads.
    reading: Direction,
    /// The design's palette, or the one high contrast decides.
    contrast: Contrast,
}

impl DesktopLook {
    /// The desktop as `appearance` looks at `now`, for somebody reading
    /// `reading`, with what they have `turned_on` for access.
    ///
    /// The only way to make one: light and dark, the accent and the text size
    /// are the appearance's own answers at that time of day, never this
    /// crate's, and which palette is drawn is what the person turned on, never
    /// a choice made here.
    #[must_use]
    pub fn of(
        appearance: &Appearance,
        turned_on: &TurnedOn,
        now: TimeOfDay,
        reading: Direction,
    ) -> Self {
        Self {
            scheme: appearance.scheme_at(now),
            accent: appearance.accent_at(now),
            scale: appearance.text(),
            reading,
            contrast: Contrast::of(turned_on),
        }
    }

    /// Light or dark.
    #[must_use]
    pub const fn scheme(self) -> Scheme {
        self.scheme
    }

    /// The accent colour.
    #[must_use]
    pub const fn accent(self) -> Colour {
        self.accent
    }

    /// The text size.
    #[must_use]
    pub const fn scale(self) -> TextScale {
        self.scale
    }

    /// Which way the person reads.
    #[must_use]
    pub const fn reading(self) -> Direction {
        self.reading
    }

    /// Which palette the desktop is drawn in.
    #[must_use]
    pub const fn contrast(self) -> Contrast {
        self.contrast
    }

    /// The colours this look draws in.
    ///
    /// # Errors
    /// `alo-appearance`'s refusal when the accent is not one a person can
    /// choose — terracotta above all.
    pub(crate) fn palette(self) -> Result<DesktopPalette, AccentError> {
        DesktopPalette::of(self.scheme, self.accent, self.contrast)
    }

    /// The same look, for the egress indicator in the dock's status area.
    pub(crate) const fn egress(self) -> EgressStatusLook {
        EgressStatusLook {
            scheme: self.scheme,
            scale: self.scale,
            reading: self.reading,
            contrast: self.contrast,
        }
    }

    /// The same look, for the in-use indicator beside it in that status area.
    ///
    /// The same four answers: the two indicators sit side by side and a person
    /// reads them as one surface, so neither has a look of its own.
    pub(crate) const fn in_use(self) -> crate::in_use_raster::InUseLook {
        crate::in_use_raster::InUseLook {
            scheme: self.scheme,
            scale: self.scale,
            reading: self.reading,
            contrast: self.contrast,
        }
    }

    /// The same look, for the notifications at the other end of the dock.
    pub(crate) const fn notifications(self) -> crate::notification_raster::NotificationLook {
        crate::notification_raster::NotificationLook {
            scheme: self.scheme,
            scale: self.scale,
            reading: self.reading,
            contrast: self.contrast,
        }
    }

    /// The same look, for the capture tools drawn over everything.
    pub(crate) const fn capture(self) -> crate::capture_raster::CaptureLook {
        crate::capture_raster::CaptureLook {
            scheme: self.scheme,
            scale: self.scale,
            contrast: self.contrast,
        }
    }

    /// Measures at this look's text size.
    pub(crate) fn measure(self) -> Measure {
        Measure::of(self.scale)
    }
}

/// The colours the ordinary desktop is drawn in, for one scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DesktopPalette {
    /// A window's ground.
    pub(crate) ground: [u8; 3],
    /// The dock's ground.
    pub(crate) dock: [u8; 3],
    /// Words, edges and rules.
    pub(crate) ink: [u8; 3],
    /// The person's accent, on this scheme's ground.
    pub(crate) accent: [u8; 3],
}

impl DesktopPalette {
    /// The palette for `scheme` with `accent`.
    ///
    /// The accent is let in only through `alo_appearance::Accent::of_colour`,
    /// and drawn as that accent's value for this scheme, so a light value asked
    /// for on a dark desktop is still the one that reads on it.
    ///
    /// # Errors
    /// [`AccentError::Reserved`] for terracotta, and the other refusals for a
    /// colour that is not an accent at all.
    pub(crate) fn of(
        scheme: Scheme,
        accent: Colour,
        contrast: Contrast,
    ) -> Result<Self, AccentError> {
        // The accent a person chose is checked whichever palette is drawn, so
        // that a colour nobody may choose is refused rather than quietly
        // replaced by the contrast palette's.
        let accent = Accent::of_colour(accent)?.on(scheme);
        Ok(Self {
            ground: contrast.ground(scheme),
            dock: contrast.dock(scheme),
            ink: contrast.ink(scheme),
            accent: contrast.accent(scheme, accent),
        })
    }

    /// Every colour this palette offers.
    #[cfg(test)]
    pub(crate) const fn offers(self) -> [[u8; 3]; 4] {
        [self.ground, self.dock, self.ink, self.accent]
    }
}

/// A colour as the painter takes it.
///
/// Only a test's now: every colour the desktop draws comes from the palette
/// door (`crate::access_contrast`), which hands them over in this shape.
#[cfg(test)]
pub(crate) const fn rgb(colour: Colour) -> [u8; 3] {
    [colour.red(), colour.green(), colour.blue()]
}

/// Measures scaled by the person's text size, never below one pixel.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Measure {
    /// The size, in percent.
    percent: i32,
}

impl Measure {
    /// Measures for this text size.
    pub(crate) fn of(scale: TextScale) -> Self {
        Self {
            percent: i32::from(scale.as_percent()),
        }
    }

    /// `at_one` pixels at the ordinary size, scaled.
    pub(crate) fn px(self, at_one: i32) -> i32 {
        (at_one * self.percent / 100).max(1)
    }

    /// The text metrics every line on the desktop uses.
    pub(crate) fn metrics(self) -> Metrics {
        let factor = self.percent as f32 / 100.0;
        Metrics::new(16.0 * factor, 24.0 * factor)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_appearance::Token;
    use alo_appearance::{Following, Shipped};

    /// Half past seven in the evening.
    fn evening() -> TimeOfDay {
        TimeOfDay::checked(19, 30).unwrap()
    }

    /// Nine in the morning.
    fn morning() -> TimeOfDay {
        TimeOfDay::checked(9, 0).unwrap()
    }

    /// **Light and dark are `alo-appearance`'s.** A person who follows the
    /// evening schedule gets a light desktop at nine and a dark one at half past
    /// seven, with the accent turning with it — and the look is exactly what the
    /// appearance answers at each moment, never a scheme chosen here.
    #[test]
    fn the_desktop_follows_light_and_dark_as_alo_appearance_decides_it() {
        let mut appearance = Appearance::shipped();
        appearance.follow(Following::from(Shipped::the_evening_schedule()));
        appearance.set_accent(Accent::Moss);
        appearance.set_text(TextScale::percent(150).unwrap());

        for now in [morning(), evening()] {
            let look = DesktopLook::of(
                &appearance,
                &TurnedOn::nothing(),
                now,
                Direction::LeftToRight,
            );
            assert_eq!(look.scheme(), appearance.scheme_at(now));
            assert_eq!(look.accent(), appearance.accent_at(now));
            assert_eq!(look.scale(), appearance.text());
            let palette = look.palette().unwrap();
            assert_eq!(palette.accent, rgb(appearance.accent_at(now)));
        }
        assert_eq!(
            DesktopLook::of(
                &appearance,
                &TurnedOn::nothing(),
                morning(),
                Direction::LeftToRight
            )
            .scheme(),
            Scheme::Light
        );
        assert_eq!(
            DesktopLook::of(
                &appearance,
                &TurnedOn::nothing(),
                evening(),
                Direction::LeftToRight
            )
            .scheme(),
            Scheme::Dark
        );

        // A person who changes their mind changes the desktop, and nothing else
        // has to be told.
        appearance.set_accent(Accent::Rose);
        let look = DesktopLook::of(
            &appearance,
            &TurnedOn::nothing(),
            evening(),
            Direction::RightToLeft,
        );
        assert_eq!(look.accent(), Accent::Rose.on(Scheme::Dark));
        assert_eq!(look.reading(), Direction::RightToLeft);
        assert_eq!(look.egress().scheme, Scheme::Dark);
    }

    /// **A palette that offers terracotta is refused**, in `alo-appearance`'s
    /// own refusal, whichever scheme asks — and so is any palette colour that is
    /// a ground rather than an accent.
    #[test]
    fn a_palette_that_offers_terracotta_is_refused() {
        for scheme in [Scheme::Light, Scheme::Dark] {
            assert_eq!(
                DesktopPalette::of(scheme, Token::Terracotta.colour(), Contrast::AsDesigned),
                Err(AccentError::Reserved)
            );
            assert_eq!(
                DesktopPalette::of(scheme, Token::Navy.colour(), Contrast::AsDesigned),
                Err(AccentError::NotAnAccent(Token::Navy))
            );
            let invented = Colour::of(0xE7, 0x6F, 0x52);
            assert_eq!(
                DesktopPalette::of(scheme, invented, Contrast::AsDesigned),
                Err(AccentError::NotOffered(invented))
            );
        }
    }

    /// **No palette the desktop can draw offers terracotta**: every accent a
    /// person can choose, on both schemes, and every colour in each palette.
    #[test]
    fn no_palette_the_desktop_draws_offers_terracotta() {
        let terracotta = rgb(Token::Terracotta.colour());
        for accent in Accent::ALL {
            for scheme in [Scheme::Light, Scheme::Dark] {
                let palette =
                    DesktopPalette::of(scheme, accent.on(scheme), Contrast::AsDesigned).unwrap();
                assert!(
                    !palette.offers().contains(&terracotta),
                    "{accent:?} {scheme:?}"
                );
                assert_eq!(palette.accent, rgb(accent.on(scheme)));
            }
        }
    }

    /// **The accent is drawn as its value for the scheme on screen**, whichever
    /// of its two values it arrived as.
    #[test]
    fn the_accent_is_drawn_for_the_scheme_on_screen() {
        let palette = DesktopPalette::of(
            Scheme::Dark,
            Accent::Indigo.on(Scheme::Light),
            Contrast::AsDesigned,
        )
        .unwrap();
        assert_eq!(palette.accent, rgb(Accent::Indigo.on(Scheme::Dark)));
    }
}
