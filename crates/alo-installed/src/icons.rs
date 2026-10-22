//! What each installed application calls its icon.
//!
//! **A name, never a picture.** A desktop entry's `Icon` key holds either an
//! absolute path or a *theme name* — `org.gnome.Nautilus`, `firefox` — and
//! turning the second into a file is an icon-theme lookup, which this crate's
//! own header declines and `alo-icons` does. What is here is the name the
//! entry gave, read off the same walk as everything else.
//!
//! # Why this is its own file
//!
//! `lib.rs` is the walk: where entries live, which ones count, and how an
//! identifier is spelled. *What an entry said about its icon* is a second
//! reason for that file to change, and this repository splits on the change
//! that discovers it rather than noting it (`CLAUDE.md`, law 4).
//!
//! # Keyed by identifier, because the identifier is what is matched
//!
//! `alo_applications::Installed` matches an identifier **exactly, with no case
//! folding**, and this map is read beside it. Keying it on the shown name
//! instead would put two applications that chose one name into one entry, and
//! would stop agreeing with `Installed` the moment a person renamed one.

use std::collections::BTreeMap;

/// The icon name each installed application declared, by identifier.
///
/// **An application missing from this is an application with no `Icon` line**,
/// which is ordinary and is not an error: the specification does not require
/// one, and the Dock's answer for it — the first letter — is already decided
/// (`docs/design/the-alo-dock.md`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Icons(BTreeMap<String, String>);

impl Icons {
    /// A machine where nothing declared an icon.
    #[must_use]
    pub fn nothing() -> Self {
        Self(BTreeMap::new())
    }

    /// Remember that this identifier calls its icon this.
    ///
    /// **The first wins**, which is [`crate::read`]'s rule and for its reason:
    /// earlier directories are more specific, so a person's own copy of an
    /// entry names the icon their copy names.
    pub fn declares(&mut self, identifier: &str, named: &str) {
        self.0
            .entry(identifier.to_owned())
            .or_insert_with(|| named.to_owned());
    }

    /// What this identifier calls its icon, or [`None`] where it named none.
    #[must_use]
    pub fn named(&self, identifier: &str) -> Option<&str> {
        self.0.get(identifier).map(String::as_str)
    }

    /// How many applications declared one.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.0.len()
    }

    /// Whether nothing declared one.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Every identifier and the icon it named, in identifier order.
    pub fn each(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0
            .iter()
            .map(|(identifier, named)| (identifier.as_str(), named.as_str()))
    }
}

#[cfg(test)]
mod a_machine_and_what_it_calls_its_icons {
    use super::Icons;

    #[test]
    fn a_machine_where_nothing_declared_one_knows_nothing() {
        let icons = Icons::nothing();
        assert!(icons.is_empty());
        assert_eq!(icons.how_many(), 0);
        assert_eq!(icons.named("org.gnome.Nautilus"), None);
    }

    #[test]
    fn what_an_application_called_its_icon_is_read_back_by_identifier() {
        let mut icons = Icons::nothing();
        icons.declares("org.gnome.Nautilus", "org.gnome.Nautilus");
        assert_eq!(
            icons.named("org.gnome.Nautilus"),
            Some("org.gnome.Nautilus")
        );
        // The control: a near miss is not a match, which is the exact-match
        // rule `alo_applications::Installed` holds and this is read beside.
        assert_eq!(icons.named("org.gnome.nautilus"), None);
        assert_eq!(icons.named("org.gnome.Nautilu"), None);
    }

    #[test]
    fn the_first_directory_to_name_one_keeps_it() {
        let mut icons = Icons::nothing();
        icons.declares("browser", "the-persons-own");
        icons.declares("browser", "what-the-image-shipped");
        assert_eq!(
            icons.named("browser"),
            Some("the-persons-own"),
            "earlier directories win, as they do for the name and the media types"
        );
    }

    #[test]
    fn every_pair_is_offered_in_identifier_order() {
        let mut icons = Icons::nothing();
        icons.declares("b", "second");
        icons.declares("a", "first");
        let each: Vec<(&str, &str)> = icons.each().collect();
        assert_eq!(each, vec![("a", "first"), ("b", "second")]);
    }
}
