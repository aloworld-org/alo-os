//! Which way out of this machine a question's road takes — asked here, decided
//! in `alo-proxy`.
//!
//! [`crate::machine_wide_proxy`] reads `[proxy]` out of `/etc/alo/agentd.toml`
//! into an `alo_proxy::Kept`, and [`crate::described::Described::proxy`] is
//! where that value ends up. Until this file existed **nothing in this service
//! handed it to anything**: on a company network with no other route out, a
//! machine read its organisation's proxy off the disk and then asked a provider
//! directly. A setting that exists, is shown, and does nothing is the one
//! failure `alo-proxy`'s own header calls worse than having no setting at all.
//!
//! # Nothing here decides which way a road goes
//!
//! `alo_proxy::the_way` decides, for `alo_proxy::Road::AskingAProvider`, and
//! this file contains no branch on the setting's shape, no exception list, no
//! loopback check and nowhere one could be added. What it does is turn *this
//! provider* into the two facts a proxy decision is made from — the scheme and
//! the host, which is `alo_proxy::Reaching` — ask, and hand the answer on as an
//! `alo_proxy::Carried`.
//!
//! That is the whole of the constraint in
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 11: *nothing here
//! re-decides which way a road goes — `alo-proxy` decides, and this is the
//! crate that asks it.*
//!
//! # Two refusals, and neither of them is a road going straight out instead
//!
//! [`NotTaken`] is both ways this can fail, and on both of them **nothing is
//! sent**:
//!
//! | | |
//! |---|---|
//! | the machine is told to ask an automatic configuration and cannot | `alo_proxy::NotOnTheRoad`, in that crate's own words |
//! | the provider's address is not somewhere a road can be decided about | [`crate::words::NO_ROAD_TO_THAT_PROVIDER`] |
//!
//! The first is the one that matters on a company network: a machine that
//! quietly went straight out when its configuration could not be worked out
//! would be sending a company's traffic around the company's own rule, with
//! nobody told. `alo-egress` refuses a destination its policy cannot permit for
//! the same reason, one step later.
//!
//! The second is narrow and real rather than defensive. `alo_models::Provider`
//! checks that an endpoint is `http`/`https` and carries no credential, and so
//! this cannot be a scheme nobody recognises — but it does not bound how long a
//! host may be, and `alo_proxy::Reaching` does, because a host becomes an
//! argument to a program and a line a person reads.
//!
//! # A proxy that asks for a password is not signed in to yet, on any road
//!
//! `alo_proxy::Carried::with_the_password` is how a credential travels, and
//! **no road in this workspace calls it** — not the rented tool's, not the
//! base's, and not this one. `[proxy]` names the keyring entry the password is
//! kept under (ADR 0022 is about a *provider's* key and settles nothing about
//! this one), and which store a machine-wide password is read from, by a
//! service running as root before anybody has signed in, is a decision nobody
//! has taken. Writing one here would be this file taking it quietly for every
//! road at once. A proxy that asks for no password works today; one that does
//! is refused by the proxy itself, which is at least a sentence somebody can
//! act on.

use alo_models::Provider;
use alo_proxy::{
    Carried, Kept, NotOnTheRoad, NotReachable, Reaching, Road, TheProxy, TheRentedEvaluator,
    the_way,
};
use alo_strings::{Filling, Said, Strings};

use crate::words::{NO_ROAD_TO_THAT_PROVIDER, Word};

/// What a machine nobody set a proxy on is asked about.
///
/// [`crate::machine_wide_proxy`] keeps *nobody set one* and *somebody wrote
/// straight out* apart, and they stay apart: what differs is who may change it
/// and what a person is told, which is `alo_proxy::Kept`'s. **Which way a road
/// goes, the two decide identically**, and they decide it through the same one
/// door rather than by this file answering for the absent case itself.
const NOBODYS: TheProxy = TheProxy::None;

/// The machine's one proxy, as its description states it, ready to be asked
/// about one road.
///
/// Held for the life of the service beside the rest of what the description
/// said, and asked per question: the setting cannot change under a running
/// daemon — `/etc/alo/agentd.toml` is read once at startup — but where a
/// question is going can, because the person may choose another provider
/// between one turn and the next.
///
/// Cloned rather than borrowed where it is read, because what a question's
/// answer borrows is the [`crate::questions::Questions`] this lives in and the
/// road out is wanted in the middle of that. It is a setting and a path; there
/// is no credential in it and nothing that could be spent by copying.
#[derive(Debug, Clone)]
pub struct TheRoadOut {
    /// The setting and who set it, or [`None`] where nobody set one.
    kept: Option<Kept>,

    /// Where an automatic configuration is worked out: a separate program with
    /// a cleared environment, which holds no grant because nothing ever granted
    /// it one (`alo_proxy::evaluator`).
    ///
    /// **A machine with nothing at that path refuses** every road under an
    /// automatic configuration, which is `alo-proxy`'s behaviour and not a
    /// second rule here.
    evaluating: TheRentedEvaluator,
}

impl Default for TheRoadOut {
    /// A machine nobody told about a proxy, which is the common case and not a
    /// gap — the same shape `crate::questions::TheBound::Nobodys` has one file
    /// over.
    fn default() -> Self {
        Self::nobodys()
    }
}

impl TheRoadOut {
    /// A machine whose description says nothing about a proxy.
    #[must_use]
    pub fn nobodys() -> Self {
        Self::of(None)
    }

    /// The proxy this machine's description states, or its absence.
    ///
    /// The evaluator is the one an alo OS machine has, at the path
    /// `alo_proxy::THE_EVALUATOR` names. There is no parameter for it and no
    /// key in any file that could name another: an automatic configuration is
    /// worked out by the image's own program or by nothing, and *by nothing*
    /// refuses rather than going straight out.
    #[must_use]
    pub fn of(kept: Option<Kept>) -> Self {
        Self {
            kept,
            evaluating: TheRentedEvaluator::on_this_machine(),
        }
    }

    /// The same machine, with the evaluator somewhere else.
    ///
    /// **A test seam, and it is `cfg(test)` on purpose**, for
    /// `crate::questions::Questions::already_found`'s reason: a constructor
    /// production could reach would be a way to point the program that decides
    /// this machine's way out at something else, through a door nobody
    /// reviewing `of` would look at.
    #[cfg(test)]
    pub(crate) fn evaluated_by(kept: Option<Kept>, evaluating: TheRentedEvaluator) -> Self {
        Self { kept, evaluating }
    }

    /// The setting a road is decided against, with absence answering as the
    /// setting nobody wrote.
    fn proxy(&self) -> &TheProxy {
        self.kept.as_ref().map_or(&NOBODYS, Kept::proxy)
    }

    /// Whose the setting is, or [`None`] where nobody set one.
    ///
    /// Not read to decide anything — `alo_proxy::the_way` never asks who set a
    /// proxy — and here because a person changing one on a managed machine is
    /// refused in words naming who set it, which is `alo_proxy::NotChanged`'s.
    #[must_use]
    pub const fn kept(&self) -> Option<&Kept> {
        self.kept.as_ref()
    }

    /// Which way the road to this provider goes, and what the request is then
    /// configured with.
    ///
    /// Called at the question rather than at startup, because which provider a
    /// question goes to is the person's and may change between turns.
    ///
    /// # Errors
    /// [`NotTaken`], on either of which **nothing is sent** — see this file's
    /// second section.
    pub fn to(&self, provider: &Provider) -> Result<Carried, NotTaken> {
        let going_to =
            Reaching::of(&provider.endpoint).map_err(NotTaken::NotSomewhereWithARoadOut)?;
        let way = the_way(
            self.proxy(),
            Road::AskingAProvider,
            &going_to,
            &self.evaluating,
        )
        .map_err(NotTaken::TheWayOutCouldNotBeDecided)?;
        Ok(Carried::of(way))
    }
}

/// Why a question was not put, because there was no road to put it on.
///
/// **No `Display`**: both of these reach somebody at the other end of a
/// connection, and the only road to words is [`NotTaken::said`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotTaken {
    /// This machine is told to ask an automatic configuration where the road
    /// goes, and could not.
    ///
    /// `alo-proxy` decided it and `alo-proxy` words it; nothing here rewords
    /// it, for [`crate::doing`]'s rule that a refusal crosses in the words of
    /// whoever refused it.
    TheWayOutCouldNotBeDecided(NotOnTheRoad),

    /// The provider's address is not somewhere a road out can be decided
    /// about, so no way out was asked for.
    NotSomewhereWithARoadOut(NotReachable),
}

impl NotTaken {
    /// What this says, in the language the person reads.
    ///
    /// **The road was refused before this was called**, as `alo-proxy` and
    /// `alo-egress` both arrange their own: a machine whose translations failed
    /// to load refuses exactly what it refused before.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::TheWayOutCouldNotBeDecided(why) => why.said(strings),
            Self::NotSomewhereWithARoadOut(_) => {
                strings.say(&Self::OUR_OWN.key(), &Filling::nothing())
            }
        }
    }

    /// The one string this crate declares for these two, which is the second of
    /// them: the first speaks in `alo-proxy`'s.
    const OUR_OWN: Word = NO_ROAD_TO_THAT_PROVIDER;

    /// What is kept for whoever administers the machine, never shown.
    ///
    /// `alo_proxy::NotOnTheRoad::because` is the half of the first refusal that
    /// came from outside this machine — what a company's own script printed —
    /// and it is answered here so that the one caller does not have to match on
    /// the variant to find out whether there is one.
    #[must_use]
    pub const fn because(&self) -> Option<&alo_proxy::NotEvaluated> {
        match self {
            Self::TheWayOutCouldNotBeDecided(why) => Some(why.because()),
            Self::NotSomewhereWithARoadOut(_) => None,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_models::Region;
    use alo_proxy::{ConfigurationAddress, NotEvaluated, ProxyAddress, SpokenTo};

    /// The company's proxy, as somebody was handed it on a slip of paper.
    fn the_companys() -> ProxyAddress {
        ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).unwrap()
    }

    /// A provider somewhere else, which is the only kind this road goes to.
    fn a_provider() -> Provider {
        Provider::checked(
            "Mine",
            "https://api.example.com",
            Region::Declared("the EU".to_owned()),
            None,
        )
        .unwrap()
    }

    /// A machine told to ask a configuration, with nothing at the path that
    /// would answer — which is every machine but an alo OS one.
    fn told_to_ask_a_configuration() -> TheRoadOut {
        TheRoadOut::evaluated_by(
            Some(Kept::by_an_organisation(TheProxy::Automatic {
                at: ConfigurationAddress::checked("http://wpad.example.com/c").unwrap(),
            })),
            TheRentedEvaluator::at(std::path::Path::new(
                "/nowhere/alo-agentd-has-no-evaluator-here",
            )),
        )
    }

    /// **The proxy the description states reaches the road a question takes.**
    #[test]
    fn a_question_takes_the_proxy_the_description_states() {
        let road = TheRoadOut::of(Some(Kept::by_an_organisation(
            TheProxy::one(the_companys()),
        )));
        let carried = road.to(&a_provider()).unwrap();

        assert_eq!(
            carried.shown(),
            Some("http://proxy.example.com:8080".to_owned())
        );
        assert!(
            carried
                .for_a_request()
                .expect("the client can use it")
                .is_some(),
            "the request was configured with no proxy"
        );
    }

    /// **A machine whose description states no proxy goes straight out, said.**
    ///
    /// `None` is what the request is configured with, rather than the client
    /// being left to find one in this process's environment.
    #[test]
    fn a_machine_with_no_section_goes_straight_out_and_says_so() {
        for road in [TheRoadOut::nobodys(), TheRoadOut::default()] {
            let carried = road.to(&a_provider()).unwrap();
            assert!(carried.is_straight());
            assert_eq!(carried.for_a_request().expect("nothing to refuse"), None);
            assert_eq!(carried.shown(), None);
        }
    }

    /// A description that says *straight out* in so many words decides the same
    /// way, and is still a setting somebody wrote rather than an absence.
    #[test]
    fn a_description_that_says_straight_out_decides_the_same_way() {
        let road = TheRoadOut::of(Some(Kept::by_this_person(TheProxy::None)));
        assert!(road.to(&a_provider()).unwrap().is_straight());
        assert!(road.kept().is_some(), "somebody set this one");
        assert!(
            TheRoadOut::nobodys().kept().is_none(),
            "nobody set that one"
        );
    }

    /// **A machine told to ask a configuration it cannot refuses the road**,
    /// and never quietly goes straight out instead.
    #[test]
    fn a_configuration_this_machine_cannot_work_out_refuses_the_road() {
        let refused = told_to_ask_a_configuration()
            .to(&a_provider())
            .expect_err("a road nobody could work out was taken");

        assert_eq!(
            refused,
            NotTaken::TheWayOutCouldNotBeDecided(NotOnTheRoad::CouldNotBeWorkedOut(
                NotEvaluated::NothingEvaluatesIt
            ))
        );
        assert_eq!(refused.because(), Some(&NotEvaluated::NothingEvaluatesIt));

        // And it is said in `alo-proxy`'s own words rather than reworded here.
        let strings = crate::testing::in_english();
        assert_eq!(
            refused.said(&strings).text(),
            NotOnTheRoad::CouldNotBeWorkedOut(NotEvaluated::NothingEvaluatesIt)
                .said(&strings)
                .text()
        );
    }

    /// **A provider whose address no road can be decided about is refused**, in
    /// this crate's own sentence, and nothing is asked of any evaluator.
    ///
    /// Built as a value rather than through `Provider::checked`, because that
    /// constructor is what makes this rare: an endpoint reaching here is
    /// already `http` or `https` and already carries no credential. What it does
    /// not bound is how long a host may be, and `alo_proxy::Reaching` does.
    #[test]
    fn a_provider_whose_address_has_no_road_is_refused_in_our_own_words() {
        let provider = Provider {
            name: "Mine".to_owned(),
            endpoint: format!("https://{}", "a".repeat(400)),
            region: Region::Unknown,
            key: None,
            models: Vec::new(),
        };
        let refused = told_to_ask_a_configuration()
            .to(&provider)
            .expect_err("a road was decided for an address that is not one");

        assert_eq!(
            refused,
            NotTaken::NotSomewhereWithARoadOut(NotReachable::TooLong)
        );
        assert_eq!(refused.because(), None);

        let strings = crate::testing::in_english();
        let said = refused.said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(
            !said
                .text()
                .contains(&provider.endpoint[..30.min(provider.endpoint.len())]),
            "the sentence repeats what was written: {said}"
        );
    }

    /// **A provider on this machine is never reached through the proxy**,
    /// whatever the description says — which is `alo_proxy::the_way`'s answer
    /// about loopback and not a second rule here.
    #[test]
    fn a_provider_at_this_machines_own_address_is_never_proxied() {
        let here = Provider::checked("Here", "http://127.0.0.1:11434", Region::Unknown, None)
            .expect("a service on this machine");
        for road in [
            TheRoadOut::of(Some(Kept::by_an_organisation(
                TheProxy::one(the_companys()),
            ))),
            told_to_ask_a_configuration(),
        ] {
            let carried = road.to(&here).expect("nothing refuses this machine");
            assert!(carried.is_straight());
            assert_eq!(carried.for_a_request().expect("nothing to refuse"), None);
        }
    }
}
