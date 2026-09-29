//! The surface a person works on, and what it is made of.
//!
//! **alo OS ships no wallpaper.** The canvas plane's own surface *is* the
//! desktop ([ADR 0075](../../../docs/decisions/0075-alo-os-has-no-wallpaper-the-canvass-own-surface-is-the-desktop.md)),
//! so what a person chooses here is that surface's material rather than a
//! picture behind it. Both places a picture could have gone were worse than
//! none: on the plane an ordinary pan drags it off and *show all* shrinks it to a
//! postage stamp, and on the viewport it is a backdrop the work slides across,
//! leaving two things to make yours where there should be one.
//!
//! # Why this is still a choice and not simply a colour
//!
//! One kind today — a colour. The record says what a person chooses is the
//! surface's **material**, and *how it is finished* is a second kind that does
//! not exist yet. Keeping the choice a choice is what lets that arrive without
//! every caller changing shape on the day it does, and it is the difference
//! between a type that is small and a type that was flattened.
//!
//! # What used to be here
//!
//! A picture, a folder that rotated, or a colour. The first two are gone with
//! the record above, and with them `rotates()` — the question the lock screen and
//! the settings panel used to ask. Nothing rotates now, so nothing asks.

use serde::{Deserialize, Serialize};

use crate::colour::Colour;

/// The surface a person works on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Background {
    /// A plain colour.
    Colour(Colour),
}

impl Background {
    /// The colour it comes out as, whatever kind it is.
    ///
    /// One kind answers this today. It exists so that the compositor asks the
    /// surface what to paint rather than taking the enum apart, which is what
    /// keeps a second kind from being a change in every drawing path.
    #[must_use]
    pub const fn colour(self) -> Colour {
        match self {
            Self::Colour(colour) => colour,
        }
    }
}

impl From<Colour> for Background {
    fn from(colour: Colour) -> Self {
        Self::Colour(colour)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::token::Token;

    /// **A colour background comes out as the colour it was made from.**
    #[test]
    fn a_colour_background_is_that_colour() {
        let porcelain = Token::Porcelain.colour();
        assert_eq!(Background::from(porcelain).colour(), porcelain);
        assert_eq!(
            Background::Colour(Token::Charcoal.colour()).colour(),
            Token::Charcoal.colour()
        );
    }

    /// **It survives being written and read**, which is what the settings file
    /// needs of it.
    #[test]
    fn it_survives_being_written_and_read() {
        let surface = Background::from(Token::Porcelain.colour());
        let written = toml::to_string(&surface).unwrap();
        assert_eq!(toml::from_str::<Background>(&written).unwrap(), surface);
    }
}
