//! **What a keyboard is called, in the person's language** — and which
//! keyboards a first-start picker offers.
//!
//! The names are `xkeyboard-config`'s: the English in its own list
//! ([`crate::Rented::called_in_english`], `German`, `English (US)`) and its
//! translators' catalogue for that English (`xkeyboard-config.mo`, read through
//! `alo_gettext`), so a German reader sees `Deutsch` and `Englisch (USA)`
//! exactly as every other Linux program shows them. Nothing here is a name
//! of ours.
//!
//! **Measured once on 2026-10-28** in `localhost/alo-os:dev`: 19 of the 23
//! official languages other than English have that catalogue. Estonian, Irish,
//! Latvian and Maltese do not, so a person reading one of them sees the
//! English names — what `xkeyboard-config` shows them on every Linux desktop.
//!
//! # Which keyboards a picker offers
//!
//! [`the_keyboards_offered`]: the keyboards the chosen language is typed on
//! first — [`crate::Offer::first`] and [`crate::Offer::beside_it`] — and then
//! every other keyboard alo OS offers any of the 24 languages, once each. So
//! somebody who reads English and types on a German keyboard finds `de` in the
//! list, which is the case the owner named on 2026-10-28.

use std::collections::BTreeMap;
use std::path::Path;

use alo_gettext::Catalogue;
use alo_strings::Language;

use crate::layout::Layout;
use crate::offering::{EVERY_OFFER, offered_with};
use crate::rented::Rented;

/// The package whose catalogue names keyboards.
pub const THE_CATALOGUE: &str = "xkeyboard-config";

/// **The keyboards a picker offers someone who reads `language`**, theirs
/// first, then every other keyboard any of the 24 is offered, each once.
#[must_use]
pub fn the_keyboards_offered(language: &Language) -> Vec<Layout> {
    let mut offered: Vec<Layout> = Vec::new();
    let mut push = |layout: Layout| {
        if !offered.contains(&layout) {
            offered.push(layout);
        }
    };
    if let Some(offer) = offered_with(language) {
        push(offer.first());
        offer.beside_it().into_iter().for_each(&mut push);
    }
    for offer in EVERY_OFFER {
        push(offer.first());
        offer.beside_it().into_iter().for_each(&mut push);
    }
    offered
}

/// **What every keyboard is called**, in every language the machine has a
/// catalogue for — read once, so a screen that names keyboards opens no
/// file while a person is choosing one.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TheKeyboardNames {
    /// The rented list, where it could be read.
    rented: Option<Rented>,
    /// Each language's catalogue, by tag.
    catalogues: BTreeMap<String, Catalogue>,
}

impl TheKeyboardNames {
    /// The names from the rented list, and the catalogues under `locales` for
    /// each of `languages`.
    #[must_use]
    pub fn read(rented: Option<Rented>, locales: &Path, languages: &[Language]) -> Self {
        let catalogues = languages
            .iter()
            .filter_map(|language| {
                Catalogue::for_language(locales, THE_CATALOGUE, language)
                    .map(|it| (language.tag().to_owned(), it))
            })
            .collect();
        Self { rented, catalogues }
    }

    /// **The names this machine has**: its rented list, and the catalogue for
    /// each of the 24 official languages it has one for. A list that would not
    /// be read leaves every keyboard called by its short name, which is a
    /// machine that still works.
    #[must_use]
    pub fn on_this_machine() -> Self {
        let languages: Vec<Language> = alo_strings::union::OFFICIAL
            .iter()
            .filter_map(|it| Language::written(it.tag).ok())
            .collect();
        Self::read(
            Rented::read().ok(),
            Path::new(alo_gettext::THE_LOCALES),
            &languages,
        )
    }

    /// **What `layout` is called for somebody reading `language`**: the
    /// package's translation, its English where there is none, and the
    /// layout's own short name where the list does not know it.
    #[must_use]
    pub fn of(&self, layout: &Layout, language: &str) -> String {
        let Some(english) = self
            .rented
            .as_ref()
            .and_then(|rented| rented.called_in_english(layout))
        else {
            return layout.to_string();
        };
        self.catalogues
            .get(language)
            .and_then(|it| it.translation_of(english))
            .unwrap_or(english)
            .to_owned()
    }

    /// The two-letter region a keyboard is named after, where it is — `de`
    /// is `DE`, `us(intl)` is `US` — for the second line of a row.
    #[must_use]
    pub fn its_country(layout: &Layout) -> Option<String> {
        let name = layout.name();
        (name.len() == 2 && name.chars().all(|it| it.is_ascii_lowercase()))
            .then(|| name.to_ascii_uppercase())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    const A_LIST: &str = "! layout\n  de              German\n  us              English (US)\n";

    fn names_in_german() -> TheKeyboardNames {
        let rented = Rented::read_text(A_LIST, Path::new("evdev.lst")).unwrap();
        let mut names = TheKeyboardNames {
            rented: Some(rented),
            catalogues: BTreeMap::new(),
        };
        let german = Catalogue::read(&alo_gettext::written(&[
            ("German", "Deutsch"),
            ("English (US)", "Englisch (USA)"),
        ]))
        .unwrap();
        names.catalogues.insert("de".to_owned(), german);
        names
    }

    /// **A keyboard is called what the package calls it in the language
    /// read**, and its English where there is no catalogue.
    #[test]
    fn a_keyboard_is_called_what_the_package_calls_it() {
        let names = names_in_german();
        let german = Layout::named("de").unwrap();
        assert_eq!(names.of(&german, "de"), "Deutsch");
        assert_eq!(names.of(&german, "mt"), "German");
        assert_eq!(
            names.of(&Layout::named("us").unwrap(), "de"),
            "Englisch (USA)"
        );
    }

    /// **A keyboard the list does not know is its short name**, never empty.
    #[test]
    fn a_keyboard_the_list_does_not_know_is_its_short_name() {
        let names = TheKeyboardNames::default();
        assert_eq!(names.of(&Layout::named("de").unwrap(), "de"), "de");
    }

    /// **English first, and a German keyboard is on the list** — the owner's
    /// own example — with no keyboard offered twice.
    #[test]
    fn an_english_reader_is_offered_a_german_keyboard() {
        let offered = the_keyboards_offered(&Language::written("en").unwrap());
        assert_eq!(offered.first(), Some(&Layout::named("us").unwrap()));
        assert!(offered.contains(&Layout::named("de").unwrap()));
        for layout in &offered {
            assert_eq!(
                offered.iter().filter(|it| *it == layout).count(),
                1,
                "{layout} offered twice"
            );
        }
    }

    /// **A keyboard's country is its name, where its name is one.**
    #[test]
    fn a_keyboards_country_is_its_name_where_that_is_one() {
        assert_eq!(
            TheKeyboardNames::its_country(&Layout::named("de").unwrap()).as_deref(),
            Some("DE")
        );
        assert_eq!(
            TheKeyboardNames::its_country(&Layout::variant_of("us", "intl").unwrap()).as_deref(),
            Some("US")
        );
    }
}
