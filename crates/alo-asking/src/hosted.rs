//! A provider somewhere else, and what the person is told it is called.
//!
//! The wire itself is `openai.rs` — the convention this and
//! [`crate::served`] both speak, in one file, because two renderings of one
//! protocol is two things that can disagree about what left the machine. What is
//! here is the half that is about a *provider*: the name and region a person
//! wrote down, and how long this machine waits for somebody else's service.
//!
//! # The road out is the machine's decision, and it arrives here decided
//!
//! On a company network there is frequently no other way out, so this door
//! takes whichever way `alo_proxy::the_way` answered for
//! [`Road::AskingAProvider`](alo_proxy::Road::AskingAProvider) — handed in by
//! [`Hosted::taking`] as an [`alo_proxy::Carried`] and never worked out here.
//! Two things follow from it, and both are stated on
//! [`Hosted::where_it_would_connect`] and on `Hosted::ask`: **what this machine
//! actually connects to is the proxy**, so that is what a caller registers with
//! the boundary (ADR 0020), and **what the answer says it came from is still
//! the provider**, because the destination never changed.
//!
//! # Nothing here opens anything on its own
//!
//! `Hosted::ask` is `pub(crate)` and its one caller is
//! [`crate::Asking::to_a_provider`], which has already obtained an
//! `alo_egress::Departing` — so there is no public function reaching a provider
//! without law 1's indicator having shown it first. The guarantee is
//! `alo-egress`' and this crate does not get a second way round it.
//!
//! [`crate::served`] reaches the same wire without a `Departing`, which is the
//! one exception and is not an exception to law 1: it will not carry a question
//! to an address that is not this machine, so there is nothing for an indicator
//! to show. That file makes its own argument for it.

use std::time::Duration;

use alo_answering::WentWrong;
use std::net::SocketAddr;

use alo_models::{InferenceSource, Provider, Secret};
use alo_proxy::Carried;

use crate::openai;
use crate::question::Question;

/// How long this machine waits for an answer from somebody else.
///
/// Shorter than the five minutes [`crate::served`] waits for a service on this
/// machine, which is ADR 0007's asymmetry rather than an oversight: a model
/// thinking on this machine's CPU is the ordinary case and a provider is a
/// datacentre. Longer than the ten seconds `alo_models::trying` waits, because
/// that is somebody watching a dialogue and this is a model thinking. Short
/// enough that a provider which has stopped answering is *said* to have stopped
/// rather than left looking like a machine that has hung.
const WHILE_A_MODEL_THINKS: Duration = Duration::from_secs(120);

/// A provider a person added, and the key it was given.
///
/// Borrowed rather than owned, for the length of one question: neither the
/// provider nor the key is kept anywhere by this.
#[derive(Debug)]
pub struct Hosted<'a> {
    /// Where to reach it, and what it is called.
    provider: &'a Provider,
    /// The key, for a provider that needs one.
    key: Option<&'a Secret>,
    /// The way out this machine decided for the road to a provider, or
    /// [`None`] where nobody has handed one over.
    ///
    /// Borrowed for the same length as the other two: an [`Carried`] can hold a
    /// credential and `alo-proxy` keeps it deliberately un-`Clone`, so it lives
    /// in the caller's frame and this reads it for the length of one question.
    through: Option<&'a Carried>,
}

impl<'a> Hosted<'a> {
    /// This provider, with this key — which is [`None`] for one that needs
    /// none.
    ///
    /// The road out is straight until [`Hosted::taking`] says otherwise, and
    /// **that is explicit rather than inherited**: see that method.
    #[must_use]
    pub fn provider(provider: &'a Provider, key: Option<&'a Secret>) -> Self {
        Self {
            provider,
            key,
            through: None,
        }
    }

    /// The same provider, reached the way this machine decided.
    ///
    /// `alo_proxy::the_way` decides it, for
    /// [`Road::AskingAProvider`](alo_proxy::Road::AskingAProvider) and for
    /// where this provider is; nothing here re-decides any of it, and there is
    /// no argument through which it could. *A great many company networks have
    /// no other route out*, so a provider that could not be reached through the
    /// machine's proxy could not be reached at all.
    ///
    /// **Straight out is said rather than left unsaid**, which is why a caller
    /// that decided *straight* still calls this, with a
    /// [`Carried::straight`]. `ureq::Config::default` reads `HTTP_PROXY` out of
    /// whatever process this happens to be running in, and a road decided by a
    /// variable is a road nobody chose and nobody can be shown; `crate::openai`
    /// therefore says which it is on every request, and
    /// `alo_models::Trying::taking` is the same rule one crate over.
    #[must_use]
    pub const fn taking(mut self, through: &'a Carried) -> Self {
        self.through = Some(through);
        self
    }

    /// Where an answer from this provider would say it came from, out of what
    /// the person wrote down when they added it.
    ///
    /// **Not `alo_models::Provider::source`**, and the difference is worth the
    /// sentence. That answers *is this address on this machine*, which is a
    /// fact about the endpoint; this answers *what would the indicator say*,
    /// which is a fact about the name and the region a person stated. This one
    /// is what [`crate::Asking`] checks the permission against, because the
    /// line law 1 shows is composed out of the source and the connection is
    /// opened out of the endpoint — and a machine where those two disagree is a
    /// machine telling somebody their question went somewhere it did not.
    ///
    /// A provider on this machine is therefore named here as though it were
    /// somewhere else, and that mismatch is exactly what
    /// [`crate::Asking::to_a_provider`] refuses: `Provider::source` calls it
    /// this machine, the permission is for this machine, and the two do not
    /// meet. [`crate::Served`] is the door such a provider belongs to.
    #[must_use]
    pub fn named_source(&self) -> InferenceSource {
        InferenceSource::Hosted {
            provider: self.provider.name.clone(),
            region: self.provider.region.clone(),
        }
    }

    /// Put the question, and read what comes back.
    ///
    /// `pub(crate)`, and the only caller is [`crate::Asking::to_a_provider`],
    /// which holds an `alo_egress::Departing` by the time it gets here. That is
    /// the whole of why this is not public: a public method here would be a way
    /// to reach a provider without law 1 having shown it.
    ///
    /// The request is configured with the way out that was decided — **including
    /// when that way is straight**, which is what stops the client reading a
    /// proxy out of this process's environment.
    ///
    /// # Errors
    /// [`WentWrong`], as `openai::put_through` answers it, and
    /// [`WentWrong::NoWayThere`] for a proxy address the client cannot use.
    /// That last one is a refusal rather than a road quietly going straight
    /// out: a machine reaching around its company's own rule with nobody told
    /// is the failure `alo-proxy` exists to prevent.
    pub(crate) fn ask(&self, question: &Question, to: &[SocketAddr]) -> Result<String, WentWrong> {
        let through = match self.through {
            None => None,
            Some(carried) => carried.for_a_request().map_err(|_| WentWrong::NoWayThere)?,
        };
        openai::put_through(
            through,
            &self.provider.endpoint,
            self.key,
            question,
            WHILE_A_MODEL_THINKS,
            to,
            None,
        )
    }

    /// The host and port this would connect to, for somebody to resolve and
    /// register before the boundary is entered (ADR 0020).
    ///
    /// **The proxy's, on a road going through one**, and the provider's
    /// otherwise. What ADR 0020 asks to be registered is where the socket
    /// really opens, and on a proxied road that is the proxy: the client is
    /// handed these addresses as its whole resolver, speaks `CONNECT` to them
    /// and never looks the provider's name up at all. Registering the
    /// provider's instead would bound a turn to an address it does not use and
    /// leave the one it does use unbounded, which is both halves of ADR 0020
    /// wrong at once.
    ///
    /// It is **not** what the indicator is built from. That is
    /// [`Hosted::named_source`], which names the provider whichever way the
    /// road goes, because the destination did not change — `alo_proxy::road`
    /// makes the same argument from the other end.
    ///
    /// [`None`] for an endpoint with no host, which is one no request can be
    /// made to either.
    #[must_use]
    pub fn where_it_would_connect(&self) -> Option<(String, u16)> {
        match self.through.and_then(|carried| carried.way().through()) {
            Some(proxy) => Some((proxy.host().to_owned(), proxy.port())),
            None => alo_models::address::where_it_connects(&self.provider.endpoint),
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
    use crate::testing::{mistral, serving};

    /// One answer, in the shape every OpenAI-compatible provider replies with.
    const AN_ANSWER: &str = r#"{"choices":[{"index":0,"message":{"role":"assistant","content":"The tenant may not sublet."}}]}"#;

    fn question() -> Question {
        Question::asked("may the tenant sublet?", "mistral-small-latest").unwrap()
    }

    /// What the indicator would say about this provider is made out of what the
    /// person wrote down, and it is what the permission is checked against.
    #[test]
    fn what_this_provider_would_be_called_is_what_the_person_named_it() {
        let provider = mistral("https://api.mistral.ai");
        assert_eq!(
            Hosted::provider(&provider, None).named_source(),
            crate::testing::mistral_source()
        );
    }

    /// **A road going through a proxy connects to the proxy**, so that is what
    /// a caller registers with the boundary (ADR 0020) — while the line the
    /// indicator is built from still names the provider.
    ///
    /// The two halves are asserted together on purpose: they are the pair that
    /// must not be confused, and a change that made one of them follow the
    /// other would be a machine either telling somebody their question went
    /// somewhere it did not, or bounding a turn to an address it never uses.
    #[test]
    fn a_road_through_a_proxy_connects_to_the_proxy_and_is_still_named_for_the_provider() {
        let provider = mistral("https://api.mistral.ai");
        let proxy =
            alo_proxy::ProxyAddress::checked(alo_proxy::SpokenTo::Http, "proxy.example.com", 8080)
                .unwrap();
        let through = Carried::through(proxy);

        let hosted = Hosted::provider(&provider, None).taking(&through);
        assert_eq!(
            hosted.where_it_would_connect(),
            Some(("proxy.example.com".to_owned(), 8080))
        );
        assert_eq!(hosted.named_source(), crate::testing::mistral_source());
    }

    /// **A road going straight out connects to the provider**, and says so
    /// rather than leaving the client to find a proxy in this process's
    /// environment.
    #[test]
    fn a_road_going_straight_out_connects_to_the_provider() {
        let provider = mistral("https://api.mistral.ai");
        let straight = Carried::straight();

        for hosted in [
            Hosted::provider(&provider, None),
            Hosted::provider(&provider, None).taking(&straight),
        ] {
            assert_eq!(
                hosted.where_it_would_connect(),
                Some(("api.mistral.ai".to_owned(), 443))
            );
        }
    }

    /// The question reaches the address the person typed, over the convention
    /// `openai.rs` owns — which is asserted in full there, and here only
    /// as the join between this type and it.
    #[test]
    fn the_question_goes_to_the_address_the_person_typed() {
        let (url, server) = serving(AN_ANSWER, 200);
        let provider = mistral(&url);
        let key = alo_models::Secret::typed("sk-live-0123456789").unwrap();
        let answer = Hosted::provider(&provider, Some(&key)).ask(&question(), &{
            use std::net::ToSocketAddrs as _;
            Hosted::provider(&provider, Some(&key))
                .where_it_would_connect()
                .into_iter()
                .flat_map(|(host, port)| {
                    (host.as_str(), port)
                        .to_socket_addrs()
                        .into_iter()
                        .flatten()
                })
                .collect::<Vec<SocketAddr>>()
        });
        let request = server.join().unwrap();

        assert_eq!(answer.unwrap(), "The tenant may not sublet.");
        assert!(
            request.starts_with("POST /v1/chat/completions "),
            "{request}"
        );
        assert!(
            request
                .to_lowercase()
                .contains("authorization: bearer sk-live-0123456789\r\n"),
            "{request}"
        );
    }

    /// **This machine waits longer for itself than for anybody else** (ADR
    /// 0007). A constant either door could have shared would have made that a
    /// coincidence rather than a decision, so the two are asserted against each
    /// other.
    #[test]
    fn a_provider_is_waited_on_for_less_time_than_this_machine_is() {
        assert!(WHILE_A_MODEL_THINKS < crate::served::WHILE_THIS_MACHINE_THINKS);
    }
}
