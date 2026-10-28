//! **A region, as `formats.toml` keeps it**: two capital letters, and nothing
//! else about the person.
//!
//! The file kept a language beside the region until 2026-10-28, so a person who
//! chose Germany while reading English and then changed to French went on
//! reading English month names. The owner's decision of that day: **a region
//! decides the order, the reading language decides the words**, and the
//! language is asked of the person's own `settings.toml` every time rather than
//! copied into this file once. So the file holds only this.

use serde::{Deserialize, Serialize};

use crate::regionally::NotARegion;

/// **A region code**: `BE`, `DE`, `XK`.
///
/// Checked for its shape and nothing more — two capital letters — which is
/// what CLDR is asked with. Whether it is on [`crate::regions::EVERY_REGION`]
/// is a question for what offers regions, not for what reads one back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RegionCode(String);

impl RegionCode {
    /// The region `code` names.
    ///
    /// # Errors
    /// [`NotARegion::NotARegion`] for anything that is not two capital letters.
    pub fn named(code: &str) -> Result<Self, NotARegion> {
        if code.len() != 2 || !code.chars().all(|letter| letter.is_ascii_uppercase()) {
            return Err(NotARegion::NotARegion {
                asked_for: code.to_owned(),
            });
        }
        Ok(Self(code.to_owned()))
    }

    /// The two letters.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RegionCode {
    type Error = NotARegion;

    fn try_from(code: String) -> Result<Self, Self::Error> {
        Self::named(&code)
    }
}

impl From<RegionCode> for String {
    fn from(region: RegionCode) -> Self {
        region.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Two capital letters, and nothing else**, refused naming what was
    /// asked for.
    #[test]
    fn only_two_capital_letters_are_a_region() {
        assert_eq!(RegionCode::named("XK").map(|it| it.0), Ok("XK".to_owned()));
        for wrong in ["be", "BEL", "B", "", "Belgium", "B1"] {
            assert_eq!(
                RegionCode::named(wrong),
                Err(NotARegion::NotARegion {
                    asked_for: wrong.to_owned()
                }),
                "{wrong}"
            );
        }
    }
}
