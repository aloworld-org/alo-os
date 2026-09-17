//! Which keyboard is offered with which language, for all 24.
//!
//! **This is the only keyboard data alo OS owns**, and it is one line per
//! language: not what the keys do — that is rented — but which of the rented
//! keyboards a person who has just chosen Greek should be handed. Choosing a
//! language and then having to hunt through two hundred entries for the Greek
//! keyboard is the failure this table exists to prevent, and it is a failure
//! that lands hardest on exactly the people `docs/features.md` refuses to
//! leave out: there is always an English keyboard in front of somebody, and
//! there is not always a Maltese one.
//!
//! Every name here is checked twice by this crate's tests: that it is a name at
//! all, and that **the rented data on this machine actually has it**. A line
//! here naming a layout `xkeyboard-config` dropped is a language that would
//! silently offer nothing, so it fails rather than shipping.
//!
//! # The offer is a starting point, not a verdict
//!
//! A language is not a country and a country is not a keyboard. German is
//! typed on `de` in Germany and on `ch` in Switzerland; French is `fr` in
//! France and `be` in Belgium; Dutch is typed on a US keyboard in the
//! Netherlands and on a Belgian one in Flanders. So each language names one
//! keyboard to start with **and the others its speakers use**
//! ([`Offer::also`]), which is what [`crate::Keyboards::add_for`] and a setup
//! list offer next — a person who needs the other one finds it in front of
//! them rather than in an alphabetical list of the world.
//!
//! Two of these deserve saying out loud, because both look like mistakes:
//!
//! - **Dutch is offered `us(intl)`**, not `nl`. The Netherlands types on a US
//!   keyboard; the `nl` layout exists and almost nobody uses it. `us(intl)`
//!   has the dead keys that put the diaereses and the accents Dutch needs on a
//!   keyboard whose keycaps say what is printed on the machine a person
//!   bought. `be` and `nl` are both beside it.
//! - **English is offered `us`**, with `gb` and `ie` beside it. English is the
//!   language this repository is written in and the one every machine can
//!   already type; `us` is the keyboard most of them have.

use alo_strings::Language;

use crate::layout::Layout;

/// The keyboard offered with one language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Offer {
    /// The language, as [`alo_strings::union::OFFICIAL`] tags it.
    pub language: &'static str,
    /// The layout offered first.
    pub layout: &'static str,
    /// Its variant, when the offer is a variant of a layout.
    pub variant: Option<&'static str>,
    /// The other keyboards this language is typed on, offered beside it.
    pub also: &'static [(&'static str, Option<&'static str>)],
}

impl Offer {
    /// The keyboard offered first.
    #[must_use]
    pub fn first(&self) -> Layout {
        Layout::offered(self.layout, self.variant)
    }

    /// The others, in the order they are offered.
    #[must_use]
    pub fn beside_it(&self) -> Vec<Layout> {
        self.also
            .iter()
            .map(|(layout, variant)| Layout::offered(layout, *variant))
            .collect()
    }
}

/// Every language `alo-strings` carries, and the keyboard it is typed on.
///
/// In the order [`alo_strings::union::OFFICIAL`] lists them, which is
/// alphabetical by the English name because that is the order the Union
/// publishes — so the two lists can be read side by side.
pub const EVERY_OFFER: [Offer; 24] = [
    Offer {
        language: "bg",
        layout: "bg",
        variant: None,
        also: &[("bg", Some("phonetic"))],
    },
    Offer {
        language: "hr",
        layout: "hr",
        variant: None,
        also: &[],
    },
    Offer {
        language: "cs",
        layout: "cz",
        variant: None,
        also: &[("cz", Some("qwerty"))],
    },
    Offer {
        language: "da",
        layout: "dk",
        variant: None,
        also: &[],
    },
    Offer {
        language: "nl",
        layout: "us",
        variant: Some("intl"),
        also: &[("be", None), ("nl", None)],
    },
    Offer {
        language: "en",
        layout: "us",
        variant: None,
        also: &[("gb", None), ("ie", None)],
    },
    Offer {
        language: "et",
        layout: "ee",
        variant: None,
        also: &[],
    },
    Offer {
        language: "fi",
        layout: "fi",
        variant: None,
        also: &[],
    },
    Offer {
        language: "fr",
        layout: "fr",
        variant: None,
        also: &[("be", None), ("ca", None), ("ch", Some("fr"))],
    },
    Offer {
        language: "de",
        layout: "de",
        variant: None,
        also: &[("at", None), ("ch", None), ("de", Some("nodeadkeys"))],
    },
    Offer {
        language: "el",
        layout: "gr",
        variant: None,
        also: &[("gr", Some("polytonic"))],
    },
    Offer {
        language: "hu",
        layout: "hu",
        variant: None,
        also: &[],
    },
    Offer {
        language: "ga",
        layout: "ie",
        variant: None,
        also: &[("gb", None)],
    },
    Offer {
        language: "it",
        layout: "it",
        variant: None,
        also: &[("ch", None)],
    },
    Offer {
        language: "lv",
        layout: "lv",
        variant: None,
        also: &[],
    },
    Offer {
        language: "lt",
        layout: "lt",
        variant: None,
        also: &[],
    },
    Offer {
        language: "mt",
        layout: "mt",
        variant: None,
        also: &[("gb", None)],
    },
    Offer {
        language: "pl",
        layout: "pl",
        variant: None,
        also: &[],
    },
    Offer {
        language: "pt",
        layout: "pt",
        variant: None,
        also: &[("br", None)],
    },
    Offer {
        language: "ro",
        layout: "ro",
        variant: None,
        also: &[],
    },
    Offer {
        language: "sk",
        layout: "sk",
        variant: None,
        also: &[("sk", Some("qwerty"))],
    },
    Offer {
        language: "sl",
        layout: "si",
        variant: None,
        also: &[],
    },
    Offer {
        language: "es",
        layout: "es",
        variant: None,
        also: &[("latam", None)],
    },
    Offer {
        language: "sv",
        layout: "se",
        variant: None,
        also: &[],
    },
];

/// What this language is typed on, or `None` for a language alo OS has no
/// offer for.
///
/// Answered on the language itself and not its region: Portuguese is `pt`
/// whether it is written `pt` or `pt-BR`, and `br` is the keyboard beside it.
#[must_use]
pub fn offered_with(language: &Language) -> Option<Offer> {
    EVERY_OFFER
        .into_iter()
        .find(|offer| offer.language == language.primary())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// **Every language `alo-strings` carries has a keyboard**, and no language
    /// is in this list twice. The named test per language is in
    /// `tests/a_keyboard_for_every_language.rs`, which also asks the rented
    /// data on this machine whether the keyboard exists.
    #[test]
    fn every_official_language_is_offered_a_keyboard() {
        let offered: BTreeSet<&str> = EVERY_OFFER.iter().map(|offer| offer.language).collect();
        assert_eq!(offered.len(), EVERY_OFFER.len());
        for official in alo_strings::union::OFFICIAL {
            let language = Language::written(official.tag).unwrap();
            assert!(
                offered_with(&language).is_some(),
                "{} has no keyboard",
                official.in_english
            );
        }
    }

    /// **Every layout written here is a layout's name**, which is what
    /// [`Layout::offered`] does not check for itself.
    #[test]
    fn every_offered_layout_is_a_layout() {
        for offer in EVERY_OFFER {
            let mut every = offer.beside_it();
            every.push(offer.first());
            for layout in every {
                let checked = match layout.variant() {
                    None => Layout::named(layout.name()),
                    Some(variant) => Layout::variant_of(layout.name(), variant),
                };
                assert_eq!(checked, Ok(layout.clone()), "{}", offer.language);
            }
        }
    }

    /// A region does not change the keyboard: Brazilian Portuguese is offered
    /// what Portuguese is offered, with `br` beside it.
    #[test]
    fn a_region_is_not_a_language() {
        let brazilian = Language::written("pt-BR").unwrap();
        let offer = offered_with(&brazilian).unwrap();
        assert_eq!(offer.first(), Layout::named("pt").unwrap());
        assert!(offer.beside_it().contains(&Layout::named("br").unwrap()));
    }

    /// A language nobody has written an offer for is `None` rather than a
    /// guess, which is what [`crate::Refused::NoKeyboardForThisLanguage`]
    /// becomes.
    #[test]
    fn a_language_with_no_offer_is_none() {
        assert_eq!(offered_with(&Language::written("is").unwrap()), None);
        assert_eq!(offered_with(&Language::written("ja").unwrap()), None);
    }
}
