//! The colours a person can make their own, and the one they cannot.
//!
//! **Terracotta is reserved** (ADR 0010). It means the agent — present, acting,
//! or waiting for an approval — and it means nothing else anywhere in the
//! system, so it is not in this set and [`Accent::of_colour`] refuses it in
//! words rather than quietly accepting it. An accent somebody could set to
//! terracotta would take away the one signal that says the machine is doing
//! something on their behalf, and it would take it away on precisely the
//! machines whose owners liked the colour enough to choose it.
//!
//! **The set is designed rather than derived**, which is the other half of that
//! decision. `docs/features.md` used to promise an accent "drawn from the design
//! tokens", and the tokens cannot answer it: the five that are not terracotta
//! are grounds and structure, and navy on the charcoal rail or cream on the
//! cream ground is not an accent, it is invisible. So there are five hues from
//! outside the palette, each with a value for a light ground and a value for a
//! dark one — because a single hex that reads on cream is illegible on charcoal,
//! and a person who works in the evening is not owed a worse accent than one who
//! works in the morning.
//!
//! **Every value here is measured, not asserted.** ADR 0010 says outright that
//! the hexes are a designer's proposal rather than a measurement; the tests
//! below are that measurement, through [`crate::contrast`], against the grounds
//! the design brief names. A hue added to this set that does not read on both
//! grounds fails to compile a release rather than reaching somebody who cannot
//! read it.
//!
//! What this module deliberately does **not** carry is ADR 0010's other half —
//! that the agent always appears with a mark and a word beside it. That is true
//! of every screen the shell draws and cannot be true of a colour, so it belongs
//! where the drawing happens rather than here.

use alo_strings::{Filling, Said, Strings, Word};
use serde::{Deserialize, Serialize};

use crate::colour::Colour;
use crate::scheme::Scheme;
use crate::token::Token;
use crate::words;

/// Why a colour is not an accent a person can choose.
///
/// All three say what to choose instead, because somebody seeing one of these
/// is in a settings panel wanting a colour rather than wanting an explanation.
/// They are three sentences rather than one because they are three different
/// mistakes: asking for the agent's colour, asking for a colour the system is
/// built out of, and asking for a colour nobody designed.
///
/// There is no `Display`, and therefore no `std::error::Error`: the only road to
/// words is [`AccentError::said`], which takes the strings this machine reads.
/// **A refusal and everything inside it are in one language** —
/// [`AccentError::NotAnAccent`] names a [`Token`], and what goes into the
/// sentence is that token said in the reader's own language rather than an
/// English word left in the middle of a translated one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccentError {
    /// Terracotta, which is the agent's and nobody else's.
    Reserved,
    /// A ground or a piece of structure: a colour the system is built out of
    /// rather than one it can be accented with.
    NotAnAccent(Token),
    /// A colour from somewhere else entirely.
    NotOffered(Colour),
}

impl AccentError {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Reserved => words::RESERVED,
            Self::NotAnAccent(_) => words::NOT_AN_ACCENT,
            Self::NotOffered(_) => words::NOT_OFFERED,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// The colour's **name** goes in as a word and the colour's **value** goes
    /// in as data, which is the whole difference between the two refusals: a
    /// person who asked for terracotta is told the name of a colour somebody
    /// translated, and a person who typed a hex is shown back what they typed.
    /// So this refusal is only as translated as the name inside it, and a hex
    /// cannot make it less translated than it is.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not. A
    /// `Strings` that was never given [`crate::appearance_words`] answers with
    /// the key, marked, and `Said::is_a_bug`.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        let filling = match self {
            Self::Reserved => Filling::nothing(),
            Self::NotAnAccent(token) => Filling::nothing().and_said("colour", &token.said(strings)),
            Self::NotOffered(colour) => Filling::of("colour", colour.to_string()),
        };
        strings.say(&self.word().key(), &filling)
    }
}

/// One of the five colours a person can make their machine.
///
/// Reads back from a settings file by name, and a file naming anything else —
/// including terracotta — is refused where it is read rather than becoming a
/// colour nobody offered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Accent {
    /// Blue-green. What a machine ships with.
    Verdigris,
    /// Deep blue.
    Indigo,
    /// Purple.
    Violet,
    /// Green.
    Moss,
    /// Deep pink.
    Rose,
}

impl Accent {
    /// All five, in the order ADR 0010 lists them, which is the order a picker
    /// shows them in.
    pub const ALL: [Self; 5] = [
        Self::Verdigris,
        Self::Indigo,
        Self::Violet,
        Self::Moss,
        Self::Rose,
    ];

    /// The colour itself, on the ground this scheme puts behind it.
    ///
    /// The same accent is two hexes, and which one is showing is not a
    /// preference: [`Scheme::Light`] is the value drawn on cream and porcelain,
    /// [`Scheme::Dark`] the value drawn on charcoal. A machine that switched to
    /// dark and kept the light value would be a machine whose accent stopped
    /// being readable at six in the evening.
    #[must_use]
    pub const fn on(self, scheme: Scheme) -> Colour {
        match (self, scheme) {
            (Self::Verdigris, Scheme::Light) => Colour::of(0x22, 0x70, 0x7E),
            (Self::Verdigris, Scheme::Dark) => Colour::of(0x5F, 0xB3, 0xC2),
            (Self::Indigo, Scheme::Light) => Colour::of(0x3A, 0x5A, 0xA8),
            (Self::Indigo, Scheme::Dark) => Colour::of(0x8A, 0xA0, 0xE6),
            (Self::Violet, Scheme::Light) => Colour::of(0x7A, 0x4E, 0x99),
            (Self::Violet, Scheme::Dark) => Colour::of(0xBE, 0x97, 0xDE),
            (Self::Moss, Scheme::Light) => Colour::of(0x4A, 0x75, 0x46),
            (Self::Moss, Scheme::Dark) => Colour::of(0x8D, 0xBE, 0x85),
            (Self::Rose, Scheme::Light) => Colour::of(0xA0, 0x46, 0x6A),
            (Self::Rose, Scheme::Dark) => Colour::of(0xE0, 0x93, 0xAF),
        }
    }

    /// The string this crate declares for it: the key a translator's file is
    /// sorted by, and the English beside it.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Verdigris => words::VERDIGRIS,
            Self::Indigo => words::INDIGO,
            Self::Violet => words::VIOLET,
            Self::Moss => words::MOSS,
            Self::Rose => words::ROSE,
        }
    }

    /// What this accent is called where a person picks it, in the language they
    /// read.
    ///
    /// The same kind of translator's judgement as [`Token::said`]: verdigris is
    /// the colour of weathered copper, and several languages say it with two
    /// words or with none. Never fails and never panics.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }

    /// The accent a colour is, if it is one of them.
    ///
    /// **This is the door a colour arrives at, and the refusal that makes ADR
    /// 0010 true rather than written down.** A settings file holds an accent by
    /// name, so it never comes here; a colour does — from a panel that let
    /// somebody type one, from `docs/features.md`'s old promise of an accent
    /// "drawn from the design tokens", and at v1 from an agent asked to *make
    /// the accent this colour*, which is a proposal a person approves and must
    /// therefore be refusable in a sentence they can read.
    ///
    /// Either value of an accent answers with that accent, because *this one,
    /// the one I am looking at* is the same choice whether the machine is light
    /// or dark at the moment it is made.
    ///
    /// # Errors
    /// [`AccentError::Reserved`] for terracotta, [`AccentError::NotAnAccent`]
    /// for the rest of the palette, [`AccentError::NotOffered`] for anything
    /// else.
    pub fn of_colour(colour: Colour) -> Result<Self, AccentError> {
        for accent in Self::ALL {
            if accent.on(Scheme::Light) == colour || accent.on(Scheme::Dark) == colour {
                return Ok(accent);
            }
        }
        for token in Token::ALL {
            if token.colour() == colour {
                return Err(match token {
                    Token::Terracotta => AccentError::Reserved,
                    other => AccentError::NotAnAccent(other),
                });
            }
        }
        Err(AccentError::NotOffered(colour))
    }
}

impl Default for Accent {
    /// Verdigris, which is a small piece of continuity: it is the name
    /// `alo-workplace`'s colour scale still carries from before the palette
    /// became terracotta.
    fn default() -> Self {
        Self::Verdigris
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::contrast::ENOUGH_FOR_TEXT;
    use crate::testing::{in_english, translated};

    /// How far a hue has to sit from terracotta's before this crate will say
    /// they are different colours. Thirty degrees is a floor rather than a
    /// perceptual claim — see [`no_accent_sits_where_terracotta_sits`].
    const FAR_ENOUGH_FROM_TERRACOTTA: f64 = 30.0;

    /// Where a colour sits on the wheel, 0 to 360, which is the one question
    /// *is this the agent's colour* reduces to when the answer has to be
    /// arithmetic.
    fn hue(colour: Colour) -> f64 {
        let red = f64::from(colour.red()) / 255.0;
        let green = f64::from(colour.green()) / 255.0;
        let blue = f64::from(colour.blue()) / 255.0;
        let most = red.max(green).max(blue);
        let least = red.min(green).min(blue);
        let spread = most - least;
        if spread < f64::EPSILON {
            return 0.0;
        }
        let sixth = if (most - red).abs() < f64::EPSILON {
            ((green - blue) / spread).rem_euclid(6.0)
        } else if (most - green).abs() < f64::EPSILON {
            (blue - red) / spread + 2.0
        } else {
            (red - green) / spread + 4.0
        };
        (sixth * 60.0).rem_euclid(360.0)
    }

    /// How far apart two hues are, the short way round the wheel.
    fn apart(one: f64, other: f64) -> f64 {
        let straight = (one - other).abs().rem_euclid(360.0);
        straight.min(360.0 - straight)
    }

    /// **Terracotta cannot be chosen, and the refusal says so in words.** It is
    /// not in the set, so no settings file and no picker can name it; asking for
    /// it through the design tokens comes back as a sentence naming the five
    /// that can be had instead.
    #[test]
    fn terracotta_is_refused_as_a_personal_accent() {
        let asked = Accent::of_colour(Token::Terracotta.colour());
        assert_eq!(asked, Err(AccentError::Reserved));
        assert!(
            asked
                .unwrap_err()
                .said(&in_english())
                .text()
                .contains("not offered as a personal accent")
        );

        for accent in Accent::ALL {
            for scheme in [Scheme::Light, Scheme::Dark] {
                assert_ne!(
                    accent.on(scheme),
                    Token::Terracotta.colour(),
                    "{} on {scheme:?} is not terracotta by another name",
                    accent.word().says()
                );
            }
        }
        assert!(
            serde_json::from_str::<Accent>(r#""Terracotta""#).is_err(),
            "and a hand-edited settings file cannot ask for it either"
        );
    }

    /// **Nothing in the palette is an accent**, including the five that are not
    /// reserved: they are grounds and structure, and the refusal for those says
    /// something different from the refusal for terracotta because they are
    /// different mistakes.
    #[test]
    fn a_ground_or_a_structure_colour_is_refused_as_itself() {
        let strings = in_english();
        for token in Token::ALL {
            let refused = Accent::of_colour(token.colour()).unwrap_err();
            let said = refused.said(&strings);
            assert!(said.unfilled().is_empty(), "{said}");
            match token {
                Token::Terracotta => assert_eq!(refused, AccentError::Reserved),
                other => {
                    assert_eq!(refused, AccentError::NotAnAccent(other));
                    assert!(
                        said.text().contains(other.said(&strings).text()),
                        "the refusal names the colour that was asked for"
                    );
                }
            }
            assert!(
                said.text().contains("verdigris"),
                "and every refusal says what can be had instead"
            );
        }
    }

    /// **A refusal and everything inside it are in one language.** The colour
    /// the sentence names is one of this crate's own strings, so a German
    /// machine does not read a German sentence with an English colour in the
    /// middle of it.
    #[test]
    fn a_refusal_names_the_colour_in_the_readers_language() {
        let strings = translated(&[
            (words::NAVY, "Marineblau"),
            (
                words::NOT_AN_ACCENT,
                "{colour} ist eine Grund- oder Strukturfarbe und keine Akzentfarbe — wählen Sie \
                 Grünspan, Indigo, Violett, Moos oder Rosé",
            ),
        ]);
        let said = Accent::of_colour(Token::Navy.colour())
            .unwrap_err()
            .said(&strings);
        assert_eq!(
            said.text(),
            "Marineblau ist eine Grund- oder Strukturfarbe und keine Akzentfarbe — wählen Sie \
             Grünspan, Indigo, Violett, Moos oder Rosé"
        );
        assert!(said.is_translated());
        assert!(said.unfilled().is_empty());
    }

    /// **A refusal is only as translated as the colour named inside it**
    /// (item 15). A colour name is the one string in this crate that carries
    /// none of itself — *Grünspan* is not reachable from *verdigris* word by
    /// word — so a German sentence with an English colour in it is a line whose
    /// reader is told to pick something they have never seen written down.
    #[test]
    fn a_refusal_naming_an_untranslated_colour_does_not_claim_to_be_translated() {
        let half = translated(&[(
            words::NOT_AN_ACCENT,
            "{colour} ist eine Grund- oder Strukturfarbe und keine Akzentfarbe — wählen Sie \
             Grünspan, Indigo, Violett, Moos oder Rosé",
        )]);
        let said = Accent::of_colour(Token::Navy.colour())
            .unwrap_err()
            .said(&half);
        assert!(!said.is_translated(), "{said}");
        assert!(said.text().starts_with("Navy ist eine"), "{said}");
    }

    /// **A colour somebody typed is data, and data cannot make a refusal
    /// untranslated.** Nobody translates `#7f4a2d`, so a German refusal quoting
    /// one back is a German refusal — which is the distinction the two variants
    /// of this error exist to make.
    #[test]
    fn a_refusal_quoting_a_value_back_is_as_translated_as_it_reads() {
        let strings = translated(&[(
            words::NOT_OFFERED,
            "{colour} ist keine der Farben, die alo OS anbietet — wählen Sie eine davon",
        )]);
        let said = Accent::of_colour(Colour::of(0x7f, 0x4a, 0x2d))
            .unwrap_err()
            .said(&strings);
        assert!(said.is_translated(), "{said}");
        assert!(said.text().starts_with("#7F4A2D"), "{said}");
    }

    /// A machine that was never given this crate's words still refuses exactly
    /// what it refused before, and says which rule it broke rather than
    /// pretending to a sentence. **A refusal never depends on a string table.**
    #[test]
    fn a_refusal_without_the_words_still_names_the_rule() {
        let nothing_declared = Strings::of(alo_strings::Vocabulary::empty());
        let said = Accent::of_colour(Token::Terracotta.colour())
            .unwrap_err()
            .said(&nothing_declared);
        assert!(said.is_a_bug());
        assert!(said.text().contains("appearance.accent.reserved"), "{said}");
    }

    /// **A colour from somewhere else is refused too**, and each of the ten
    /// designed values answers with the accent it belongs to — either value,
    /// because *this one, the one I am looking at* is the same choice whether
    /// the machine is light or dark at the moment it is made.
    #[test]
    fn a_colour_is_the_accent_it_is_or_it_is_refused() {
        for accent in Accent::ALL {
            for scheme in [Scheme::Light, Scheme::Dark] {
                assert_eq!(Accent::of_colour(accent.on(scheme)), Ok(accent));
            }
        }

        let invented = Colour::written("#123456").unwrap();
        assert_eq!(
            Accent::of_colour(invented),
            Err(AccentError::NotOffered(invented))
        );
        assert!(
            Accent::of_colour(invented)
                .unwrap_err()
                .said(&in_english())
                .text()
                .contains("#123456"),
            "the refusal says which colour was asked for, and a hex is not translated"
        );
    }

    /// **Every accent reads on the ground it is drawn on**, measured rather
    /// than asserted: the light-ground value against both cream and the
    /// porcelain canvas, the dark-ground value against the charcoal rail, at
    /// the ratio EN 301 549 requires of ordinary text. This is ADR 0010's
    /// "wants contrast verified before they ship", and it is why a sixth hue
    /// cannot be added without measuring it.
    ///
    /// Text is the harder of the two thresholds — [`crate::contrast`]'s 4.5 is
    /// above its 3.0 for a shape — so an accent that clears it clears a fill and
    /// a focus ring drawn in the same colour as well.
    #[test]
    fn every_accent_reads_on_both_grounds() {
        let light_grounds = [Token::Cream, Token::Porcelain];
        for accent in Accent::ALL {
            for ground in light_grounds {
                let measured = accent.on(Scheme::Light).contrast_with(ground.colour());
                assert!(
                    measured >= ENOUGH_FOR_TEXT,
                    "{} on {} measured {measured}",
                    accent.word().says(),
                    ground.word().says()
                );
            }
            let measured = accent
                .on(Scheme::Dark)
                .contrast_with(Token::Charcoal.colour());
            assert!(
                measured >= ENOUGH_FOR_TEXT,
                "{} on charcoal measured {measured}",
                accent.word().says()
            );
        }
    }

    /// **The two values of one accent are the same colour, not two colours.**
    /// A light-ground and a dark-ground value that drifted apart in hue would be
    /// a machine that changes accent at six in the evening, which nobody asked
    /// it to do.
    #[test]
    fn an_accent_is_one_hue_in_two_strengths() {
        for accent in Accent::ALL {
            let light = accent.on(Scheme::Light);
            let dark = accent.on(Scheme::Dark);
            assert_ne!(
                light,
                dark,
                "{} needs a value per ground",
                accent.word().says()
            );
            assert!(
                apart(hue(light), hue(dark)) <= 15.0,
                "{} is {} degrees apart from itself",
                accent.word().says(),
                apart(hue(light), hue(dark))
            );
            assert!(
                dark.relative_luminance() > light.relative_luminance(),
                "{} is the lighter of the two on a dark ground",
                accent.word().says()
            );
        }
    }

    /// **No accent sits where terracotta sits.** Hue distance is arithmetic and
    /// not a claim about perception — deuteranopia makes terracotta and moss
    /// neighbours whatever the wheel says, which is exactly why the agent gets a
    /// mark and a word as well. What this catches is the thing arithmetic can
    /// catch: a hue added later that is terracotta with two digits changed.
    #[test]
    fn no_accent_sits_where_terracotta_sits() {
        let reserved = hue(Token::Terracotta.colour());
        for accent in Accent::ALL {
            for scheme in [Scheme::Light, Scheme::Dark] {
                let distance = apart(hue(accent.on(scheme)), reserved);
                assert!(
                    distance >= FAR_ENOUGH_FROM_TERRACOTTA,
                    "{} on {scheme:?} is {distance} degrees from terracotta",
                    accent.word().says()
                );
            }
        }
    }

    /// Every accent is named, and no two share a name or a value — a picker
    /// with two identical entries is a picker a person cannot use.
    #[test]
    fn every_accent_is_named_and_distinct() {
        let strings = in_english();
        for (at, accent) in Accent::ALL.iter().enumerate() {
            assert!(!accent.said(&strings).text().is_empty());
            assert!(!accent.said(&strings).is_a_bug(), "{accent:?}");
            for other in Accent::ALL.iter().skip(at.saturating_add(1)) {
                assert_ne!(accent.said(&strings).text(), other.said(&strings).text());
                assert_ne!(accent.on(Scheme::Light), other.on(Scheme::Light));
                assert_ne!(accent.on(Scheme::Dark), other.on(Scheme::Dark));
            }
        }
    }

    /// **The set is the one in ADR 0010**, hex by hex, so a value changed here
    /// without the decision changing is caught by the person who wrote it.
    #[test]
    fn the_set_is_the_one_in_the_decision() {
        let written = [
            (Accent::Verdigris, "#22707E", "#5FB3C2"),
            (Accent::Indigo, "#3A5AA8", "#8AA0E6"),
            (Accent::Violet, "#7A4E99", "#BE97DE"),
            (Accent::Moss, "#4A7546", "#8DBE85"),
            (Accent::Rose, "#A0466A", "#E093AF"),
        ];
        for (accent, light, dark) in written {
            assert_eq!(accent.on(Scheme::Light).to_string(), light);
            assert_eq!(accent.on(Scheme::Dark).to_string(), dark);
        }
        assert_eq!(written.len(), Accent::ALL.len(), "all five, and only five");
        assert_eq!(Accent::default(), Accent::Verdigris);
    }

    /// An accent survives a settings file, by name rather than by hex — so a
    /// release that corrects a value corrects it for everybody who chose that
    /// colour, rather than freezing the hex they happened to be shown.
    #[test]
    fn an_accent_is_written_down_by_name() {
        assert_eq!(
            serde_json::to_string(&Accent::Moss).unwrap(),
            r#""Moss""#,
            "a name a person can read in their own settings file"
        );
        for accent in Accent::ALL {
            let written = serde_json::to_string(&accent).unwrap();
            assert_eq!(serde_json::from_str::<Accent>(&written).unwrap(), accent);
        }
    }
}
