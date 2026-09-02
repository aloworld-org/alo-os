//! What is behind the windows: a picture, a folder that rotates, or a colour.
//!
//! Three and no fourth. `docs/features.md` names exactly these three at v0.5,
//! and a background is one of the few places where a list that grew by habit
//! would cost something real — every kind here is something the compositor has
//! to draw correctly on every display, at every scale, before the first window
//! appears.
//!
//! **Whether a background rotates is a question this type answers**, because two
//! other decisions turn on it: the lock screen does not follow a rotating
//! desktop ([`crate::lock`]), and a settings panel showing "changes every ten
//! minutes" needs to know without taking the enum apart itself.

use serde::{Deserialize, Serialize};

use crate::colour::Colour;
use crate::picture::Picture;
use crate::rotating::Rotating;

/// What is behind the windows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Background {
    /// One picture.
    Picture(Picture),
    /// A folder of them, one at a time.
    Rotating(Rotating),
    /// A plain colour.
    Colour(Colour),
}

impl Background {
    /// Whether this background changes on its own.
    #[must_use]
    pub fn rotates(&self) -> bool {
        matches!(self, Self::Rotating(_))
    }
}

impl From<Picture> for Background {
    fn from(picture: Picture) -> Self {
        Self::Picture(picture)
    }
}

impl From<Rotating> for Background {
    fn from(rotating: Rotating) -> Self {
        Self::Rotating(rotating)
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
    use crate::rotating::Every;
    use crate::token::Token;
    use std::path::PathBuf;

    /// A rotating background, on a path no machine has.
    fn rotating() -> Rotating {
        let folder = if cfg!(windows) {
            PathBuf::from(r"C:\Users\a\Pictures\Rotating")
        } else {
            PathBuf::from("/home/a/Pictures/Rotating")
        };
        Rotating::folder(folder, Every::minutes(10).unwrap()).unwrap()
    }

    /// One of the three answers *yes* to rotating and the other two answer
    /// *no*, which is what the lock screen and the settings panel ask.
    #[test]
    fn only_a_folder_rotates() {
        assert!(Background::from(rotating()).rotates());
        assert!(!Background::from(Picture::shipped("alo").unwrap()).rotates());
        assert!(!Background::from(Token::Navy.colour()).rotates());
    }

    /// Each of the three survives a settings file unchanged.
    #[test]
    fn every_kind_survives_being_written_down() {
        let each = [
            Background::from(Picture::shipped("alo").unwrap()),
            Background::from(rotating()),
            Background::from(Token::Charcoal.colour()),
        ];
        for background in each {
            let written = serde_json::to_string(&background).unwrap();
            assert_eq!(
                serde_json::from_str::<Background>(&written).unwrap(),
                background
            );
        }
    }
}
