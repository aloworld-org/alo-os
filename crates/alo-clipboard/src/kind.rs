//! One form a copier can give what it copied in.
//!
//! `docs/features.md` promises **copy, cut and paste — text, images and files,
//! across applications**, and the shape of this crate turns on how those three
//! are read. They are **three offered forms and not three mechanisms**: an
//! owner says which forms it can produce, somebody asks for one of them, and
//! nothing anywhere switches on *what sort of thing* is being moved. A
//! clipboard with a text path, an image path and a file path in it would be
//! three implementations to keep honest, three sets of refusals, and a fourth
//! thing — a font, a spreadsheet range, a colour — arriving as a fourth path
//! nobody wrote.
//!
//! # A media type, because that is what the protocol carries
//!
//! Wayland's data device names forms as MIME types and so does every toolkit a
//! client is written against, so a [`Kind`] is a media type and this crate
//! invents no vocabulary of its own for them. The promise's three are the three
//! families a person recognises:
//!
//! | The promise says | A client offers | [`Form`] |
//! |---|---|---|
//! | text | `text/plain;charset=utf-8`, `text/html`, … | [`Form::Text`] |
//! | images | `image/png`, `image/jpeg`, … | [`Form::AnImage`] |
//! | files | `text/uri-list` | [`Form::Files`] |
//!
//! `text/uri-list` is checked **before** the `text/` family, because it is the
//! one media type whose family says the wrong thing: a list of file locations
//! is how every desktop moves files and it is spelt as text. Anything else is
//! [`Form::Another`] and is carried exactly as well as the three — it simply
//! has no name of ours to be shown by, and is shown as the type itself.
//!
//! # It arrives from a client, so it is checked like anything else that does
//!
//! An application offering a selection is not part of alo OS and its strings
//! are not trusted: a media type here is length-bounded, ASCII, and has a type
//! and a subtype. It is also **lowercased on the way in**, because media types
//! are case-insensitive and a paste refused because one client wrote
//! `TEXT/plain` and another asked for `text/plain` would be a refusal nobody
//! could see the cause of.

use alo_strings::{Filling, Strings};

use crate::words::{self, Word};

/// The longest a media type may be.
///
/// Long enough for a real one with its parameters —
/// `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet` is
/// sixty-five characters and is about as long as they get — and short enough
/// that a client cannot hand a compositor a selection whose list of forms is
/// the thing that costs the memory. It is a bound rather than a judgement: what
/// is past it is refused by name, not truncated into a different type.
pub const THE_LONGEST: usize = 255;

/// What the promise's three forms are called here, and everything else.
///
/// A closed enum rather than a set anybody can add to: the three are the three
/// `docs/features.md` names and each has a word a person reads. A fourth family
/// would be a string this crate says, which is a change to its vocabulary and
/// therefore a change somebody makes deliberately.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    /// Text of any flavour — plain, HTML, a rich-text fragment.
    Text,
    /// A picture.
    AnImage,
    /// A list of files, which is `text/uri-list` and nothing else.
    Files,
    /// Something with no name of ours: shown as the media type itself.
    Another,
}

impl Form {
    /// The string this crate declares for it, and nothing for [`Form::Another`]
    /// — which has no name of ours and is shown as the type a client wrote.
    #[must_use]
    pub fn word(self) -> Option<Word> {
        match self {
            Self::Text => Some(words::THE_FORM_TEXT),
            Self::AnImage => Some(words::THE_FORM_AN_IMAGE),
            Self::Files => Some(words::THE_FORM_FILES),
            Self::Another => None,
        }
    }
}

/// One form a copier can give what it copied in.
///
/// ```
/// use alo_clipboard::{Form, Kind};
///
/// let text = Kind::named("text/plain;charset=utf-8").expect("a media type");
/// assert_eq!(text.form(), Form::Text);
///
/// // Case is not a difference, because it is not a difference to the protocol.
/// assert_eq!(Kind::named("IMAGE/PNG").expect("a media type"), Kind::image_png());
///
/// // Files travel as a list of locations, which is spelt as text and is not.
/// assert_eq!(Kind::files().form(), Form::Files);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Kind {
    /// The media type, lowercased and checked.
    media: String,
}

impl Kind {
    /// A form, if what a client wrote is a media type.
    ///
    /// # Errors
    /// [`KindError`], which says what is wrong with it. Nothing is repaired
    /// beyond case: a type this cannot read is refused rather than guessed at,
    /// because a guess would be this crate deciding what an application meant
    /// to offer.
    pub fn named(media: &str) -> Result<Self, KindError> {
        let written = media.trim();
        if written.is_empty() {
            return Err(KindError::Empty);
        }
        if written.len() > THE_LONGEST {
            return Err(KindError::TooLong {
                how_long: written.len(),
            });
        }
        for character in written.chars() {
            if !character.is_ascii() || character.is_ascii_control() || character == ' ' {
                return Err(KindError::BadCharacter { character });
            }
        }
        let (of, rest) = written.split_once('/').ok_or(KindError::NoSubtype {
            media: written.to_owned(),
        })?;
        let subtype = rest.split(';').next().unwrap_or_default();
        if of.is_empty() || subtype.is_empty() {
            return Err(KindError::NoSubtype {
                media: written.to_owned(),
            });
        }
        Ok(Self {
            media: written.to_ascii_lowercase(),
        })
    }

    /// Plain text in the one encoding alo OS writes, which is the form nearly
    /// every copy offers.
    #[must_use]
    pub fn text() -> Self {
        Self {
            media: "text/plain;charset=utf-8".to_owned(),
        }
    }

    /// A PNG, which is the form nearly every image copy offers.
    #[must_use]
    pub fn image_png() -> Self {
        Self {
            media: "image/png".to_owned(),
        }
    }

    /// A list of files, which is how files move between applications.
    #[must_use]
    pub fn files() -> Self {
        Self {
            media: "text/uri-list".to_owned(),
        }
    }

    /// The media type, as it is written on the wire.
    #[must_use]
    pub fn media(&self) -> &str {
        &self.media
    }

    /// Which of the promise's three families this is, or [`Form::Another`].
    #[must_use]
    pub fn form(&self) -> Form {
        let without_parameters = self.media.split(';').next().unwrap_or_default().trim_end();
        if without_parameters == "text/uri-list" {
            Form::Files
        } else if without_parameters.starts_with("image/") {
            Form::AnImage
        } else if without_parameters.starts_with("text/") {
            Form::Text
        } else {
            Form::Another
        }
    }

    /// What to call this in a sentence a person reads.
    ///
    /// The promise's three have a word each and everything else is the media
    /// type itself — which is honest rather than tidy: a person told *this
    /// cannot be pasted as `application/x-krita-node`* has been handed the
    /// thing to search for, and a person told *this cannot be pasted as that*
    /// has been handed nothing.
    #[must_use]
    pub fn shown(&self, strings: &Strings) -> String {
        match self.form().word() {
            Some(word) => strings.say(&word.key(), &Filling::nothing()).into_text(),
            None => self.media.clone(),
        }
    }
}

/// Why what a client wrote is not a form anything can be offered in.
///
/// Not a refusal anybody reads in their own language: nothing on the far side
/// is a person, and what has gone wrong is in an application rather than on
/// this machine. A compositor that is handed one has one thing to do with it,
/// which is to ignore that form and keep the rest of the offer — see
/// [`crate::Offer`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum KindError {
    /// Nothing, or only spaces.
    #[error("a form with no media type in it is not a form anything can be offered in")]
    Empty,

    /// Longer than [`THE_LONGEST`].
    #[error(
        "a media type of {how_long} characters is longer than the {THE_LONGEST} a real one needs, and a clipboard does not hold what a client says without a bound on it"
    )]
    TooLong {
        /// How long the one that arrived was.
        how_long: usize,
    },

    /// Something a media type cannot contain: a space, a control character, or
    /// anything outside ASCII.
    #[error(
        "a media type cannot contain {character:?} — it is written in ASCII, without spaces, and is not somewhere to put a sentence"
    )]
    BadCharacter {
        /// What was in it.
        character: char,
    },

    /// No `/`, or nothing on one side of it.
    #[error("{media} is not a media type: it needs a type and a subtype, as `text/plain` does")]
    NoSubtype {
        /// What arrived.
        media: String,
    },
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// **The three the promise names are three forms**, read off the media type
    /// a client writes and not off three different doors into this crate.
    #[test]
    fn the_promises_three_are_three_forms() {
        assert_eq!(Kind::text().form(), Form::Text);
        assert_eq!(Kind::image_png().form(), Form::AnImage);
        assert_eq!(Kind::files().form(), Form::Files);
    }

    /// Every family a real client offers lands where a person would put it, and
    /// `text/uri-list` lands in files rather than in text — which is the one
    /// case where the media type's own family says the wrong thing.
    #[test]
    fn a_real_clients_forms_land_where_a_person_would_put_them() {
        for (media, form) in [
            ("text/plain", Form::Text),
            ("text/plain;charset=utf-8", Form::Text),
            ("text/html", Form::Text),
            ("text/uri-list", Form::Files),
            ("text/uri-list;charset=utf-8", Form::Files),
            ("image/png", Form::AnImage),
            ("image/svg+xml", Form::AnImage),
            ("application/pdf", Form::Another),
            ("application/x-krita-node", Form::Another),
        ] {
            assert_eq!(Kind::named(media).unwrap().form(), form, "{media}");
        }
    }

    /// **Case is not a difference.** Two clients writing one media type two
    /// ways are offering and asking for one form, and a paste refused over the
    /// shift key would be a refusal nobody could see the cause of.
    #[test]
    fn case_is_not_a_difference() {
        assert_eq!(
            Kind::named("TEXT/PLAIN").unwrap(),
            Kind::named("text/plain").unwrap()
        );
        assert_eq!(Kind::named("Image/PNG").unwrap(), Kind::image_png());
        assert_eq!(Kind::named("TEXT/URI-LIST").unwrap().form(), Form::Files);
    }

    /// Surrounding space is a client's whitespace rather than a different type,
    /// and it is trimmed rather than refused.
    #[test]
    fn space_around_a_media_type_is_not_part_of_it() {
        assert_eq!(
            Kind::named("  text/plain  ").unwrap(),
            Kind::named("text/plain").unwrap()
        );
    }

    /// **What is not a media type is refused by name**, one wrong shape at a
    /// time. Each of these arrives from an application rather than from this
    /// repository, which is why none of them is an assertion.
    #[test]
    fn what_is_not_a_media_type_is_refused() {
        assert_eq!(Kind::named(""), Err(KindError::Empty));
        assert_eq!(Kind::named("   "), Err(KindError::Empty));
        for media in ["text", "/plain", "text/", "text/;charset=utf-8"] {
            assert!(
                matches!(Kind::named(media), Err(KindError::NoSubtype { .. })),
                "{media} was accepted"
            );
        }
        for media in [
            "text/pl ain",
            "text/pl\nain",
            "text/pläin",
            "text\u{0}/plain",
        ] {
            assert!(
                matches!(Kind::named(media), Err(KindError::BadCharacter { .. })),
                "{media} was accepted"
            );
        }
    }

    /// **A form longer than a real one is refused rather than truncated**, and
    /// the refusal says how long it was. Truncating would turn a client's
    /// mistake into a different type that something else might match.
    #[test]
    fn a_form_longer_than_any_real_one_is_refused() {
        let longest = format!("text/{}", "x".repeat(THE_LONGEST - 5));
        assert_eq!(Kind::named(&longest).unwrap().media().len(), THE_LONGEST);

        let over = format!("text/{}", "x".repeat(THE_LONGEST));
        assert_eq!(
            Kind::named(&over),
            Err(KindError::TooLong {
                how_long: THE_LONGEST + 5
            })
        );
    }

    /// **The three have a name a person reads, and everything else is shown as
    /// itself** — which hands somebody the thing to search for instead of the
    /// word *that*.
    #[test]
    fn the_three_are_named_and_everything_else_is_shown_as_itself() {
        let strings = in_english();
        assert_eq!(Kind::text().shown(&strings), "text");
        assert_eq!(Kind::image_png().shown(&strings), "an image");
        assert_eq!(Kind::files().shown(&strings), "files");
        assert_eq!(
            Kind::named("application/x-krita-node")
                .unwrap()
                .shown(&strings),
            "application/x-krita-node"
        );
    }
}
