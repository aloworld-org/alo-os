//! What a person reads when a change to the network was made, and when it was
//! not.
//!
//! One sentence each, and each refusal carries only what its sentence fills in:
//! the name of the network, as it announced it. The broker answers in one word
//! from a closed list and has no person in front of it; this is where that word
//! becomes words, in the reader's language.

use alo_broker::{Answer, Switch};
use alo_record::AtTheBroker;
use alo_strings::{Filling, Said, Strings};

use crate::wanted::Change;
use crate::words;

/// Why a change to the network was not made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotChanged {
    /// What was handed over was not an approved change to the network.
    NotANetworkChange,
    /// The network manager could not be asked.
    NetworkNotAnswering,
    /// No network in range has this name.
    NoneVisibleCalled(String),
    /// No saved network has this name.
    NoneSavedCalled(String),
    /// More than one network has this name.
    MoreThanOneCalled(String),
    /// The network asks for an organisation's sign-in.
    AsksForASignIn(String),
    /// The proposal says something untrue about this conversation's connection.
    SaysTheWrongThingAboutThisConversation,
    /// What the change does to this conversation is no longer what was approved.
    ConnectionChangedSinceApproval,
    /// The approval was refused where the change is made.
    ApprovalNotAccepted,
    /// The change was accepted, and the machine could not finish it.
    CouldNotFinish,
    /// The proxy a person chose was not set.
    ProxyNotSet,
    /// The organisation that manages this machine set its proxy, so a person's
    /// is not set over it — and neither is a password for it.
    ///
    /// **`alo-proxy`'s own sentence**, said here rather than written again
    /// (ADR 0060 §5): the useful half of it is the second one, which says who
    /// can change it. Nothing was handed over, so a password typed on a managed
    /// machine never left the process it was typed into.
    AnOrganisationSetTheProxy,
    /// The machine has stopped writing down changes, so it makes none.
    NotBeingKept,
    /// Nothing was there to make the change.
    NothingMakesChanges,
}

impl NotChanged {
    /// What a person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let (word, called) = match self {
            Self::NotANetworkChange => (words::NOT_A_NETWORK_CHANGE, None),
            Self::NetworkNotAnswering => (words::NETWORK_NOT_ANSWERING, None),
            Self::NoneVisibleCalled(called) => (words::NONE_VISIBLE_CALLED, Some(called)),
            Self::NoneSavedCalled(called) => (words::NONE_SAVED_CALLED, Some(called)),
            Self::MoreThanOneCalled(called) => (words::MORE_THAN_ONE_CALLED, Some(called)),
            Self::AsksForASignIn(called) => (words::ASKS_FOR_A_SIGN_IN, Some(called)),
            Self::SaysTheWrongThingAboutThisConversation => (words::SAYS_THE_WRONG_THING, None),
            Self::ConnectionChangedSinceApproval => (words::CONNECTION_CHANGED, None),
            Self::ApprovalNotAccepted => (words::APPROVAL_NOT_ACCEPTED, None),
            Self::CouldNotFinish => (words::COULD_NOT_FINISH, None),
            Self::ProxyNotSet => (words::PROXY_NOT_SET, None),
            Self::AnOrganisationSetTheProxy => {
                return alo_proxy::NotChanged::AnOrganisationSetIt.said(strings);
            }
            Self::NotBeingKept => (words::NOT_BEING_KEPT, None),
            Self::NothingMakesChanges => (words::NOTHING_MAKES_CHANGES, None),
        };
        strings.say(&word.key(), &filled(called.map(String::as_str)))
    }

    /// What the broker's answer means.
    ///
    /// `not_carried` is what a refusal to carry the change out is said as:
    /// [`NotChanged::CouldNotFinish`] for a network, [`NotChanged::ProxyNotSet`]
    /// for the proxy.
    ///
    /// # Errors
    /// The refusal, for every answer but `carried`.
    pub fn from_the_brokers(answer: Answer, not_carried: Self) -> Result<(), Self> {
        match answer {
            Answer::Carried => Ok(()),
            Answer::NotKept => Err(Self::NotBeingKept),
            Answer::Refused(AtTheBroker::NotCarried) => Err(not_carried),
            // Everything else the door refuses is about the approval, or about
            // who asked — which, for the one process that asks, is the same
            // fact: this approval did not make this change. Named one by one so
            // a reason added to the broker's list meets this line.
            Answer::Refused(
                AtTheBroker::NotTheAgentService
                | AtTheBroker::NotARequest
                | AtTheBroker::NotOneOfItsVerbs
                | AtTheBroker::NotApproved
                | AtTheBroker::ApprovalSpent
                | AtTheBroker::ApprovalLapsed,
            ) => Err(Self::ApprovalNotAccepted),
        }
    }
}

/// What a person reads once a change was made.
#[must_use]
pub fn changed_said(change: &Change, strings: &Strings) -> Said {
    let word = match change {
        Change::Join(_) => words::JOINED,
        Change::Forget(_) => words::FORGOTTEN,
        Change::Wireless(Switch::On) => words::WIRELESS_ON,
        Change::Wireless(Switch::Off) => words::WIRELESS_OFF,
    };
    strings.say(&word.key(), &filled(change.called()))
}

/// What a person reads once the proxy they chose was set.
#[must_use]
pub fn proxy_set_said(strings: &Strings) -> Said {
    strings.say(&words::PROXY_SET.key(), &Filling::nothing())
}

/// The filling for a sentence about the network called this, or about none.
fn filled(called: Option<&str>) -> Filling {
    called.map_or_else(Filling::nothing, |called| {
        Filling::of(words::NETWORK, called)
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// **Every refusal and every change made is a whole sentence**, with the
    /// network's name where it names one, and no gap left open.
    #[test]
    fn every_refusal_and_every_change_made_is_a_whole_sentence() {
        let strings = in_english();
        let called = || "Home".to_owned();
        for refused in [
            NotChanged::NotANetworkChange,
            NotChanged::NetworkNotAnswering,
            NotChanged::NoneVisibleCalled(called()),
            NotChanged::NoneSavedCalled(called()),
            NotChanged::MoreThanOneCalled(called()),
            NotChanged::AsksForASignIn(called()),
            NotChanged::SaysTheWrongThingAboutThisConversation,
            NotChanged::ConnectionChangedSinceApproval,
            NotChanged::ApprovalNotAccepted,
            NotChanged::CouldNotFinish,
            NotChanged::ProxyNotSet,
            NotChanged::AnOrganisationSetTheProxy,
            NotChanged::NotBeingKept,
            NotChanged::NothingMakesChanges,
        ] {
            let said = refused.said(&strings);
            assert!(!said.is_a_bug(), "{refused:?}: {said}");
            assert!(said.unfilled().is_empty(), "{refused:?}: {said}");
        }
        for change in [
            Change::Join(called()),
            Change::Forget(called()),
            Change::Wireless(Switch::On),
            Change::Wireless(Switch::Off),
        ] {
            let said = changed_said(&change, &strings);
            assert!(!said.is_a_bug(), "{change:?}: {said}");
            assert!(said.unfilled().is_empty(), "{change:?}: {said}");
        }
        assert_eq!(
            changed_said(&Change::Join(called()), &strings).text(),
            "This machine is connected to Home"
        );
        assert!(!proxy_set_said(&strings).is_a_bug());
    }

    /// **A person on a machine an organisation manages reads `alo-proxy`'s own
    /// sentence**, naming who can change it — not a second one written here and
    /// not *the proxy was not set* (ADR 0060 §5).
    #[test]
    fn a_managed_machine_is_refused_in_the_sentence_that_names_who_set_it() {
        let strings = in_english();
        let said = NotChanged::AnOrganisationSetTheProxy.said(&strings);
        assert_eq!(
            said.text(),
            alo_proxy::NotChanged::AnOrganisationSetIt
                .said(&strings)
                .text()
        );
        assert!(said.text().contains("organisation"), "{said}");
        assert!(said.text().contains("Ask whoever manages it"), "{said}");
        assert_ne!(said.text(), NotChanged::ProxyNotSet.said(&strings).text());
    }

    /// **Every answer the broker gives means one thing**, and only `carried` is
    /// a change made.
    #[test]
    fn every_answer_the_broker_gives_means_one_thing() {
        let not_carried = || NotChanged::CouldNotFinish;
        assert_eq!(
            NotChanged::from_the_brokers(Answer::Carried, not_carried()),
            Ok(())
        );
        assert_eq!(
            NotChanged::from_the_brokers(Answer::NotKept, not_carried()),
            Err(NotChanged::NotBeingKept)
        );
        assert_eq!(
            NotChanged::from_the_brokers(
                Answer::Refused(AtTheBroker::NotCarried),
                NotChanged::ProxyNotSet
            ),
            Err(NotChanged::ProxyNotSet)
        );
        for why in [
            AtTheBroker::NotTheAgentService,
            AtTheBroker::NotARequest,
            AtTheBroker::NotOneOfItsVerbs,
            AtTheBroker::NotApproved,
            AtTheBroker::ApprovalSpent,
            AtTheBroker::ApprovalLapsed,
        ] {
            assert_eq!(
                NotChanged::from_the_brokers(Answer::Refused(why), not_carried()).unwrap_err(),
                NotChanged::ApprovalNotAccepted
            );
        }
    }
}
