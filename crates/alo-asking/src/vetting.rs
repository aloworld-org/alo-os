//! Testing a provider before it is saved, with the indicator showing the test.
//!
//! `docs/features.md`, v0.5: *test a provider before saving it, so a mistyped
//! key is found now rather than in the middle of a question.* The test itself
//! is `alo_models::Trying` — one `GET` for the provider's model list, the key
//! as a bearer token, ten seconds, no redirect followed and nothing of the
//! person's in it. What that crate cannot do is put the request on the
//! indicator: `alo-egress` depends on `alo-models`, so the wire cannot reach
//! the indicator, and until this file nothing did. A test request is an egress
//! somebody asked for, and the indicator says so.
//!
//! # The one door, and the order inside it
//!
//! [`Vetting::under`] is the only road from a settings panel to that wire, and
//! it takes the same three steps [`crate::Asking::to_a_provider`] takes, for
//! the same reasons:
//!
//! 1. **The provider has to be one this crate knows how to reach.** What this
//!    crate knows is one convention — the OpenAI-compatible one, which is what
//!    `openai.rs` speaks and `alo_models::trying` asks about — and an address
//!    it can turn into somewhere to connect. A provider with no host in its
//!    address, or a scheme alo OS does not open, is one nothing here will guess
//!    an endpoint for. The honest answer is [`NotVetted::CannotBeTestedFromHere`],
//!    nothing is sent, and the save proceeds as it does today.
//! 2. **The place can be shown and the rule permits it**, or nothing is sent.
//!    `alo_egress::Indicator::beginning` is the only maker of a `Departing`,
//!    it asks the policy and shows the line in one call, and the request is
//!    made holding what it hands back. A provider on this machine is the one
//!    exception and is not an exception to law 1: `alo_models::Provider::source`
//!    calls it this machine, nothing leaves, and there is nothing to show —
//!    which is what lets a person test a local vLLM under the strictest rule
//!    there is.
//! 3. **Only then is the request made**, and the departure comes back inside
//!    [`Vetted`] so what left can be written down ([`crate::vetted`] has that
//!    argument).
//!
//! # Whose egress it is
//!
//! A `Leaving` is under somebody's authority, and a person pressing *Test* is
//! not an agent. The caller names whose it is, exactly as [`crate::Asking::by`]
//! takes its agent from its caller: the surface that made the request on the
//! person's instruction names itself, and this crate invents no name for it.
//! The reason is `alo_egress::Why::Fetching`, because that is what the request
//! does — it fetches the provider's model list and sends nothing of the
//! person's — and the line a person reads is the indicator's own:
//! *@settings is fetching something from Mistral, in the EU*.
//!
//! # What is deliberately not here
//!
//! **No retry and no wait of its own.** One request, once; how long it waits is
//! `alo_models::trying`'s ten seconds, which is somebody watching a dialogue.
//! `tests/a_provider_is_tested_before_it_is_saved.rs` reads this file and that
//! one to make sure of both.
//!
//! **No writing.** Nothing here reaches a settings file or a keyring; a
//! refusal or an unreachable provider therefore writes nothing, because there
//! is nothing here that could. Whether a provider that did not work is saved
//! anyway is the person's to say, and `alo_choosing::Choosing::adding` is the
//! one door that saves it — the same door as today, unchanged.
//!
//! **No second road to the wire.** `alo_models::Trying` is public because this
//! file needs it, and the same integration test reads the workspace to make
//! sure nothing that ships reaches it except `alo-models` itself and this file.

use std::time::SystemTime;

use alo_capability::Grantee;
use alo_egress::{Destination, EgressPolicy, Indicator, Leaving, Why};
use alo_models::{InferenceSource, Provider, Secret, SourcePolicy, Trying};

use crate::found::{Found, NotVetted};
use crate::vetted::Vetted;

/// A provider about to be tested, with the key as it was typed.
///
/// Borrowed rather than owned, for the length of one request, as
/// [`crate::Hosted`] is: neither the provider nor the key is kept anywhere by
/// this, and the key goes in and does not come out.
#[derive(Debug)]
pub struct Vetting<'a> {
    /// What is being tested.
    provider: &'a Provider,
    /// The key, when the provider needs one.
    key: Option<&'a Secret>,
}

impl<'a> Vetting<'a> {
    /// This provider, with this key — which is [`None`] for one that needs
    /// none, such as a service on this machine started without one.
    #[must_use]
    pub fn provider(provider: &'a Provider, key: Option<&'a Secret>) -> Self {
        Self { provider, key }
    }

    /// The host and port the test would connect to, or [`None`] for a
    /// provider this crate does not know how to reach.
    ///
    /// The same answer [`crate::Hosted::where_it_would_connect`] gives, and
    /// the question step one of [`Vetting::under`] asks.
    #[must_use]
    pub fn where_it_would_connect(&self) -> Option<(String, u16)> {
        alo_models::address::where_it_connects(&self.provider.endpoint)
            .filter(|(host, _)| !host.is_empty())
    }

    /// Make the one request, on this machine's indicator and under its rule.
    ///
    /// `by` is whose authority the egress is under — the surface that made the
    /// request on the person's instruction, named by the caller as
    /// [`crate::Asking::by`] is. `now` is the moment, passed in so what the
    /// indicator shows and what a record says cannot disagree about when it
    /// happened.
    ///
    /// # Errors
    /// [`NotVetted`], which is the test **not happening**: nothing was sent in
    /// any of its four cases, and the person may save the provider as they do
    /// today. What a test that did happen found is the [`Found`] inside the
    /// [`Vetted`] this answers with, whichever of its three values it is.
    pub fn under(
        &self,
        by: &Grantee,
        policy: &SourcePolicy,
        indicator: &mut Indicator,
        now: SystemTime,
    ) -> Result<Vetted, NotVetted> {
        // Step one: a provider nothing here can reach is not guessed at.
        if self.where_it_would_connect().is_none() {
            return Err(NotVetted::CannotBeTestedFromHere);
        }

        // A service on this machine leaves nothing, so there is no departure
        // to be made and nothing for the indicator to show — `alo-models`'
        // rule, asked rather than repeated, and the same one `crate::Served`
        // rests on. Every rule permits this machine, so the wire's own check
        // cannot refuse it; the arm is kept because a reporter does not get to
        // assume it passes its own reader's check.
        let source = self.provider.source();
        if source == InferenceSource::ThisMachine {
            return match Found::of(Trying::provider(self.provider, self.key).under(policy)) {
                Ok(found) => Ok(Vetted::new(None, found)),
                Err(refusal) => Err(NotVetted::Forbidden(refusal)),
            };
        }

        // Law 1, in law 1's order: it must be showable, then it must be
        // permitted — and being permitted *is* being shown, because there is
        // one call for the two.
        let destination = Destination::of(&source).map_err(NotVetted::CannotBeShown)?;
        let departing = indicator
            .beginning(
                &EgressPolicy::from(policy),
                Leaving::because(by, Why::Fetching, destination),
                now,
            )
            .map_err(NotVetted::HeldBack)?;

        // And only now. The wire asks the rule again on the way, as
        // `Asking::to_a_provider` lets it: the two rules agree about every
        // source there is, so this is the arm nobody expects to see — and if
        // it is ever taken, nothing left, so the line comes off first.
        match Found::of(Trying::provider(self.provider, self.key).under(policy)) {
            Ok(found) => Ok(Vetted::new(Some(departing), found)),
            Err(refusal) => {
                indicator.ended(departing);
                Err(NotVetted::Forbidden(refusal))
            }
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
    use crate::testing::{in_english, mistral, serving, serving_with};
    use alo_egress::Refusal;
    use alo_models::{NotTried, Region};
    use std::time::Duration;

    /// Two models, in the shape every OpenAI-compatible provider answers with.
    const A_MODEL_LIST: &str = r#"{"object":"list","data":[{"id":"mistral-small-latest","object":"model"},{"id":"mistral-large-latest","object":"model"}]}"#;

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * 12)
    }

    fn settings() -> Grantee {
        Grantee::named("@settings")
    }

    fn key() -> Secret {
        Secret::typed("sk-live-0123456789").unwrap()
    }

    /// A provider that is genuinely somewhere else, at an address nothing is
    /// listening on and that needs no name looked up.
    ///
    /// Not `127.0.0.1`, nor anything on `127.0.0.0/8`: the whole of it **is**
    /// this machine as far as `alo_models::Provider::source` is concerned.
    /// This one is a hosted provider to every question the rule asks and a
    /// refused connection to every question the network asks, so a test that
    /// expects nothing to be sent gets a different answer if something is.
    fn far_away() -> Provider {
        Provider::checked(
            "Mistral",
            "https://0.0.0.0:1",
            Region::Declared("the EU".to_owned()),
            None,
        )
        .unwrap()
    }

    /// What the test found, read off the value it came back in.
    fn found(vetted: &Vetted) -> &Found {
        vetted.found()
    }

    /// The refusal a rule made, if that is what this was.
    ///
    /// A function rather than an `else { panic!(…) }`, because that would be a
    /// test written with the one thing this workspace's lints forbid — and an
    /// `Option` unwrapped in a test says the same thing.
    fn held_back(not_vetted: &NotVetted) -> Option<&alo_egress::NotPermitted> {
        match not_vetted {
            NotVetted::HeldBack(refused) => Some(refused),
            NotVetted::CannotBeTestedFromHere
            | NotVetted::CannotBeShown(_)
            | NotVetted::Forbidden(_) => None,
        }
    }

    /// **A provider on this machine answers, and puts nothing on the
    /// indicator.** The test server is on loopback, which is this machine to
    /// `alo-models`, so the request leaves nothing and there is no line to show
    /// and no departure to record — and the key still travelled as a bearer
    /// token, on one `GET`, with no body.
    #[test]
    fn a_provider_on_this_machine_answers_with_what_it_offers_and_leaves_nothing() {
        let (url, server) = serving(A_MODEL_LIST, 200);
        let provider = Provider::checked("Local", &url, Region::Unknown, None).unwrap();
        let key = key();
        let mut indicator = Indicator::default();

        let vetted = Vetting::provider(&provider, Some(&key))
            .under(
                &settings(),
                &SourcePolicy::ThisMachineOnly,
                &mut indicator,
                noon(),
            )
            .unwrap();
        let request = server.join().unwrap();

        assert!(indicator.is_quiet());
        assert!(!vetted.left());
        assert!(vetted.departing().is_none());
        assert!(request.starts_with("GET /v1/models "), "{request}");
        assert!(
            request
                .to_lowercase()
                .contains("authorization: bearer sk-live-0123456789\r\n"),
            "{request}"
        );
        assert!(
            !request.to_lowercase().contains("content-length"),
            "{request}"
        );

        let found = vetted.ended(&mut indicator);
        assert!(found.is_a_working_provider());
        assert!(
            matches!(&found, Found::Answered(tried) if tried.models() == ["mistral-small-latest", "mistral-large-latest"]),
            "{found:?}"
        );
        assert!(
            found.said(&in_english()).text().starts_with("that worked"),
            "{found:?}"
        );
        assert!(found.caveats(&in_english()).is_empty());
    }

    /// **The whole reason the feature exists**: the mistyped key is found while
    /// the person is still looking at the field — and the sentence they read
    /// does not quote the key or the provider's own words.
    #[test]
    fn a_key_the_provider_refuses_is_found_now_and_the_sentence_does_not_carry_it() {
        let (url, server) = serving(r#"{"message":"Unauthorized","request_id":"abc"}"#, 401);
        let provider = Provider::checked("Local", &url, Region::Unknown, None).unwrap();
        let key = key();
        let mut indicator = Indicator::default();

        let vetted = Vetting::provider(&provider, Some(&key))
            .under(&settings(), &SourcePolicy::Anywhere, &mut indicator, noon())
            .unwrap();
        server.join().unwrap();

        assert_eq!(
            found(&vetted),
            &Found::RefusedTheKey(NotTried::KeyNotAccepted)
        );
        let said = vetted.ended(&mut indicator).said(&in_english());
        assert!(said.text().contains("the whole key"), "{said}");
        assert!(!said.text().contains("sk-live"), "{said}");
        assert!(!said.text().contains("request_id"), "{said}");
    }

    /// A provider that wants a key and was given none is told apart from one
    /// that refused the key it was given: both are [`Found::RefusedTheKey`],
    /// and the reason inside says which.
    #[test]
    fn a_provider_given_no_key_that_wants_one_says_so_rather_than_blaming_a_key() {
        let (url, server) = serving(r#"{"message":"Unauthorized"}"#, 401);
        let provider = Provider::checked("Local", &url, Region::Unknown, None).unwrap();
        let mut indicator = Indicator::default();

        let vetted = Vetting::provider(&provider, None)
            .under(&settings(), &SourcePolicy::Anywhere, &mut indicator, noon())
            .unwrap();
        server.join().unwrap();

        assert_eq!(found(&vetted), &Found::RefusedTheKey(NotTried::NeedsAKey));
        let said = vetted.ended(&mut indicator).said(&in_english());
        assert!(said.text().contains("add the one it"), "{said}");
    }

    /// **Something answered and it was not a provider, a redirect, and a
    /// provider having a bad day** are all a working provider that could not
    /// be reached, and each one keeps its own sentence.
    #[test]
    fn the_ways_a_working_provider_is_not_reached_each_keep_their_own_reason() {
        for (body, status, location, expected) in [
            (
                r#"{"hello":"world"}"#,
                200,
                "",
                Found::CouldNotBeReached(NotTried::NotUnderstood),
            ),
            (
                "{}",
                302,
                "Location: http://127.0.0.1:1/v1/models\r\n",
                Found::CouldNotBeReached(NotTried::Redirected),
            ),
            (
                r#"{"error":"upstream capacity exceeded"}"#,
                503,
                "",
                Found::CouldNotBeReached(NotTried::NotWell(503)),
            ),
        ] {
            let (url, server) = serving_with(body, status, location);
            let provider = Provider::checked("Local", &url, Region::Unknown, None).unwrap();
            let key = key();
            let mut indicator = Indicator::default();
            let vetted = Vetting::provider(&provider, Some(&key))
                .under(&settings(), &SourcePolicy::Anywhere, &mut indicator, noon())
                .unwrap();
            server.join().unwrap();
            assert_eq!(found(&vetted), &expected, "{status}: {body}");
            assert!(!vetted.ended(&mut indicator).is_a_working_provider());
        }
    }

    /// **A request to a provider somewhere else is on the indicator while it
    /// happens**, in the indicator's own words, and the departure comes back
    /// so what left can be written down — on the path where nothing answered,
    /// because a request that failed still left the machine.
    #[test]
    fn a_test_request_that_leaves_is_shown_leaving_and_the_departure_comes_back() {
        let provider = far_away();
        let key = key();
        let mut indicator = Indicator::default();
        assert!(indicator.is_quiet());

        let vetted = Vetting::provider(&provider, Some(&key))
            .under(&settings(), &SourcePolicy::Anywhere, &mut indicator, noon())
            .unwrap();

        // Still showing, because the record has not been written yet and the
        // departure is the only thing that can write it.
        assert!(vetted.left());
        assert_eq!(indicator.showing().len(), 1);
        assert_eq!(
            indicator
                .showing()
                .first()
                .map(|shown| shown.said(&in_english()).text().to_owned()),
            Some("@settings is fetching something from Mistral, in the EU".to_owned())
        );
        let departing = vetted.departing().unwrap();
        assert_eq!(departing.agent(), &settings());
        assert_eq!(departing.why(), Why::Fetching);
        assert_eq!(departing.at(), noon());
        assert_eq!(
            departing.destination(),
            &Destination::provider("Mistral", Region::Declared("the EU".to_owned())).unwrap()
        );

        let found = vetted.ended(&mut indicator);
        assert!(indicator.is_quiet());
        assert_eq!(found, Found::CouldNotBeReached(NotTried::Unreachable));
        assert!(
            found
                .said(&in_english())
                .text()
                .contains("check the address"),
            "{found:?}"
        );
    }

    /// **The refusal that matters most, and one test per rule.** A machine an
    /// organisation has set to keep questions in the building does not reach
    /// out to a provider to find out whether a key works. Nothing is sent, the
    /// indicator stays quiet because nothing is leaving, and the refusal is the
    /// rule's own so a person reads one account of one moment.
    #[test]
    fn a_rule_that_forbids_it_sends_nothing_and_the_indicator_stays_quiet() {
        for (policy, expected) in [
            (SourcePolicy::InTheBuilding, Refusal::OutsideTheBuilding),
            (
                SourcePolicy::InRegion("Switzerland".to_owned()),
                Refusal::OutsideTheRegion {
                    region: "Switzerland".to_owned(),
                },
            ),
            (SourcePolicy::ThisMachineOnly, Refusal::NothingMayLeave),
        ] {
            let provider = far_away();
            let key = key();
            let mut indicator = Indicator::default();

            let not_vetted = Vetting::provider(&provider, Some(&key))
                .under(&settings(), &policy, &mut indicator, noon())
                .unwrap_err();

            let refused = held_back(&not_vetted).unwrap();
            assert_eq!(refused.why(), &expected, "{policy:?}");
            assert_eq!(refused.leaving().agent(), &settings());
            let said = not_vetted.said(&in_english());
            assert!(said.text().contains("this machine"), "{policy:?}: {said}");
            assert!(said.text().contains("Mistral"), "{policy:?}: {said}");
            assert!(indicator.is_quiet(), "{policy:?}");
        }
    }

    /// The rule that permits everything permits this, and a region the provider
    /// stated satisfies a rule naming it — so the request is made and shown.
    #[test]
    fn the_rules_that_permit_it_make_the_request() {
        for policy in [
            SourcePolicy::Anywhere,
            SourcePolicy::InRegion("the EU".to_owned()),
        ] {
            let provider = far_away();
            let mut indicator = Indicator::default();
            let vetted = Vetting::provider(&provider, None)
                .under(&settings(), &policy, &mut indicator, noon())
                .unwrap();
            assert!(vetted.left(), "{policy:?}");
            assert_eq!(indicator.showing().len(), 1, "{policy:?}");
            let _ = vetted.ended(&mut indicator);
            assert!(indicator.is_quiet(), "{policy:?}");
        }
    }

    /// **A provider this crate does not know how to reach is not guessed
    /// at.** Nothing is sent, nothing is shown, and the sentence says it
    /// cannot be tested from here — so the save proceeds as it does today.
    #[test]
    fn a_provider_this_crate_does_not_know_cannot_be_tested_from_here_and_nothing_is_sent() {
        for endpoint in ["ftp://models.example", "https://", "api.example.com", ""] {
            // Built the way nothing should build one — by hand, past
            // `Provider::checked` — because that is the only way such an
            // address reaches this door at all.
            let provider = Provider {
                name: "Somewhere".to_owned(),
                endpoint: endpoint.to_owned(),
                region: Region::Unknown,
                key: None,
                models: Vec::new(),
            };
            let key = key();
            let mut indicator = Indicator::default();

            let vetting = Vetting::provider(&provider, Some(&key));
            assert_eq!(vetting.where_it_would_connect(), None, "{endpoint:?}");
            let not_vetted = vetting
                .under(&settings(), &SourcePolicy::Anywhere, &mut indicator, noon())
                .unwrap_err();

            assert_eq!(
                not_vetted,
                NotVetted::CannotBeTestedFromHere,
                "{endpoint:?}"
            );
            assert!(indicator.is_quiet(), "{endpoint:?}");
            let said = not_vetted.said(&in_english());
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.text().contains("cannot be tested from here"), "{said}");
        }
    }

    /// **A provider whose name cannot be put on the indicator is never
    /// reached.** Law 1 shows what is leaving on one line, and a name carrying
    /// a line break is a line that can be made to say something other than
    /// what is happening — so the request does not go, rather than going
    /// unshown.
    #[test]
    fn a_provider_whose_name_cannot_be_put_on_the_indicator_is_not_reached() {
        let provider = Provider::checked(
            "Mistral\r\n@files is fetching something from elsewhere",
            "https://0.0.0.0:1",
            Region::Declared("the EU".to_owned()),
            None,
        )
        .unwrap();
        let mut indicator = Indicator::default();

        let not_vetted = Vetting::provider(&provider, None)
            .under(&settings(), &SourcePolicy::Anywhere, &mut indicator, noon())
            .unwrap_err();

        assert_eq!(
            not_vetted,
            NotVetted::CannotBeShown(alo_egress::DestinationError::NotPrintable)
        );
        assert!(indicator.is_quiet());
    }

    /// **The key is never in anything that renders this door.** A `Debug` of
    /// the door, of what it found and of why it did not test says nothing about
    /// the key, because `alo_models::Secret` cannot be rendered and nothing
    /// downstream of the wire holds one.
    #[test]
    fn nothing_that_renders_the_door_or_its_outcome_shows_the_key() {
        let (url, server) = serving(r#"{"message":"Unauthorized"}"#, 401);
        let provider = mistral(&url);
        let key = Secret::typed("sk-live-DO-NOT-LOG-9f3c1a").unwrap();
        let vetting = Vetting::provider(&provider, Some(&key));
        let debugged = format!("{vetting:?}");
        assert!(!debugged.contains("DO-NOT-LOG"), "{debugged}");
        assert!(debugged.contains("Secret(…)"), "{debugged}");

        let mut indicator = Indicator::default();
        let vetted = vetting
            .under(&settings(), &SourcePolicy::Anywhere, &mut indicator, noon())
            .unwrap();
        server.join().unwrap();
        let debugged = format!("{vetted:?}");
        assert!(!debugged.contains("DO-NOT-LOG"), "{debugged}");
        let found = vetted.ended(&mut indicator);
        assert!(!format!("{found:?}").contains("DO-NOT-LOG"));
        assert!(!found.said(&in_english()).text().contains("DO-NOT-LOG"));
    }

    /// Where the test would connect is the host and the scheme's port, read
    /// the way `alo-models` reads an address — and an address with no host in
    /// it is nowhere.
    #[test]
    fn where_the_test_would_connect_is_the_providers_host() {
        let provider = far_away();
        assert_eq!(
            Vetting::provider(&provider, None).where_it_would_connect(),
            Some(("0.0.0.0".to_owned(), 1))
        );
        let mistral = Provider::checked(
            "Mistral",
            "https://api.mistral.ai/v1",
            Region::Unknown,
            None,
        )
        .unwrap();
        assert_eq!(
            Vetting::provider(&mistral, None).where_it_would_connect(),
            Some(("api.mistral.ai".to_owned(), 443))
        );
    }
}
