//! `[applications]` — the places an organisation permits applications to come
//! from, as somebody typed them, and the rule they state.
//!
//! ADR 0016: the organisation **bounds**, the person **chooses**. For where a
//! question may be answered that is `[questions]`, read in `crate::describing`;
//! this is the same bound for where an application may be installed from, and
//! the value it becomes is `alo_software::Bound`, which that crate already
//! decides with. Nothing here decides what a permitted place may do — the order
//! a place is refused in, the words a person reads, the signature check — and
//! nothing here edits `alo-software`. This file is only the door from a line in
//! `/etc/alo/agentd.toml` to that value.
//!
//! # One key, a list of names
//!
//! ```toml
//! [applications]
//! may-come-from = ["acme-apps", "flathub"]
//! ```
//!
//! Each name is what this machine calls a place applications come from — the
//! name the rented tool gives it, the same `alo_software::SourceName` a person
//! installs by — and a name is matched exactly, as grants are.
//!
//! # Absent is not permissive, and present is never read as unrestricted
//!
//! No section is `alo_software::Bound::Nobodys`: nobody set a rule, and nobody is
//! named in a refusal. A section that is present and does not hold **stops the
//! service** rather than becoming no rule, because an organisation that wrote a
//! list and got none, with nothing on the machine saying so, is the one failure
//! worth building the section around. So each of these is refused:
//!
//! - no `may-come-from` in the section, or one that is not a list of strings
//!   (the shape, answered by the reader with the line it is on);
//! - a key in the section this service does not know;
//! - a name that could never be a place's name —
//!   [`NotDescribed::NotAPlaceName`];
//! - the same place named twice — [`NotDescribed::APlaceNamedTwice`]. It permits
//!   nothing a single mention would not, and a line copied and not edited is
//!   exactly how the place somebody meant to permit goes missing, so it is
//!   answered rather than tolerated.
//!
//! **An empty list is a rule**, and it keeps every place out. That is
//! `alo_software::Bound`'s decision rather than this file's — *an organisation
//! that wrote an empty list wrote a rule* — and it is not reopened here.
//!
//! # Who set it is who owns the file
//!
//! Exactly as `[questions]`: root is an administrator
//! (`alo_software::SetBy::AnAdministrator`), the person is their own machine
//! (`alo_software::SetBy::ThisMachine`), and how strict the list is never enters
//! into it.

use std::collections::BTreeSet;

use alo_software::{Bound, SetBy, SourceName};
use serde::Deserialize;

use crate::refusing::NotDescribed;
use crate::trusting::WhoDescribedIt;

/// The key the permitted places are written under.
pub const THE_PLACES: &str = "applications.may-come-from";

/// The places an organisation permits applications to come from, exactly as
/// they were typed.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct TheApplications {
    /// Every place permitted, by the name this machine knows it by.
    may_come_from: Vec<String>,
}

impl TheApplications {
    /// This section as the rule it states, attributed to whoever wrote the
    /// file — or the first reason it states none.
    ///
    /// # Errors
    ///
    /// [`NotDescribed::NotAPlaceName`] for a name the rented tool could never
    /// be handed, and [`NotDescribed::APlaceNamedTwice`] for a place written
    /// twice. Neither is ever read as no rule.
    pub(crate) fn checked(self, who: WhoDescribedIt) -> Result<Bound, NotDescribed> {
        let mut permitted = BTreeSet::new();
        for said in self.may_come_from {
            let Some(named) = SourceName::checked(&said) else {
                return Err(NotDescribed::NotAPlaceName { said });
            };
            if !permitted.insert(named) {
                return Err(NotDescribed::APlaceNamedTwice { named: said });
            }
        }
        let set_by = match who {
            WhoDescribedIt::AnAdministrator => SetBy::AnAdministrator,
            WhoDescribedIt::ThePerson => SetBy::ThisMachine,
        };
        Ok(Bound::only(permitted, set_by))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The section as it would arrive from a file.
    fn written(text: &str) -> Result<TheApplications, toml::de::Error> {
        toml::from_str(text)
    }

    fn named(name: &str) -> SourceName {
        SourceName::checked(name).unwrap()
    }

    /// **The places named are the places permitted**, and an administrator's
    /// file makes it an organisation's rule.
    #[test]
    fn the_places_named_are_the_places_permitted() {
        let bound = written(r#"may-come-from = ["acme-apps", "flathub"]"#)
            .unwrap()
            .checked(WhoDescribedIt::AnAdministrator)
            .unwrap();
        assert_eq!(
            bound,
            Bound::only(
                [named("acme-apps"), named("flathub")],
                SetBy::AnAdministrator
            )
        );
        assert_eq!(bound.keeps_out(&named("flathub")), None);
        assert_eq!(
            bound.keeps_out(&named("elsewhere")),
            Some(SetBy::AnAdministrator)
        );
    }

    /// **The same list in the person's own file names no organisation.**
    #[test]
    fn the_same_list_the_person_wrote_is_their_own() {
        let bound = written(r#"may-come-from = ["flathub"]"#)
            .unwrap()
            .checked(WhoDescribedIt::ThePerson)
            .unwrap();
        assert_eq!(
            bound.keeps_out(&named("acme-apps")),
            Some(SetBy::ThisMachine)
        );
    }

    /// **An empty list is a rule that keeps every place out**, never no rule.
    #[test]
    fn an_empty_list_keeps_every_place_out() {
        let bound = written("may-come-from = []")
            .unwrap()
            .checked(WhoDescribedIt::AnAdministrator)
            .unwrap();
        assert_ne!(bound, Bound::Nobodys);
        assert_eq!(
            bound.keeps_out(&named("flathub")),
            Some(SetBy::AnAdministrator)
        );
    }

    /// **A name that could never be a place is refused**, including the one
    /// that would reach the rented tool as an instruction.
    #[test]
    fn a_name_that_could_never_be_a_place_is_refused() {
        for said in ["", "   ", "--no-deps", "acme apps", "acme/apps", "a\nb"] {
            let refused = TheApplications {
                may_come_from: vec!["flathub".to_owned(), said.to_owned()],
            }
            .checked(WhoDescribedIt::AnAdministrator)
            .unwrap_err();
            assert!(
                matches!(refused, NotDescribed::NotAPlaceName { said: ref s } if s == said),
                "{said:?}: {refused:?}"
            );
            assert!(refused.to_string().contains(THE_PLACES), "{refused}");
        }
    }

    /// **A place named twice is refused**, and the refusal names it.
    #[test]
    fn a_place_named_twice_is_refused() {
        let refused = written(r#"may-come-from = ["flathub", "acme-apps", "flathub"]"#)
            .unwrap()
            .checked(WhoDescribedIt::AnAdministrator)
            .unwrap_err();
        assert!(
            matches!(refused, NotDescribed::APlaceNamedTwice { ref named } if named == "flathub"),
            "{refused:?}"
        );
        assert!(refused.to_string().contains("flathub"), "{refused}");
    }

    /// **Names are matched exactly**: two spellings are two places.
    #[test]
    fn two_spellings_are_two_places() {
        let bound = written(r#"may-come-from = ["flathub", "Flathub"]"#)
            .unwrap()
            .checked(WhoDescribedIt::AnAdministrator)
            .unwrap();
        assert!(bound.keeps_out(&named("FLATHUB")).is_some());
    }

    /// **The shape is refused by the reader**: no key, a key nobody declared,
    /// and a single name where a list belongs.
    #[test]
    fn a_section_not_in_this_shape_is_refused_by_the_reader() {
        assert!(
            written("")
                .unwrap_err()
                .to_string()
                .contains("may-come-from")
        );
        assert!(
            written("may-come-from = [\"flathub\"]\nmay-not-come-from = [\"x\"]")
                .unwrap_err()
                .to_string()
                .contains("may-not-come-from")
        );
        assert!(written(r#"may-come-from = "flathub""#).is_err());
        assert!(written("may-come-from = [1]").is_err());
    }

    /// The key is named once, because it is in a refusal somebody reads and in
    /// the contract they wrote the file against.
    #[test]
    fn the_key_is_named_once() {
        assert_eq!(THE_PLACES, "applications.may-come-from");
    }
}
