//! A person changing the proxy on a machine whose root-owned description sets
//! one, refused in the words naming the organisation — beside the same change
//! on a machine whose description names no proxy at all.
//!
//! The acceptance for reading `[proxy]`, and every step of it is the production
//! one: the description is read by the same `crate::describing::read` that
//! `Described::at` uses, as a file root wrote (which is what makes a proxy an
//! organisation's, and which a test process that is not root cannot put on a
//! disk — `tests/what_a_machine_says_about_itself.rs` reads the section off a
//! real disk under whichever owner the suite runs as); the value it produces is
//! `alo_proxy::Kept` exactly as it comes back; the change a person makes is
//! `alo_proxy::Kept::changed_by_the_person`, which is the one door a settings
//! panel has; and the refusal is rendered in this machine's whole vocabulary,
//! the one `crate::starting` loads.
//!
//! And because a proxy that is read and not honoured would be a setting doing
//! nothing, the setting that comes off the description is put to
//! `alo_proxy::the_way` on every road out of this machine — which is what says
//! the lines somebody typed reach the roads they were typed for.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_proxy::{
    ConfigurationAddress, Kept, NotChanged, NotEvaluated, ProxyAddress, Reaching, Road, Scheme,
    SetBy, SpokenTo, TheEvaluator, TheProxy, Way,
};

use crate::described::Described;
use crate::describing::read;
use crate::trusting::WhoDescribedIt;

/// A machine with nothing to work an automatic configuration out with.
///
/// The ordinary case for the two shapes under test here, and what makes the
/// third refuse rather than go straight out — which is `alo-proxy`'s decision
/// and is only exercised here.
struct NothingEvaluatesIt;

impl TheEvaluator for NothingEvaluatesIt {
    fn asked(&self, _: &ConfigurationAddress, _: &str, _: &str) -> Result<String, NotEvaluated> {
        Err(NotEvaluated::NothingEvaluatesIt)
    }
}

/// A description of an ordinary machine in the newest shape, with whatever else
/// is written after it.
fn a_description_with(after: &str) -> String {
    format!(
        r#"format = 4

[logins]
person = 1000
agent = 989
group = 989

[agent]
name = "alo"
turn-seconds = 900
proposal-seconds = 300

[record]
path = "/var/lib/alo/record"
keeping = "forever"
{after}"#
    )
}

/// The `[proxy]` section a company hands somebody on a slip of paper.
const THE_COMPANY_PROXY: &str = "\n[proxy]\n\
     goes-through = \"an-address\"\n\
     http = \"http://proxy.example.com:8080\"\n\
     https = \"http://proxy.example.com:8080\"\n\
     except = [\"intranet.example.com\"]\n";

/// That description, read as root wrote it.
fn written_by_an_administrator(said: &str) -> Described {
    read(
        said,
        Path::new("/etc/alo/agentd.toml"),
        WhoDescribedIt::AnAdministrator,
    )
    .unwrap()
}

/// The same text, read as the person whose machine it is wrote it.
fn written_by_the_person(said: &str) -> Described {
    read(
        said,
        Path::new("/etc/alo/agentd.toml"),
        WhoDescribedIt::ThePerson,
    )
    .unwrap()
}

/// **A person is refused a change on a machine whose organisation set the
/// proxy, in the words naming who set it — and the setting is unchanged** —
/// beside the same change on a machine whose description names no proxy, where
/// there is nobody to refuse them.
#[test]
fn a_persons_change_is_refused_on_a_machine_whose_description_sets_the_proxy() {
    let loaded = crate::starting::what_this_machine_says().unwrap();
    let strings = loaded.strings();

    // The organisation's machine: the proxy came off the description, and it is
    // theirs because root owns the file.
    let managed = written_by_an_administrator(&a_description_with(THE_COMPANY_PROXY));
    let kept = managed.proxy().unwrap();
    assert_eq!(kept.set_by(), SetBy::AnOrganisation);
    assert_eq!(
        kept.proxy().for_(Scheme::Https).unwrap().written(),
        "http://proxy.example.com:8080"
    );

    let refused = kept
        .clone()
        .changed_by_the_person(TheProxy::None)
        .unwrap_err();
    assert_eq!(refused, NotChanged::AnOrganisationSetIt);
    let said = refused.said(strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("organisation"), "{said}");
    assert!(said.unfilled().is_empty(), "{said}");
    // And what the machine holds is still what the description said: a refused
    // change is refused rather than half-applied.
    assert!(
        managed
            .proxy()
            .unwrap()
            .proxy()
            .exceptions()
            .unwrap()
            .let_through(&Reaching::over(Scheme::Https, "intranet.example.com").unwrap()),
        "a refused change altered the machine's proxy"
    );

    // The same machine with no section: nobody set a proxy, so nobody refuses
    // the person — and *nobody set one* is not *straight out*.
    let unmanaged = written_by_an_administrator(&a_description_with(""));
    assert_eq!(unmanaged.proxy(), None);
    let theirs = Kept::by_this_person(TheProxy::one(
        ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).unwrap(),
    ));
    assert_eq!(theirs.set_by(), SetBy::ThisPerson);
    let changed = theirs.changed_by_the_person(TheProxy::None).unwrap();
    assert_eq!(changed.proxy(), &TheProxy::None);
    assert_eq!(changed.set_by(), SetBy::ThisPerson);
}

/// **The identical section in the person's own description is theirs**, and
/// they may change it: a restrictive value is never, on its own, evidence that
/// somebody else set it.
#[test]
fn the_same_section_in_the_persons_own_description_is_their_own() {
    let theirs = written_by_the_person(&a_description_with(THE_COMPANY_PROXY));
    let kept = theirs.proxy().unwrap().clone();
    assert_eq!(kept.set_by(), SetBy::ThisPerson);
    assert_eq!(
        kept.changed_by_the_person(TheProxy::None).unwrap().proxy(),
        &TheProxy::None
    );
}

/// **The proxy the description states reaches every road out of this machine**,
/// and the place it excepts still goes straight out. A setting read and not
/// honoured would be a line in `/etc` doing nothing.
#[test]
fn the_proxy_the_description_states_reaches_every_road_out() {
    let managed = written_by_an_administrator(&a_description_with(THE_COMPANY_PROXY));
    let proxy = managed.proxy().unwrap().proxy();

    let out = Reaching::over(Scheme::Https, "dl.example.org").unwrap();
    for road in Road::EVERY {
        let way = alo_proxy::the_way(proxy, road, &out, &NothingEvaluatesIt).unwrap();
        assert!(
            matches!(way, Way::Through(ref address) if address.written() == "http://proxy.example.com:8080"),
            "{road:?}: {way:?}"
        );
    }

    let excepted = Reaching::over(Scheme::Https, "files.intranet.example.com").unwrap();
    assert!(
        alo_proxy::the_way(proxy, Road::AskingAProvider, &excepted, &NothingEvaluatesIt)
            .unwrap()
            .is_straight()
    );
}

/// **A machine told to go straight out is told so**, and a machine told to ask
/// the network refuses every road it cannot work out — never going around the
/// rule. Both come off a description and neither is this file's decision.
#[test]
fn the_other_two_shapes_come_off_the_description_and_are_honoured() {
    let out = Reaching::over(Scheme::Https, "dl.example.org").unwrap();

    let straight = written_by_an_administrator(&a_description_with(
        "\n[proxy]\ngoes-through = \"nothing\"\n",
    ));
    assert_eq!(straight.proxy().unwrap().proxy(), &TheProxy::None);
    assert!(
        alo_proxy::the_way(
            straight.proxy().unwrap().proxy(),
            Road::CheckingForAnUpdate,
            &out,
            &NothingEvaluatesIt,
        )
        .unwrap()
        .is_straight()
    );

    let asking = written_by_an_administrator(&a_description_with(
        "\n[proxy]\n\
         goes-through = \"a-configuration\"\n\
         configuration = \"http://wpad.example.com/wpad.dat\"\n",
    ));
    assert!(asking.proxy().unwrap().proxy().configuration().is_some());
    assert!(
        alo_proxy::the_way(
            asking.proxy().unwrap().proxy(),
            Road::CheckingForAnUpdate,
            &out,
            &NothingEvaluatesIt,
        )
        .is_err(),
        "a configuration nothing could work out was taken straight out"
    );
}

/// **A password written into the description stops the service, in a sentence
/// that names the key and never repeats the password.**
#[test]
fn a_password_in_the_description_stops_the_service_and_is_never_repeated() {
    let refused = read(
        &a_description_with(
            "\n[proxy]\n\
             goes-through = \"an-address\"\n\
             http = \"http://proxy.example.com:8080\"\n\
             sign-in-as = \"anna\"\n\
             password = \"hunter2\"\n",
        ),
        Path::new("/etc/alo/agentd.toml"),
        WhoDescribedIt::AnAdministrator,
    )
    .unwrap_err();
    assert!(
        matches!(
            refused,
            crate::refusing::NotDescribed::AProxyPasswordInTheFile { .. }
        ),
        "{refused:?}"
    );
    assert!(!refused.to_string().contains("hunter2"), "{refused}");
    assert!(
        refused.to_string().contains("proxy.password-in-keyring"),
        "{refused}"
    );
}
