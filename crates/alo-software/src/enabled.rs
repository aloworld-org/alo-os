//! The places this machine installs applications from, under the rule that
//! bounds them.
//!
//! What a person or an organisation set up in the rented tool, read once, with
//! [`Bound`] beside it — and the one question every installation and every
//! update asks before anything leaves: **may this place be used, and where does
//! using it reach?** [`Enabled::usable`] answers it, and every way the answer
//! is no is a [`NotDone`] a person can read.
//!
//! # The order is the refusal a person needs
//!
//! 1. **Is it set up at all?** A name nothing is set up under is
//!    [`NotDone::NotEnabled`], whatever any rule says about it.
//! 2. **Does a rule keep it out?** [`NotDone::OutsideTheBound`], naming who set
//!    the rule — before either fact about the place itself, because a person
//!    told *it does not check signatures* about a place their organisation
//!    forbids would go and fix the wrong thing.
//! 3. **Does it check what it sends?** [`NotDone::NotVerified`] if not.
//! 4. **Can it be reached?** [`NotDone::NowhereToReach`] if its address is not
//!    one this machine reads, because an installation that cannot name where
//!    it leaves for cannot go on the indicator.

use std::collections::BTreeMap;

use alo_egress::Destination;

use crate::bound::Bound;
use crate::refusing::NotDone;
use crate::source::{Configured, Source, SourceName};
use crate::tool::{Failed, Tool};

/// The places this machine installs from, and the rule over them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enabled {
    /// Every place set up and switched on, by name.
    sources: BTreeMap<SourceName, Source>,
    /// The rule about which of them may be used.
    bound: Bound,
}

impl Enabled {
    /// The places the rented tool is set up with, under this rule.
    ///
    /// Places switched off, and places whose names could never be handed to
    /// the tool, are not enabled. Where two share a name the first is kept, as
    /// `alo_applications::Installed` keeps the first application — the tool
    /// does not allow two, so a second is a fault rather than a choice.
    #[must_use]
    pub fn of(configured: &[Configured], bound: Bound) -> Self {
        let mut sources = BTreeMap::new();
        for source in configured.iter().filter_map(Source::of) {
            sources.entry(source.name().clone()).or_insert(source);
        }
        Self { sources, bound }
    }

    /// The places the rented tool says it is set up with right now, under
    /// this rule.
    ///
    /// # Errors
    /// [`NotDone::DidNotAnswer`] when the tool could not say.
    pub fn read(tool: &impl Tool, bound: Bound) -> Result<Self, NotDone> {
        match tool.sources() {
            Ok(configured) => Ok(Self::of(&configured, bound)),
            Err(failed) => Err(did_not_answer(failed)),
        }
    }

    /// Every place set up and switched on, by name — for a settings surface
    /// listing where applications can come from. Whether each may be used is
    /// [`Enabled::usable`].
    pub fn all(&self) -> impl Iterator<Item = &Source> {
        self.sources.values()
    }

    /// The rule these places are under.
    #[must_use]
    pub const fn bound(&self) -> &Bound {
        &self.bound
    }

    /// The place with this name, and where using it reaches — or why it may
    /// not be used, in the order at the top of this file.
    ///
    /// # Errors
    /// [`NotDone`]: not enabled, outside the bound, not verified, or nowhere to
    /// reach.
    pub fn usable(&self, named: &str) -> Result<(&Source, &Destination), NotDone> {
        let Some(source) = SourceName::checked(named).and_then(|name| self.sources.get(&name))
        else {
            return Err(NotDone::not_enabled(named));
        };
        let called = || source.name().as_str().to_owned();
        if let Some(set_by) = self.bound.keeps_out(source.name()) {
            return Err(NotDone::OutsideTheBound {
                source: called(),
                set_by,
            });
        }
        if !source.checks_signatures() {
            return Err(NotDone::NotVerified { source: called() });
        }
        let Some(destination) = source.destination() else {
            return Err(NotDone::NowhereToReach { source: called() });
        };
        Ok((source, destination))
    }
}

/// A tool that could not answer a question, as the refusal a person reads.
///
/// Every [`Failed`] that is not about a particular application or place means
/// the same thing when only a list was asked for: the tool did not say.
pub(crate) fn did_not_answer(failed: Failed) -> NotDone {
    NotDone::DidNotAnswer {
        said: match failed {
            Failed::DidNotAnswer { said } => said,
            other => format!("{other:?}"),
        },
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::bound::SetBy;

    fn configured(name: &str, address: &str, checks: bool, off: bool) -> Configured {
        Configured {
            name: name.to_owned(),
            address: address.to_owned(),
            checks_signatures: checks,
            switched_off: off,
        }
    }

    fn a_machine(bound: Bound) -> Enabled {
        Enabled::of(
            &[
                configured("flathub", "https://dl.flathub.org/repo/", true, false),
                configured("acme-apps", "https://apps.acme.example/repo", true, false),
                configured("unchecked", "https://loose.example/repo", false, false),
                configured("old", "https://old.example/repo", true, true),
                configured("local", "file:///srv/apps", true, false),
            ],
            bound,
        )
    }

    fn name(text: &str) -> SourceName {
        SourceName::checked(text).unwrap()
    }

    #[test]
    fn a_place_set_up_checked_and_permitted_is_usable_and_says_where_it_reaches() {
        let enabled = a_machine(Bound::Nobodys);
        let (source, destination) = enabled.usable("flathub").unwrap();
        assert_eq!(source.name().as_str(), "flathub");
        assert_eq!(destination, &Destination::at("dl.flathub.org").unwrap());
    }

    /// **Every way a place may not be used is refused, in the order a person
    /// needs to hear it.**
    #[test]
    fn every_way_a_place_may_not_be_used_is_refused_in_words() {
        let enabled = a_machine(Bound::Nobodys);
        assert_eq!(
            enabled.usable("elsewhere").unwrap_err(),
            NotDone::not_enabled("elsewhere")
        );
        assert_eq!(
            enabled.usable("old").unwrap_err(),
            NotDone::not_enabled("old"),
            "a place switched off was usable"
        );
        assert_eq!(
            enabled.usable("--no-deps").unwrap_err(),
            NotDone::not_enabled("--no-deps")
        );
        assert_eq!(
            enabled.usable("unchecked").unwrap_err(),
            NotDone::NotVerified {
                source: "unchecked".to_owned()
            }
        );
        assert_eq!(
            enabled.usable("local").unwrap_err(),
            NotDone::NowhereToReach {
                source: "local".to_owned()
            }
        );
    }

    /// **A rule keeps a place out before anything about the place is said**,
    /// and names who set it; a place the rule permits is usable as before.
    #[test]
    fn a_rule_keeps_a_place_out_before_anything_else_is_said_about_it() {
        let enabled = a_machine(Bound::only([name("acme-apps")], SetBy::AnAdministrator));
        assert!(enabled.usable("acme-apps").is_ok());
        assert_eq!(
            enabled.usable("flathub").unwrap_err(),
            NotDone::OutsideTheBound {
                source: "flathub".to_owned(),
                set_by: SetBy::AnAdministrator
            }
        );
        // Unchecked *and* forbidden: the rule is what the person hears.
        assert_eq!(
            enabled.usable("unchecked").unwrap_err(),
            NotDone::OutsideTheBound {
                source: "unchecked".to_owned(),
                set_by: SetBy::AnAdministrator
            }
        );
        // Not set up at all: that, whatever the rule says.
        assert_eq!(
            enabled.usable("elsewhere").unwrap_err(),
            NotDone::not_enabled("elsewhere")
        );
    }

    /// **A second place under a name already taken does not take it.**
    #[test]
    fn the_first_place_under_a_name_keeps_it() {
        let enabled = Enabled::of(
            &[
                configured("flathub", "https://dl.flathub.org/repo/", true, false),
                configured("flathub", "https://impostor.example/repo", true, false),
            ],
            Bound::Nobodys,
        );
        assert_eq!(enabled.all().count(), 1);
        assert_eq!(
            enabled.usable("flathub").unwrap().1,
            &Destination::at("dl.flathub.org").unwrap()
        );
    }
}
