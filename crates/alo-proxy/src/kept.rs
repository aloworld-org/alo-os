//! Whose proxy this is, and who may change it.
//!
//! [ADR 0016](../../../docs/decisions/0016-the-organisation-bounds-and-the-person-chooses.md):
//! the organisation **bounds**, the person **chooses**. A proxy is the shape of
//! that rule where there is nothing to choose *within* — a road out either goes
//! through the company's proxy or it does not — so what the ADR settles here is
//! narrower and sharper than usual:
//!
//! - on a machine an organisation manages, the proxy is the organisation's and
//!   a person is **told so** rather than finding a setting that quietly does
//!   nothing when they change it;
//! - on a personal machine, it is the person's and nobody else's.
//!
//! # A refusal names who set it, and that is the point of the refusal
//!
//! [`NotChanged::AnOrganisationSetIt`] is what a person meets when they try to
//! change a managed machine's proxy, and it says *your organisation set this*.
//! A settings panel that simply hid the field would leave somebody on a broken
//! network with nothing to act on; the sentence tells them who to ask, which is
//! the only useful thing anybody can tell them.
//!
//! # Who set it is decided by who owns what it was read from
//!
//! The rule `docs/contracts/machine-description.md` states for `[questions]`,
//! and `alo_software::Bound` states for where applications come from: a setting
//! written by root is an organisation's, and one the person wrote on their own
//! machine is theirs. Reading it out of the machine's description is
//! `alo-agentd`'s, where that file is read; what is here is the value it hands
//! over and the refusal it produces.

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::setting::TheProxy;
use crate::words;

/// Who set the proxy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SetBy {
    /// The organisation that manages this machine, through a description
    /// written by root.
    AnOrganisation,
    /// The person whose machine this is.
    ThisPerson,
}

impl SetBy {
    /// Both of them, in the order this file declares them.
    pub const EVERY: [Self; 2] = [Self::AnOrganisation, Self::ThisPerson];

    /// The string this crate declares for saying who set it.
    #[must_use]
    pub const fn word(self) -> words::Word {
        match self {
            Self::AnOrganisation => words::SET_BY_AN_ORGANISATION,
            Self::ThisPerson => words::SET_BY_THIS_PERSON,
        }
    }

    /// Who set it, in the language the person reads.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

/// Why the proxy was not changed.
///
/// **No `Display`**: this is somebody in a settings panel, and the only road to
/// words is [`NotChanged::said`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotChanged {
    /// The organisation that manages this machine set it.
    AnOrganisationSetIt,
}

impl NotChanged {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(self) -> words::Word {
        match self {
            Self::AnOrganisationSetIt => words::NOT_CHANGED_AN_ORGANISATION_SET_IT,
        }
    }

    /// What this says, in the language the person reads.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

/// The proxy this machine has, and who set it.
///
/// Deliberately no `Default`, as [`TheProxy`] has none and for the same reason:
/// a machine is told what its proxy is and whose it is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kept {
    /// The setting itself.
    proxy: TheProxy,
    /// Who set it.
    set_by: SetBy,
}

impl Kept {
    /// The proxy an organisation set on a machine it manages.
    #[must_use]
    pub fn by_an_organisation(proxy: TheProxy) -> Self {
        Self {
            proxy,
            set_by: SetBy::AnOrganisation,
        }
    }

    /// The proxy the person whose machine this is set.
    #[must_use]
    pub fn by_this_person(proxy: TheProxy) -> Self {
        Self {
            proxy,
            set_by: SetBy::ThisPerson,
        }
    }

    /// The setting itself, which is what every road out asks.
    #[must_use]
    pub const fn proxy(&self) -> &TheProxy {
        &self.proxy
    }

    /// Who set it.
    #[must_use]
    pub const fn set_by(&self) -> SetBy {
        self.set_by
    }

    /// The same machine, with the person having set a different proxy.
    ///
    /// # Errors
    /// [`NotChanged::AnOrganisationSetIt`] on a machine an organisation
    /// manages. A person is refused **in words naming who set it**, never by a
    /// field that silently does nothing.
    pub fn changed_by_the_person(self, proxy: TheProxy) -> Result<Self, NotChanged> {
        match self.set_by {
            SetBy::AnOrganisation => Err(NotChanged::AnOrganisationSetIt),
            SetBy::ThisPerson => Ok(Self::by_this_person(proxy)),
        }
    }

    /// The same machine, with the organisation having set a different proxy.
    ///
    /// An organisation's description is the authority over a machine it
    /// manages, and it is also what turns a personal machine into a managed one
    /// — so this never refuses. What it is not is a road a person can reach:
    /// `alo-agentd` reads the description, and a person is not that file.
    #[must_use]
    pub fn changed_by_the_organisation(self, proxy: TheProxy) -> Self {
        Self::by_an_organisation(proxy)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::address::{ProxyAddress, SpokenTo};
    use crate::testing::in_english;

    fn an_address() -> ProxyAddress {
        ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).unwrap()
    }

    /// **A person may set the proxy on their own machine**, and what they set
    /// is what every road out then asks.
    #[test]
    fn a_person_sets_the_proxy_on_their_own_machine() {
        let kept = Kept::by_this_person(TheProxy::None);
        assert_eq!(kept.set_by(), SetBy::ThisPerson);
        let changed = kept
            .changed_by_the_person(TheProxy::one(an_address()))
            .unwrap();
        assert_eq!(changed.proxy(), &TheProxy::one(an_address()));
        assert_eq!(changed.set_by(), SetBy::ThisPerson);
    }

    /// **A person is refused on a managed machine, in words that say who set
    /// it** — and the setting is unchanged.
    #[test]
    fn a_person_is_refused_on_a_managed_machine_and_told_who_set_it() {
        let kept = Kept::by_an_organisation(TheProxy::one(an_address()));
        assert_eq!(
            kept.clone()
                .changed_by_the_person(TheProxy::None)
                .unwrap_err(),
            NotChanged::AnOrganisationSetIt
        );
        assert_eq!(kept.proxy(), &TheProxy::one(an_address()));

        let strings = in_english();
        let said = NotChanged::AnOrganisationSetIt.said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("organisation"), "{said}");
    }

    /// The organisation's description is the authority on a machine it manages,
    /// and setting one is also what makes a machine managed.
    #[test]
    fn the_organisations_description_is_the_authority_and_can_take_a_machine_over() {
        let personal = Kept::by_this_person(TheProxy::None);
        let managed = personal.changed_by_the_organisation(TheProxy::one(an_address()));
        assert_eq!(managed.set_by(), SetBy::AnOrganisation);
        assert!(managed.changed_by_the_person(TheProxy::None).is_err());
    }

    /// Both answers to *whose is this* say so in the language the person
    /// reads, which is what *answerable in ten seconds* needs.
    #[test]
    fn both_answers_to_whose_this_is_are_sayable() {
        let strings = in_english();
        assert_eq!(SetBy::EVERY.len(), 2);
        for set_by in SetBy::EVERY {
            let said = set_by.said(&strings);
            assert!(!said.is_a_bug(), "{set_by:?}: {said}");
            assert!(said.unfilled().is_empty(), "{set_by:?}: {said}");
        }
    }

    /// The whole thing survives being written down and read back: *machine-wide*
    /// means one setting outliving whatever process set it, and *whose* outlives
    /// it too.
    #[test]
    fn whose_proxy_this_is_survives_being_written_down() {
        for kept in [
            Kept::by_an_organisation(TheProxy::one(an_address())),
            Kept::by_this_person(TheProxy::None),
        ] {
            let written = serde_json::to_string(&kept).unwrap();
            assert_eq!(
                serde_json::from_str::<Kept>(&written).ok(),
                Some(kept),
                "{written}"
            );
        }
    }
}
