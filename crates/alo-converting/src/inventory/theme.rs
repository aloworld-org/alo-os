//! The two fonts a document's theme names, which text set in *the theme's
//! font* is shown in.
//!
//! All three formats let text say *the heading font* or *the body font* rather
//! than a family, and the theme part says which families those are. Word calls
//! them `majorHAnsi` and `minorHAnsi`, PowerPoint `+mj-lt` and `+mn-lt`; both
//! arrive here as a [`Slot`].

use crate::xml::{self, NotXml, Read};

/// Which of the theme's two fonts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    /// The heading font.
    Major,
    /// The body font.
    Minor,
}

/// The Latin-script families a theme names.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Theme {
    /// The heading font.
    major: Option<String>,
    /// The body font.
    minor: Option<String>,
}

impl Theme {
    /// A theme part, read.
    ///
    /// # Errors
    /// [`NotXml`].
    pub fn of(bytes: &[u8]) -> Result<Self, NotXml> {
        let mut theme = Self::default();
        let mut inside: Option<Slot> = None;
        for read in xml::read(bytes)? {
            if read.opens("majorFont") {
                inside = Some(Slot::Major);
            } else if read.opens("minorFont") {
                inside = Some(Slot::Minor);
            } else if read.closes("majorFont") || read.closes("minorFont") {
                inside = None;
            } else if read.opens("latin") {
                let typeface = read.attribute("typeface").map(str::to_owned);
                match inside {
                    Some(Slot::Major) if theme.major.is_none() => theme.major = typeface,
                    Some(Slot::Minor) if theme.minor.is_none() => theme.minor = typeface,
                    _ => {}
                }
            }
        }
        Ok(theme)
    }

    /// The family in one slot, when the theme names one.
    #[must_use]
    pub fn family(&self, slot: Slot) -> Option<&str> {
        match slot {
            Slot::Major => self.major.as_deref(),
            Slot::Minor => self.minor.as_deref(),
        }
        .filter(|family| !family.trim().is_empty())
    }
}

/// Whether a part read has any text in it that shows.
pub(crate) fn shows(read: &Read) -> bool {
    matches!(read, Read::Text(text) if !text.trim().is_empty())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_part, the_document};

    /// **A real theme names its two fonts.**
    #[test]
    fn a_real_theme_names_its_two_fonts() {
        let theme = Theme::of(&a_part(
            &the_document("sample.pptx"),
            "ppt/theme/theme1.xml",
        ))
        .unwrap();
        assert_eq!(theme.family(Slot::Major), Some("Aptos Display"));
        assert_eq!(theme.family(Slot::Minor), Some("Aptos"));
    }

    /// A theme with no fonts names none, and an empty name is no name.
    #[test]
    fn a_theme_without_fonts_names_none() {
        let theme =
            Theme::of(br#"<theme><minorFont><latin typeface=""/></minorFont></theme>"#).unwrap();
        assert_eq!(theme.family(Slot::Major), None);
        assert_eq!(theme.family(Slot::Minor), None);
    }
}
