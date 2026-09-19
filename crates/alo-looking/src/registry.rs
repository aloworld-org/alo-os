//! The real place, asked over the network — and the only file in this crate
//! that knows an address is a URL.
//!
//! Two questions and no third ([`crate::ThePlace`]): which names the place
//! holds, and which build one of them is. Both are answered by the place's
//! own distribution interface, which is the same interface the base uses when
//! a person actually applies an update — so *where it was checked* and *where
//! it was fetched* are one address, taken from `image/pinned.toml`.
//!
//! # What this fetches, and what it never fetches
//!
//! It fetches a list of names and one header. **There is no request in this
//! file for the bytes of a build**, no path here contains `blobs`, and nothing
//! in this crate writes what a place sent it to a disk. The build is pulled by
//! the base, later, only when a person chooses to apply the update
//! (`alo_updating::apply`) — a check that downloaded it to be helpful would be
//! exactly the traffic law 1 exists to make visible, arriving before anybody
//! agreed to it.
//!
//! The second question is a `HEAD`, which is the distribution interface's own
//! way of asking *which build is this* without being sent the description of
//! it. What comes back that this machine reads is one header.
//!
//! # The machine's proxy is on this road
//!
//! On a great many company networks there is no other route out, so a check
//! that ignored the machine's proxy would be a machine that can never find out
//! there is an update. The way out is decided by `alo_proxy::the_way` for
//! `alo_proxy::Road::CheckingForAnUpdate` and handed in
//! ([`TheRegistry::taking`]) — **said rather than left unsaid**, so that the
//! environment this process happens to be running in cannot point alo OS's own
//! road somewhere nobody chose. A road going straight out is
//! `alo_proxy::Carried::straight`, which is what [`TheRegistry::at`] is.
//!
//! # Asked anonymously, and never with a credential of the person's
//!
//! A place that wants to be asked for a token is asked for one, with the scope
//! it named itself, and nothing else is sent. This machine has no account at
//! the place its updates come from and does not acquire one to ask a question:
//! a check is not a sign-in, and the only errand `alo_egress` has for signing
//! in is a different one. A place that refuses an anonymous question is
//! [`NoAnswer::ItRefused`], which is a sentence rather than a prompt.

use std::time::Duration;

use alo_proxy::Carried;
use serde::Deserialize;

use crate::asking::ThePlace;
use crate::place::Place;
use crate::refusing::NoAnswer;
use crate::release::Release;

/// How long this machine waits for the place to answer.
///
/// A check is a question somebody is waiting on, or one the machine asks on
/// its way up; neither is worth holding a machine open for minutes.
pub const WHILE_SOMEBODY_WAITS: Duration = Duration::from_secs(20);

/// The most of an answer this machine reads.
///
/// A list of names and a token are both small. A place answering with
/// something enormous is a place answering with something that is not this.
const MOST_OF_AN_ANSWER: u64 = 256 * 1024;

/// How many names this machine asks for at once.
///
/// One page. A place holding more releases than this would need its answer read
/// across several, which nothing needs yet and which would be its own change.
const HOW_MANY_NAMES: usize = 1000;

/// What this machine will accept as a description of a build.
const WHAT_AN_ANSWER_MAY_BE: &str = "application/vnd.oci.image.index.v1+json, \
                                     application/vnd.oci.image.manifest.v1+json, \
                                     application/vnd.docker.distribution.manifest.list.v2+json, \
                                     application/vnd.docker.distribution.manifest.v2+json";

/// The header the place names the build in.
const THE_BUILD: &str = "docker-content-digest";

/// The header a place asks to be asked for a token with.
const HOW_TO_ASK: &str = "www-authenticate";

/// The only way a token is fetched.
const OVER_TLS: &str = "https://";

/// A token, as a place hands one back.
///
/// Two spellings, because places disagree about which; either is the same
/// thing. Deliberately nothing else: what a place says beside it is not read.
#[derive(Debug, Deserialize)]
struct AToken {
    /// What most places call it.
    #[serde(default)]
    token: Option<String>,
    /// What some call it instead.
    #[serde(default)]
    access_token: Option<String>,
}

/// The place this machine's updates come from, asked over the network.
///
/// Deliberately not `Clone` and not `PartialEq`: it holds the road out it was
/// given, and that can hold a credential — `alo_proxy::Carried` says why a
/// credential that can be copied or compared is a credential somewhere nobody
/// meant it to be. Its `Debug` is safe because that type's own is.
#[derive(Debug)]
pub struct TheRegistry<'a> {
    /// Where this machine asks.
    place: &'a Place,
    /// The way out this machine decided for this road.
    taking: Carried,
}

impl<'a> TheRegistry<'a> {
    /// This place, going straight out.
    #[must_use]
    pub fn at(place: &'a Place) -> Self {
        Self {
            place,
            taking: Carried::straight(),
        }
    }

    /// The same place, taking the way out this machine decided for this road.
    #[must_use]
    pub fn taking(mut self, taking: Carried) -> Self {
        self.taking = taking;
        self
    }

    /// Where this machine asks.
    #[must_use]
    pub fn place(&self) -> &Place {
        self.place
    }

    /// The way out it is taking.
    #[must_use]
    pub fn through(&self) -> &Carried {
        &self.taking
    }

    /// Ask, once, with the token where there is one.
    fn once(
        &self,
        url: &str,
        heading: bool,
        accept: &str,
        token: Option<&str>,
    ) -> Result<ureq::http::Response<ureq::Body>, NoAnswer> {
        let through = self
            .taking
            .for_a_request()
            .map_err(|_| NoAnswer::NoWayOut)?;
        let asking = if heading {
            ureq::head(url)
        } else {
            ureq::get(url)
        };
        let asking = asking
            .header("accept", accept)
            .config()
            .timeout_global(Some(WHILE_SOMEBODY_WAITS))
            // Refused rather than followed: a place that answers a question
            // about updates by sending this machine somewhere else is not
            // answering it.
            .max_redirects(0)
            // Every answer comes back to be read here, because *it refused*
            // and *nothing came back* are different things to tell somebody.
            .http_status_as_error(false)
            // Said rather than left unsaid — see this file's header.
            .proxy(through)
            .build();
        let asking = match token {
            Some(token) => asking.header("authorization", format!("Bearer {token}")),
            None => asking,
        };
        asking.call().map_err(|why| why_it_did_not_answer(&why))
    }

    /// Ask, and ask again with a token if that is what the place wants.
    fn answered(
        &self,
        url: &str,
        heading: bool,
        accept: &str,
    ) -> Result<ureq::http::Response<ureq::Body>, NoAnswer> {
        let first = self.once(url, heading, accept, None)?;
        if first.status() != ureq::http::StatusCode::UNAUTHORIZED {
            return Ok(first);
        }
        let asked_for = first
            .headers()
            .get(HOW_TO_ASK)
            .and_then(|said| said.to_str().ok())
            .ok_or(NoAnswer::ItRefused)?
            .to_owned();
        let token = self.a_token(&asked_for)?;
        let again = self.once(url, heading, accept, Some(&token))?;
        if again.status() == ureq::http::StatusCode::UNAUTHORIZED {
            return Err(NoAnswer::ItRefused);
        }
        Ok(again)
    }

    /// A token for the question this machine is asking, from the place that
    /// asked to be asked for one.
    fn a_token(&self, asked_for: &str) -> Result<String, NoAnswer> {
        let realm = named_in(asked_for, "realm").ok_or(NoAnswer::NotUnderstood)?;
        // A place may name any realm it likes and this machine will not follow
        // it off an encrypted road: a token is a thing that is sent onwards,
        // and one fetched in clear is one somebody on the way has.
        if !realm.starts_with(OVER_TLS) {
            return Err(NoAnswer::NotUnderstood);
        }
        let scope = named_in(asked_for, "scope")
            .unwrap_or_else(|| format!("repository:{}:pull", self.place.repository()));
        let service =
            named_in(asked_for, "service").unwrap_or_else(|| self.place.host().to_owned());
        let asking = format!(
            "{realm}?service={}&scope={}",
            written_for_a_query(&service),
            written_for_a_query(&scope)
        );
        let mut answered = self.once(&asking, false, "application/json", None)?;
        if answered.status() != ureq::http::StatusCode::OK {
            return Err(NoAnswer::ItRefused);
        }
        let said = answered
            .body_mut()
            .with_config()
            .limit(MOST_OF_AN_ANSWER)
            .read_to_string()
            .map_err(|_| NoAnswer::NotUnderstood)?;
        let token: AToken = serde_json::from_str(&said).map_err(|_| NoAnswer::NotUnderstood)?;
        token
            .token
            .or(token.access_token)
            .filter(|token| !token.is_empty())
            .ok_or(NoAnswer::NotUnderstood)
    }
}

/// Every name a place holds, as it answers with them.
#[derive(Debug, Deserialize)]
struct Held {
    /// The names. Absent on a place that holds none.
    #[serde(default)]
    tags: Option<Vec<String>>,
}

impl ThePlace for TheRegistry<'_> {
    fn every_name(&self) -> Result<Vec<String>, NoAnswer> {
        let asking = format!(
            "{OVER_TLS}{}/v2/{}/tags/list?n={HOW_MANY_NAMES}",
            self.place.host(),
            self.place.repository()
        );
        let mut answered = self.answered(&asking, false, "application/json")?;
        if answered.status() != ureq::http::StatusCode::OK {
            return Err(NoAnswer::ItRefused);
        }
        let said = answered
            .body_mut()
            .with_config()
            .limit(MOST_OF_AN_ANSWER)
            .read_to_string()
            .map_err(|_| NoAnswer::NotUnderstood)?;
        let held: Held = serde_json::from_str(&said).map_err(|_| NoAnswer::NotUnderstood)?;
        Ok(held.tags.unwrap_or_default())
    }

    fn the_build_of(&self, release: &Release) -> Result<String, NoAnswer> {
        let asking = format!(
            "{OVER_TLS}{}/v2/{}/manifests/{}",
            self.place.host(),
            self.place.repository(),
            release.named_as()
        );
        let answered = self.answered(&asking, true, WHAT_AN_ANSWER_MAY_BE)?;
        if answered.status() != ureq::http::StatusCode::OK {
            return Err(NoAnswer::ItRefused);
        }
        answered
            .headers()
            .get(THE_BUILD)
            .and_then(|named| named.to_str().ok())
            .map(str::to_owned)
            .ok_or(NoAnswer::NotUnderstood)
    }
}

/// Which of the five refusals a client's failure is.
///
/// Told apart by what a person can do about it: nothing was reached at all,
/// something was reached and said nothing, something answered and refused, or
/// something answered with what this machine could not read.
fn why_it_did_not_answer(why: &ureq::Error) -> NoAnswer {
    match why {
        ureq::Error::Timeout(_) => NoAnswer::NothingCameBack,
        ureq::Error::HostNotFound | ureq::Error::ConnectionFailed | ureq::Error::Io(_) => {
            NoAnswer::NoWayOut
        }
        ureq::Error::StatusCode(_)
        | ureq::Error::TooManyRedirects
        | ureq::Error::RedirectFailed => NoAnswer::ItRefused,
        _ => NoAnswer::NotUnderstood,
    }
}

/// One `name="value"` out of the line a place asks to be asked with.
fn named_in(said: &str, name: &str) -> Option<String> {
    said.split(',')
        .filter_map(|part| part.trim().split_once('='))
        .find(|(key, _)| key.trim().rsplit(' ').next() == Some(name))
        .map(|(_, value)| value.trim().trim_matches('"').to_owned())
}

/// Some text, written so that it is one value of a query and not two.
///
/// Every character a place's own scope is written with goes through
/// unchanged; anything else — which cannot come from this machine, only from
/// what the place said — is written as its bytes.
fn written_for_a_query(text: &str) -> String {
    let mut written = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b':' | b'/') {
            written.push(char::from(byte));
        } else {
            written.push_str(&format!("%{byte:02X}"));
        }
    }
    written
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::the_place;

    /// The place is the pin's, and the road out is straight until somebody
    /// says otherwise.
    #[test]
    fn a_place_asked_straight_out_is_asked_straight_out() {
        let place = the_place();
        let asking = TheRegistry::at(&place);
        assert!(asking.through().is_straight());
        assert_eq!(asking.place(), &place);
    }

    /// **A proxy is taken when the machine decided one**, which on a great
    /// many company networks is the only road out there is.
    #[test]
    fn a_place_asked_through_a_proxy_takes_it() {
        let place = the_place();
        let proxy =
            alo_proxy::ProxyAddress::checked(alo_proxy::SpokenTo::Http, "proxy.example", 8080)
                .unwrap();
        let asking = TheRegistry::at(&place).taking(Carried::through(proxy.clone()));
        assert!(!asking.through().is_straight());
        assert_eq!(asking.through().way().through(), Some(&proxy));
    }

    /// The line a place asks to be asked with is read for the three things
    /// this machine needs, and a realm in clear is refused.
    #[test]
    fn what_a_place_asks_to_be_asked_with_is_read() {
        let said = "Bearer realm=\"https://place.example/token\",service=\"place.example\",\
                    scope=\"repository:somebody/alo-os:pull\"";
        assert_eq!(
            named_in(said, "realm").unwrap(),
            "https://place.example/token"
        );
        assert_eq!(named_in(said, "service").unwrap(), "place.example");
        assert_eq!(
            named_in(said, "scope").unwrap(),
            "repository:somebody/alo-os:pull"
        );
        assert_eq!(named_in(said, "nothing"), None);
        assert_eq!(named_in("Basic", "realm"), None);
    }

    /// **A place asking for a token in clear is refused**, because a token is
    /// a thing that is sent onwards and one fetched in clear is one somebody
    /// on the way has.
    #[test]
    fn a_token_asked_for_over_an_unencrypted_road_is_refused() {
        let place = the_place();
        let refused = TheRegistry::at(&place)
            .a_token("Bearer realm=\"http://place.example/token\",service=\"place.example\"")
            .unwrap_err();
        assert_eq!(refused, NoAnswer::NotUnderstood);

        let refused = TheRegistry::at(&place)
            .a_token("Basic realm=\"nowhere\"")
            .unwrap_err();
        assert_eq!(refused, NoAnswer::NotUnderstood);
    }

    /// A scope a place named is written as one value of a query, whatever is
    /// in it.
    #[test]
    fn a_scope_is_written_as_one_value() {
        assert_eq!(
            written_for_a_query("repository:somebody/alo-os:pull"),
            "repository:somebody/alo-os:pull"
        );
        assert_eq!(written_for_a_query("a b&c=d"), "a%20b%26c%3Dd");
    }
}
