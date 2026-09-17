//! **Every language alo OS speaks has a keyboard, and it is one this machine
//! actually has.**
//!
//! One test per language, each naming the keyboard out loud, because that is
//! the only shape in which a missing one is a failure somebody can read: a
//! single loop over a table would fail saying *a language has no keyboard* and
//! leave whoever reads it to work out which. A language added to `alo-strings`
//! and not here fails `every_one_of_the_twenty_four_languages_has_a_keyboard`;
//! a keyboard `xkeyboard-config` renames or drops fails that language's own
//! test, by name.
//!
//! Every one of these asks **the rented data on this machine**, not a table in
//! the test: `/usr/share/X11/xkb/rules/evdev.lst`, which is the list this
//! machine's own keyboard configuration is built from. A test that checked our
//! table against our table would prove nothing at all.
//!
//! It is not the hardware verification `CLAUDE.md` asks for. Nobody has typed
//! on a certified machine here; what is held is that the keyboard offered with
//! each language is one the machine has, which is the half a test can hold.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_keyboards::{Keyboards, Layout, Rented, offering};
use alo_strings::Language;

/// The machine's own list of keyboards, read.
fn this_machine() -> Rented {
    let read = Rented::read();
    assert!(
        read.is_ok(),
        "this machine's list of keyboards ({}) could not be read: {read:?} — alo OS ships \
         xkeyboard-config, and a machine running these tests needs it installed",
        alo_keyboards::THE_RULES
    );
    read.unwrap()
}

/// A language, and the keyboard it is typed on.
macro_rules! typed_on {
    ($named:ident: $tag:literal => $layout:literal) => {
        typed_on!($named: $tag => $layout, None);
    };
    ($named:ident: $tag:literal => $layout:literal, $variant:expr) => {
        #[test]
        fn $named() {
            let language = Language::written($tag).unwrap();
            let keyboard = match $variant {
                None => Layout::named($layout),
                Some(variant) => Layout::variant_of($layout, variant),
            }
            .unwrap();

            let offer = offering::offered_with(&language);
            assert!(offer.is_some(), "{} has no keyboard at all", $tag);
            assert_eq!(
                offer.unwrap().first(),
                keyboard,
                "{} is offered the wrong keyboard",
                $tag
            );

            assert!(
                this_machine().has(&keyboard),
                "{} is offered {keyboard}, which this machine does not have",
                $tag
            );

            assert_eq!(
                Keyboards::offered_with(&language).in_use(),
                &keyboard,
                "{} does not start on its own keyboard",
                $tag
            );
        }
    };
}

typed_on!(bulgarian_is_typed_on_the_bulgarian_keyboard: "bg" => "bg");
typed_on!(croatian_is_typed_on_the_croatian_keyboard: "hr" => "hr");
typed_on!(czech_is_typed_on_the_czech_keyboard: "cs" => "cz");
typed_on!(danish_is_typed_on_the_danish_keyboard: "da" => "dk");
typed_on!(dutch_is_typed_on_the_us_international_keyboard: "nl" => "us", Some("intl"));
typed_on!(english_is_typed_on_the_us_keyboard: "en" => "us");
typed_on!(estonian_is_typed_on_the_estonian_keyboard: "et" => "ee");
typed_on!(finnish_is_typed_on_the_finnish_keyboard: "fi" => "fi");
typed_on!(french_is_typed_on_the_french_keyboard: "fr" => "fr");
typed_on!(german_is_typed_on_the_german_keyboard: "de" => "de");
typed_on!(greek_is_typed_on_the_greek_keyboard: "el" => "gr");
typed_on!(hungarian_is_typed_on_the_hungarian_keyboard: "hu" => "hu");
typed_on!(irish_is_typed_on_the_irish_keyboard: "ga" => "ie");
typed_on!(italian_is_typed_on_the_italian_keyboard: "it" => "it");
typed_on!(latvian_is_typed_on_the_latvian_keyboard: "lv" => "lv");
typed_on!(lithuanian_is_typed_on_the_lithuanian_keyboard: "lt" => "lt");
typed_on!(maltese_is_typed_on_the_maltese_keyboard: "mt" => "mt");
typed_on!(polish_is_typed_on_the_polish_keyboard: "pl" => "pl");
typed_on!(portuguese_is_typed_on_the_portuguese_keyboard: "pt" => "pt");
typed_on!(romanian_is_typed_on_the_romanian_keyboard: "ro" => "ro");
typed_on!(slovak_is_typed_on_the_slovak_keyboard: "sk" => "sk");
typed_on!(slovenian_is_typed_on_the_slovenian_keyboard: "sl" => "si");
typed_on!(spanish_is_typed_on_the_spanish_keyboard: "es" => "es");
typed_on!(swedish_is_typed_on_the_swedish_keyboard: "sv" => "se");

/// **No language is left without one.** The twenty-four tests above name a
/// keyboard each; this is what fails when a twenty-fifth language arrives and
/// nobody wrote its line.
#[test]
fn every_one_of_the_twenty_four_languages_has_a_keyboard() {
    let rented = this_machine();
    let mut without = Vec::new();
    for official in alo_strings::union::OFFICIAL {
        let language = Language::written(official.tag).unwrap();
        match offering::offered_with(&language) {
            None => without.push(official.in_english),
            Some(offer) => assert!(
                rented.has(&offer.first()),
                "{} is offered {}, which this machine does not have",
                official.in_english,
                offer.first()
            ),
        }
    }
    assert!(without.is_empty(), "no keyboard for: {without:?}");
    assert_eq!(alo_strings::union::OFFICIAL.len(), 24);
}

/// **And every keyboard offered beside the first one exists too**, so a Belgian
/// who speaks French or a Swiss who speaks German finds theirs in the short list
/// rather than among two hundred entries.
#[test]
fn every_keyboard_offered_beside_the_first_is_one_this_machine_has() {
    let rented = this_machine();
    for offer in alo_keyboards::EVERY_OFFER {
        for beside in offer.beside_it() {
            assert!(
                rented.has(&beside),
                "{} is offered {beside}, which this machine does not have",
                offer.language
            );
            assert_ne!(beside, offer.first(), "{} offers one twice", offer.language);
        }
    }
}

/// **A keyboard nobody rents cannot be added**, whatever asks for it: the
/// refusal is the rented list's answer, not a list of our own.
#[test]
fn a_keyboard_this_machine_does_not_have_cannot_be_added() {
    let rented = this_machine();
    let mut keyboards = Keyboards::shipped();
    let made_up = Layout::named("euro-qwertz-2026").unwrap();
    assert!(!rented.has(&made_up));
    let refused = keyboards.add(made_up.clone(), &rented).unwrap_err();
    assert_eq!(refused.keyboard(), Some(&made_up));
    assert_eq!(keyboards.all().how_many(), 1);
}
