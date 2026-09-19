//! Which part of a screen a window had, as a list of what was open writes it
//! down.
//!
//! The whole of the screen, or one of the pieces `alo_dividing::Place` names —
//! a half, a quarter, or a part that is neither. Nothing else: not a position,
//! not a size, not a window's identifier, and nothing that could be read back
//! as where a person's work was on the glass in front of them.
//!
//! # Why a mirror of `Place`, and why it is safe
//!
//! `alo_dividing::Place` is read off the geometry by the crate that owns
//! divisions and is not a thing this crate decides. What *is* this crate's is
//! how it is spelled in a person's own file, which is a name a person may edit
//! and a name a later release may not quietly change — so the spelling is
//! declared here, as [`Written`], and every conversion is an exhaustive match.
//! A tenth place added to `alo-dividing` tomorrow is a compile error in this
//! file rather than a silent `part` in somebody's file.

use alo_dividing::Place;
use serde::{Deserialize, Serialize};

/// What a window had of one screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Written", into = "Written")]
pub enum Split {
    /// The whole of that screen: a window nobody had divided.
    TheWholeScreen,
    /// A named piece of it.
    Of(Place),
}

impl Split {
    /// The name this file writes it under, and the one a person may read in
    /// their own folder.
    #[must_use]
    pub const fn named(self) -> &'static str {
        match self {
            Self::TheWholeScreen => "the-whole-screen",
            Self::Of(Place::LeftHalf) => "left-half",
            Self::Of(Place::RightHalf) => "right-half",
            Self::Of(Place::TopHalf) => "top-half",
            Self::Of(Place::BottomHalf) => "bottom-half",
            Self::Of(Place::TopLeftQuarter) => "top-left-quarter",
            Self::Of(Place::TopRightQuarter) => "top-right-quarter",
            Self::Of(Place::BottomLeftQuarter) => "bottom-left-quarter",
            Self::Of(Place::BottomRightQuarter) => "bottom-right-quarter",
            Self::Of(Place::Part) => "part",
        }
    }

    /// Every split a file may say, in the order this crate declares them — for
    /// the tests, and for a surface that has to offer them all.
    pub const EVERY_ONE: [Self; 10] = [
        Self::TheWholeScreen,
        Self::Of(Place::LeftHalf),
        Self::Of(Place::RightHalf),
        Self::Of(Place::TopHalf),
        Self::Of(Place::BottomHalf),
        Self::Of(Place::TopLeftQuarter),
        Self::Of(Place::TopRightQuarter),
        Self::Of(Place::BottomLeftQuarter),
        Self::Of(Place::BottomRightQuarter),
        Self::Of(Place::Part),
    ];
}

/// A split as `leaving.toml` spells it.
///
/// This crate's own, for the reason at the top of this file. Both conversions
/// below are exhaustive, so the two cannot drift.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Written {
    /// The whole of that screen.
    TheWholeScreen,
    /// The left half.
    LeftHalf,
    /// The right half.
    RightHalf,
    /// The top half.
    TopHalf,
    /// The bottom half.
    BottomHalf,
    /// The top-left quarter.
    TopLeftQuarter,
    /// The top-right quarter.
    TopRightQuarter,
    /// The bottom-left quarter.
    BottomLeftQuarter,
    /// The bottom-right quarter.
    BottomRightQuarter,
    /// A piece that is none of those.
    Part,
}

impl From<Written> for Split {
    fn from(written: Written) -> Self {
        match written {
            Written::TheWholeScreen => Self::TheWholeScreen,
            Written::LeftHalf => Self::Of(Place::LeftHalf),
            Written::RightHalf => Self::Of(Place::RightHalf),
            Written::TopHalf => Self::Of(Place::TopHalf),
            Written::BottomHalf => Self::Of(Place::BottomHalf),
            Written::TopLeftQuarter => Self::Of(Place::TopLeftQuarter),
            Written::TopRightQuarter => Self::Of(Place::TopRightQuarter),
            Written::BottomLeftQuarter => Self::Of(Place::BottomLeftQuarter),
            Written::BottomRightQuarter => Self::Of(Place::BottomRightQuarter),
            Written::Part => Self::Of(Place::Part),
        }
    }
}

impl From<Split> for Written {
    fn from(split: Split) -> Self {
        match split {
            Split::TheWholeScreen => Self::TheWholeScreen,
            Split::Of(Place::LeftHalf) => Self::LeftHalf,
            Split::Of(Place::RightHalf) => Self::RightHalf,
            Split::Of(Place::TopHalf) => Self::TopHalf,
            Split::Of(Place::BottomHalf) => Self::BottomHalf,
            Split::Of(Place::TopLeftQuarter) => Self::TopLeftQuarter,
            Split::Of(Place::TopRightQuarter) => Self::TopRightQuarter,
            Split::Of(Place::BottomLeftQuarter) => Self::BottomLeftQuarter,
            Split::Of(Place::BottomRightQuarter) => Self::BottomRightQuarter,
            Split::Of(Place::Part) => Self::Part,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **Every split survives the file it is written into**, under the name
    /// this crate declares for it and no other.
    #[test]
    fn every_split_is_written_as_its_name_and_read_back_as_itself() {
        for split in Split::EVERY_ONE {
            let written = toml::to_string(&Held { split }).unwrap();
            assert_eq!(written, format!("split = \"{}\"\n", split.named()));
            let back: Held = toml::from_str(&written).unwrap();
            assert_eq!(back.split, split);
        }
    }

    /// **Every name is its own**, so two splits cannot read as one.
    #[test]
    fn no_two_splits_share_a_name() {
        let mut named: Vec<&str> = Split::EVERY_ONE.iter().map(|it| it.named()).collect();
        named.sort_unstable();
        let how_many = named.len();
        named.dedup();
        assert_eq!(named.len(), how_many);
    }

    /// **A name nothing declares is refused** rather than read as a part.
    #[test]
    fn a_split_nobody_declares_is_refused() {
        assert!(toml::from_str::<Held>("split = \"middle-third\"\n").is_err());
    }

    /// A table with one split in it, so the tests above go through TOML rather
    /// than through a serialiser nobody's file uses.
    #[derive(Serialize, Deserialize)]
    struct Held {
        /// The split.
        split: Split,
    }
}
