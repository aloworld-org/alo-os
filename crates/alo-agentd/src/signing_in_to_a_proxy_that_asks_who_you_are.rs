//! A proxy that asks who this machine is, signed in to on the road a turn's
//! question takes.
//!
//! The acceptance for `docs/autonomy/v0-5-software-and-the-web-plan.md` task 12
//! on this crate's road, built on
//! [ADR 0059](../../../docs/decisions/0059-where-a-machine-wide-proxy-password-is-kept.md).
//! Every step is the production one: the `[proxy]` section — `sign-in-as` and
//! `password-in-keyring` included — is read by the same `crate::describing::read`
//! a running service uses, the password is read out of the machine's own
//! credentials by `alo_proxy::TheMachinesPasswords`, the credential is put on
//! the road by `alo_proxy::signed_in`, and the question crosses as a line an
//! agent really sends.
//!
//! # What it watches, and why that is a socket
//!
//! `crate::the_proxy_on_the_road_a_question_takes` says it in full and the
//! reason is the same: a test that read a configured proxy back off a value
//! would pass on a machine that configured it and then connected somewhere
//! else. So the assertion is **what arrived on the wire** — the first line of
//! the `CONNECT` conversation, and the `Proxy-Authorization` header beside it,
//! which is this machine saying who it is to the company's proxy.
//!
//! # And the three refusals, which are the half that matters most
//!
//! A machine that cannot sign in **refuses the question**. It does not knock on
//! the proxy's door as somebody with no password — the proxy would refuse it,
//! and on a network with no other route out that is a person's whole afternoon
//! — and it does not go around the proxy either, which would be this machine
//! sending a company's traffic past the company's own rule with nobody told.
//! Each refusal is checked at the proxy's door, at the provider's door, and in
//! the record.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported — and \
              the door nobody came to is reported with the sentence the agent was given instead, \
              which is the one thing that says why"
)]

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;

use alo_models::Catalogue;
use alo_proxy::{NotSignedIn, TheMachinesPasswords};
use alo_record::{Asking, Only, Record};
use alo_strings::Strings;

use crate::described::Described;
use crate::describing::read;
use crate::doing::what_an_agent_said;
use crate::questions::{Questions, TheBound, WhoseKeyring};
use crate::testing::{
    a_directory_of_our_own, a_listener_that_reports_what_it_was_told, a_message, hour, in_english,
    noon, on_a_machine_that_answers,
};
use crate::the_road_out::TheRoadOut;
use crate::trusting::WhoDescribedIt;

/// An agent's question in words, as it really arrives on the socket.
const ASKED: &str = r#"{"ask":{"question":"how many invoices are unpaid?"}}"#;

/// What the person called the provider they chose.
const THE_PROVIDER: &str = "Mine";

/// The name the company's proxy asks this machine to sign in as.
const SIGNING_IN_AS: &str = "anna";

/// The name the machine's own password for that proxy is kept under.
const KEPT_UNDER: &str = "the company proxy";

/// The password itself, as a machine would have been given it.
///
/// Letters and digits on purpose: what goes on the wire is then exactly what is
/// written here, with nothing in the middle rewriting it, so a failure names
/// the thing that is wrong rather than an encoding.
const THE_PASSWORD: &str = "hunter2";

/// A description of an ordinary machine in the newest shape, with whatever a
/// test writes after it.
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

/// That description, read as the organisation's configuration system wrote it.
fn written_by_an_administrator(said: &str) -> Described {
    read(
        said,
        Path::new("/etc/alo/agentd.toml"),
        WhoDescribedIt::AnAdministrator,
    )
    .unwrap()
}

/// A `[proxy]` section sending every road out through a proxy at `at` that
/// asks who this machine is.
fn a_proxy_that_asks_who_you_are(at: SocketAddr) -> String {
    format!(
        "\n[proxy]\n\
         goes-through = \"an-address\"\n\
         http = \"http://{at}\"\n\
         https = \"http://{at}\"\n\
         sign-in-as = \"{SIGNING_IN_AS}\"\n\
         password-in-keyring = \"{KEPT_UNDER}\"\n"
    )
}

/// Settings in which this person chose one provider, at this address.
///
/// `needs-a-key = false`: what is being measured is how this machine signs in
/// to the **proxy**, and a provider that wanted a key of its own would put a
/// keyring between the question and the socket for reasons that have nothing to
/// do with it.
fn a_person_who_chose(called: &str, provider: SocketAddr) -> PathBuf {
    let config = a_directory_of_our_own(called);
    let folder = config.join(alo_choosing::THE_FOLDER);
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(
        folder.join(alo_choosing::THE_SETTINGS),
        format!(
            "format = 2

[answers]
provider = {{ name = \"{THE_PROVIDER}\", model = \"a-model\" }}

[[provider]]
name = \"{THE_PROVIDER}\"
endpoint = \"https://{provider}\"
needs-a-key = false
"
        ),
    )
    .unwrap();
    config
}

/// The credentials a machine gave this unit, with the proxy's password in them.
fn a_machine_given_the_password(called: &str) -> TheMachinesPasswords {
    let directory = a_directory_of_our_own(called);
    let at = directory.join(KEPT_UNDER);
    std::fs::write(&at, THE_PASSWORD).unwrap();
    kept_the_way_a_machine_keeps_one(&at);
    TheMachinesPasswords::at(&directory)
}

/// The credentials of a unit that was given none, which is every machine whose
/// unit file does not carry the line ADR 0059 names.
fn a_machine_given_nothing(called: &str) -> TheMachinesPasswords {
    let nowhere = a_directory_of_our_own(called).join("never-made");
    TheMachinesPasswords::at(&nowhere)
}

/// The permissions systemd gives a credential, so that a test reads what a
/// machine would.
#[cfg(unix)]
fn kept_the_way_a_machine_keeps_one(at: &Path) {
    use std::os::unix::fs::PermissionsExt as _;

    std::fs::set_permissions(at, std::fs::Permissions::from_mode(0o400)).unwrap();
}

/// Nothing to do where a mode is not what says it.
#[cfg(not(unix))]
fn kept_the_way_a_machine_keeps_one(_at: &Path) {}

/// What a question left behind: the sentence the agent was given, and the
/// record of what left the machine.
struct WhatHappened {
    /// What crossed back to the agent, as text.
    said: String,
    /// What was written down, so the indicator's own line can be read off it.
    record: Record,
}

impl WhatHappened {
    /// The line law 1 showed, out of the one egress entry this question made.
    fn the_line_the_indicator_showed(&self, strings: &Strings) -> String {
        let egress = Asking::anything().only(Only::Egress);
        let mut left = self
            .record
            .answering(&egress)
            .filter_map(|entry| entry.happened().destination().map(|to| to.shown(strings)));
        let line = left
            .next()
            .expect("nothing was written down as having left");
        assert!(left.next().is_none(), "one question left more than once");
        line
    }

    /// How many times this question was written down as having left.
    fn how_often_something_left(&self) -> usize {
        self.record
            .answering(&Asking::anything().only(Only::Egress))
            .count()
    }
}

/// Put one question on a machine described like this, whose own credentials are
/// these, and say what happened.
fn a_question_on_a_machine(
    called: &str,
    section: &str,
    provider: SocketAddr,
    signing_in: TheMachinesPasswords,
) -> WhatHappened {
    let described = written_by_an_administrator(&a_description_with(section));
    let config = a_person_who_chose(called, provider);
    let mut questions = Questions::of_a_session(
        Some(config.into_os_string()),
        None,
        Catalogue::built_in().unwrap(),
        TheBound::Nobodys,
        WhoseKeyring::Nobodys,
    )
    .through(TheRoadOut::signing_in_at(
        described.proxy().cloned(),
        signing_in,
    ));

    let mut record = Record::default();
    let said = on_a_machine_that_answers(&mut record, |turning, grants, strings| {
        what_an_agent_said(
            &a_message(ASKED),
            turning,
            &mut questions,
            None,
            grants,
            strings,
            hour(),
            noon(),
        )
    });
    let said = said
        .refusal()
        .map(|wording| wording.text().to_owned())
        .unwrap_or_default();
    WhatHappened { said, record }
}

/// **A machine whose proxy asks who it is signs in**, on the road a turn's
/// question takes — and the provider is never spoken to directly.
///
/// The first half of task 12's acceptance on this road, read off the wire: the
/// `CONNECT` line says this question is being carried to the provider, and the
/// header beside it says who is asking. `alo_proxy::Carried::as_an_address` is
/// the one function that turned the password into text, and
/// `alo_proxy::signed_in` is the one door it travelled through to get here.
#[test]
fn a_machine_whose_proxy_asks_who_it_is_signs_in_on_the_road_a_question_takes() {
    let (proxy, at_the_proxy) = a_listener_that_reports_what_it_was_told();
    let (provider, at_the_provider) = a_listener_that_reports_what_it_was_told();

    let happened = a_question_on_a_machine(
        "signing-in",
        &a_proxy_that_asks_who_you_are(proxy),
        provider,
        a_machine_given_the_password("signing-in-credentials"),
    );

    let said = what_was_said(at_the_proxy)
        .unwrap_or_else(|| panic!("nobody came to the proxy's door: {}", happened.said));
    let said = String::from_utf8_lossy(&said).into_owned();
    assert!(
        said.starts_with(&format!("CONNECT {provider} ")),
        "the proxy was not asked to carry this question to the provider: {said}"
    );
    assert!(
        said.contains(&format!(
            "Proxy-Authorization: Basic {}\r\n",
            signed_in_as()
        )),
        "this machine reached the company's proxy without saying who it is: {said}"
    );
    assert!(
        what_was_said(at_the_provider).is_none(),
        "the question went straight to the provider on a machine told to use a proxy"
    );
}

/// **A machine that was never given the password refuses the question**, in
/// `alo-proxy`'s own words — and knocks on nobody's door.
///
/// The refusal this whole task is about. Reaching the proxy without the
/// credential it asked for would be refused by the proxy, in somebody else's
/// words, at the far end of a connection that should never have been opened;
/// going straight out instead would be worse still.
#[test]
fn a_machine_that_was_never_given_the_password_refuses_the_question() {
    let (proxy, at_the_proxy) = a_listener_that_reports_what_it_was_told();
    let (provider, at_the_provider) = a_listener_that_reports_what_it_was_told();

    let happened = a_question_on_a_machine(
        "given-nothing",
        &a_proxy_that_asks_who_you_are(proxy),
        provider,
        a_machine_given_nothing("given-nothing-credentials"),
    );

    let strings = in_english();
    assert_eq!(
        happened.said,
        NotSignedIn::NothingKeepsIt.said(&strings).text(),
        "the question was not refused in the words alo-proxy already has"
    );
    assert!(
        happened.said.contains("nothing was sent"),
        "the person is not told the question did not quietly happen: {}",
        happened.said
    );
    assert!(
        what_was_said(at_the_proxy).is_none(),
        "this machine knocked on the company's proxy with no password to give it"
    );
    assert!(
        what_was_said(at_the_provider).is_none(),
        "this machine went round the company's proxy rather than through it"
    );
    assert_eq!(
        happened.how_often_something_left(),
        0,
        "something was written down as having left, and nothing did"
    );
}

/// **A password anybody on the machine could read refuses the question**, in a
/// sentence of its own, because setting it again is a different thing to do.
#[cfg(unix)]
#[test]
fn a_password_anybody_could_read_refuses_the_question_in_its_own_sentence() {
    use std::os::unix::fs::PermissionsExt as _;

    let (proxy, at_the_proxy) = a_listener_that_reports_what_it_was_told();
    let (provider, at_the_provider) = a_listener_that_reports_what_it_was_told();

    let directory = a_directory_of_our_own("readable-by-anybody-credentials");
    let at = directory.join(KEPT_UNDER);
    std::fs::write(&at, THE_PASSWORD).unwrap();
    std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o444)).unwrap();

    let happened = a_question_on_a_machine(
        "readable-by-anybody",
        &a_proxy_that_asks_who_you_are(proxy),
        provider,
        TheMachinesPasswords::at(&directory),
    );

    let strings = in_english();
    assert_eq!(
        happened.said,
        NotSignedIn::ReadableByAnybody.said(&strings).text()
    );
    assert_ne!(
        happened.said,
        NotSignedIn::NothingKeepsIt.said(&strings).text(),
        "a password left readable and a password never given read the same"
    );
    assert!(what_was_said(at_the_proxy).is_none());
    assert!(what_was_said(at_the_provider).is_none());
    assert_eq!(happened.how_often_something_left(), 0);
}

/// **Nothing a person reads and nothing written down carries the password** —
/// not the sentence the agent was given, not the line the indicator shows, and
/// not the record.
///
/// Checked on the road that **succeeds**, which is the one where a credential
/// exists to leak: the question really went through the proxy signed in, and
/// the password is in none of what that left behind.
#[test]
fn nothing_a_person_reads_and_nothing_written_down_carries_the_password() {
    let strings = in_english();
    let (proxy, _at_the_proxy) = a_listener_that_reports_what_it_was_told();
    let (provider, _at_the_provider) = a_listener_that_reports_what_it_was_told();

    let happened = a_question_on_a_machine(
        "nothing-carries-it",
        &a_proxy_that_asks_who_you_are(proxy),
        provider,
        a_machine_given_the_password("nothing-carries-it-credentials"),
    );

    let line = happened.the_line_the_indicator_showed(&strings);
    assert!(line.contains(THE_PROVIDER), "{line}");
    assert!(!line.contains(THE_PASSWORD), "the indicator's line: {line}");
    assert!(
        !line.contains(SIGNING_IN_AS),
        "the indicator's line names who this machine signed in as: {line}"
    );
    assert!(
        !line.contains(&proxy.port().to_string()),
        "the indicator names the proxy rather than the provider: {line}"
    );

    assert!(
        !happened.said.contains(THE_PASSWORD),
        "the sentence the agent was given: {}",
        happened.said
    );
    assert!(
        !format!("{:?}", happened.record).contains(THE_PASSWORD),
        "the record carries the password"
    );
}

/// What this machine says on the wire when it signs in, as the proxy reads it.
///
/// Worked out here rather than written down, so that a change to how a
/// credential is put on the wire fails as a difference on the socket rather
/// than as a constant nobody updated.
fn signed_in_as() -> String {
    let said = format!("{SIGNING_IN_AS}:{THE_PASSWORD}");
    let letters = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut written = String::new();
    for lump in said.as_bytes().chunks(3) {
        let byte = |place: usize| u32::from(lump.get(place).copied().unwrap_or_default());
        let held = byte(0) << 16 | byte(1) << 8 | byte(2);
        for place in 0..4 {
            if place <= lump.len() {
                let at = usize::try_from((held >> (18 - 6 * place)) & 0b11_1111).unwrap();
                written.push(char::from(letters.get(at).copied().unwrap_or(b'=')));
            } else {
                written.push('=');
            }
        }
    }
    written
}

/// What was said at that door, waiting for the listener's own answer —
/// [`None`] where nobody came.
fn what_was_said(listener: JoinHandle<Option<Vec<u8>>>) -> Option<Vec<u8>> {
    listener.join().unwrap()
}
