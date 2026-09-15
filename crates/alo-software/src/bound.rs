//! Which places an application may come from on this machine, and who said so.
//!
//! [ADR 0016](../../../docs/decisions/0016-the-organisation-bounds-and-the-person-chooses.md):
//! the organisation **bounds**, the person **chooses**. Here the choice is the
//! places a person set up to install from, and the bound is the organisation's
//! rule about which places are permitted at all. A place outside the bound is
//! **refused out loud**, naming who set the rule — never quietly dropped from a
//! list, and never swapped for a permitted place that happens to offer the same
//! application.
//!
//! # Absent is not permissive
//!
//! On a machine nobody manages there is no rule, and [`Bound::Nobodys`] is that
//! absence as a value — not an empty rule and not a rule permitting
//! everything. The two permit the same places; what they do not share is
//! somebody to name in a refusal, which is ADR 0016's own argument about
//! `[questions]`.
//!
//! # Who set it is decided by who owns what it was read from
//!
//! The same rule `docs/contracts/machine-description.md` states for
//! `[questions]`: a rule written by root is an organisation's
//! ([`SetBy::AnAdministrator`]), and one the person wrote on their own machine
//! is theirs ([`SetBy::ThisMachine`]) — and a restrictive rule is never, on its
//! own, evidence that somebody else set it. Reading the rule from the machine's
//! description is `alo-agentd`'s, where that file is read; what is here is the
//! value it hands over and the refusal it produces.

use std::collections::BTreeSet;

use crate::source::SourceName;

/// Who set a rule about where applications may come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetBy {
    /// The organisation that manages this machine, through a description
    /// written by root.
    AnAdministrator,
    /// The person whose machine this is, through a description they own.
    ThisMachine,
}

/// The rule about which places applications may come from.
///
/// Deliberately no `Default`: a machine is told whether anybody set a rule,
/// and *nobody did* is something written rather than something forgotten.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bound {
    /// Nobody set a rule. Every place a person set up may be installed from.
    Nobodys,
    /// Only these places, and who said so.
    Only {
        /// The places permitted, by name.
        permitted: BTreeSet<SourceName>,
        /// Who set the rule.
        set_by: SetBy,
    },
}

impl Bound {
    /// Only these places, as set by this owner.
    pub fn only(permitted: impl IntoIterator<Item = SourceName>, set_by: SetBy) -> Self {
        Self::Only {
            permitted: permitted.into_iter().collect(),
            set_by,
        }
    }

    /// Who set the rule that keeps this place out, or [`None`] when the place
    /// is permitted.
    #[must_use]
    pub fn keeps_out(&self, source: &SourceName) -> Option<SetBy> {
        match self {
            Self::Nobodys => None,
            Self::Only { permitted, set_by } => (!permitted.contains(source)).then_some(*set_by),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn named(name: &str) -> SourceName {
        SourceName::checked(name).unwrap()
    }

    /// **No rule keeps nothing out**, and says there is nobody to name.
    #[test]
    fn nobodys_rule_keeps_no_place_out() {
        assert_eq!(Bound::Nobodys.keeps_out(&named("flathub")), None);
    }

    /// **A rule keeps out what it does not name, and says who set it** — the
    /// organisation's and the person's own alike, told apart.
    #[test]
    fn a_rule_keeps_out_what_it_does_not_name_and_says_who_set_it() {
        let organisations = Bound::only([named("acme-apps")], SetBy::AnAdministrator);
        assert_eq!(organisations.keeps_out(&named("acme-apps")), None);
        assert_eq!(
            organisations.keeps_out(&named("flathub")),
            Some(SetBy::AnAdministrator)
        );

        let owners = Bound::only([named("flathub")], SetBy::ThisMachine);
        assert_eq!(
            owners.keeps_out(&named("acme-apps")),
            Some(SetBy::ThisMachine)
        );
    }

    /// **A rule naming nothing keeps everything out.** It is not read as no
    /// rule, because an organisation that wrote an empty list wrote a rule.
    #[test]
    fn a_rule_naming_nothing_keeps_every_place_out() {
        let empty = Bound::only([], SetBy::AnAdministrator);
        assert_eq!(
            empty.keeps_out(&named("flathub")),
            Some(SetBy::AnAdministrator)
        );
    }

    /// Names are matched exactly, as grants are.
    #[test]
    fn a_permitted_name_is_matched_exactly() {
        let rule = Bound::only([named("flathub")], SetBy::AnAdministrator);
        assert!(rule.keeps_out(&named("Flathub")).is_some());
        assert!(rule.keeps_out(&named("flathub-beta")).is_some());
    }
}
