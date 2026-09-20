//! The proxy a machine was told about, carried to the question a turn puts.
//!
//! The acceptance for
//! `docs/autonomy/v0-5-software-and-the-web-plan.md` task 11, and every step of
//! it is the production one: the `[proxy]` section is read by the same
//! `crate::describing::read` a running service uses, the value it produces is
//! the `alo_proxy::Kept` that `crate::described::Described::proxy` answers with,
//! the way out is decided by `alo_proxy::the_way` through
//! [`crate::the_road_out`], the question crosses as a line an agent really
//! sends, and what carries it is `alo_asking`'s own provider door.
//!
//! # What it watches, and why that is a socket rather than a value
//!
//! Two listeners of this test's own — one standing for the company's proxy and
//! one for the provider — and **which of them a question actually connects to**
//! is the assertion. A test that read a configured proxy back off a value would
//! pass on a machine that configured it and then connected somewhere else, and
//! that is precisely the failure this task exists to fix: on a company network,
//! a setting that is read and not taken is worse than no setting at all.
//!
//! Neither listener speaks TLS or the `CONNECT` conversation, so no answer
//! comes back and the question is refused. That is expected and is not what is
//! being measured. What is measured is which door the machine knocked on, and
//! which it did not.
//!
//! # The environment is the other half, and it takes a second process
//!
//! `ureq::Config::default` fills its proxy in from `ALL_PROXY`, `HTTPS_PROXY`
//! and `HTTP_PROXY` in whatever process it is running in. Until this task, this
//! service's provider road inherited exactly that, which is a road nobody chose
//! and nobody can be shown. Setting an environment variable inside a test is
//! `unsafe` under Rust 2024 and `CLAUDE.md` forbids `unsafe`, so
//! [`a_proxy_in_the_environment_points_this_machines_question_nowhere`] runs the
//! question in **a second copy of this test binary**, started with those
//! variables pointing at a listener the parent owns — and the parent then asks
//! that listener whether anybody came.

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
use std::process::Command;
use std::thread::JoinHandle;

use alo_choosing::{Chosen, Which};
use alo_models::{Catalogue, InferenceSource, Region};
use alo_protocol::ToAnAgent;
use alo_proxy::TheRentedEvaluator;
use alo_record::{Asking, Only, Record};
use alo_strings::Strings;

use crate::described::Described;
use crate::describing::read;
use crate::doing::what_an_agent_said;
use crate::questions::{Questions, TheBound, WhoseKeyring};
use crate::testing::{
    a_directory_of_our_own, a_listener_that_reports_what_it_was_told, a_message, a_runtime_saying,
    hour, in_english, noon, on_a_machine_that_answers,
};
use crate::the_road_out::TheRoadOut;
use crate::trusting::WhoDescribedIt;

/// An agent's question in words, as it really arrives on the socket.
const ASKED: &str = r#"{"ask":{"question":"how many invoices are unpaid?"}}"#;

/// What the person called the provider they chose.
const THE_PROVIDER: &str = "Mine";

/// A description of an ordinary machine in the newest shape, with whatever a
/// test writes after it.
///
/// The same shape `crate::changing_the_proxy_under_the_description` uses, so
/// the two acceptances read one machine rather than two.
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

/// A `[proxy]` section sending every road out through `at`.
fn a_section_naming(at: SocketAddr) -> String {
    format!(
        "\n[proxy]\n\
         goes-through = \"an-address\"\n\
         http = \"http://{at}\"\n\
         https = \"http://{at}\"\n"
    )
}

/// Settings in which this person chose one provider, at this address.
///
/// `needs-a-key = false`, which is `alo-choosing`'s way of writing *this
/// service takes no credential*. Deliberate: what is being measured here is
/// which address a road out opens a socket to, and a provider that wanted a key
/// would put a keyring between the question and that socket — tested already,
/// in `crate::doing`, and nothing to do with a proxy.
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
}

/// Put one question on a machine described like this, with its settings naming
/// this provider, and say what happened.
///
/// `evaluating` is where an automatic configuration would be worked out. Every
/// caller but one hands over the machine's real one, which is
/// [`TheRentedEvaluator::on_this_machine`]; the exception points it at a path
/// nothing is at, so that *a machine that cannot work one out* is the same
/// answer wherever these tests run.
fn a_question_on_a_machine(
    called: &str,
    section: &str,
    provider: SocketAddr,
    evaluating: TheRentedEvaluator,
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
    .through(TheRoadOut::evaluated_by(
        described.proxy().cloned(),
        evaluating,
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

/// The same, on the machine as it really is — nothing at the evaluator's path
/// on a developer's machine, and the image's program on an alo OS one.
fn a_question_on_a_real_machine(called: &str, section: &str, provider: SocketAddr) -> WhatHappened {
    a_question_on_a_machine(
        called,
        section,
        provider,
        TheRentedEvaluator::on_this_machine(),
    )
}

/// **A question on a machine whose description states a proxy is put to the
/// proxy, as a proxy** — and the provider is never spoken to directly.
///
/// The first half of task 11's acceptance, watched on a socket rather than read
/// off a value: *the proxy `alo_agentd::Described::proxy` answers with reaches
/// the road a turn's question takes.*
///
/// What is asserted is the **first line on the wire**, and that is not
/// fastidiousness. A road out of this machine is handed the addresses it may
/// use and resolves nothing (ADR 0020), so the address a socket opens to is the
/// one that was registered either way — and a test that watched addresses would
/// pass on a machine that connected to the proxy and then spoke to it as though
/// it were the provider. `CONNECT` is the difference.
#[test]
fn a_question_on_a_machine_told_about_a_proxy_is_put_to_the_proxy_as_a_proxy() {
    let (proxy, at_the_proxy) = a_listener_that_reports_what_it_was_told();
    let (provider, at_the_provider) = a_listener_that_reports_what_it_was_told();

    let happened =
        a_question_on_a_real_machine("told-about-a-proxy", &a_section_naming(proxy), provider);

    let said = what_was_said(at_the_proxy)
        .unwrap_or_else(|| panic!("nobody came to the proxy's door: {}", happened.said));
    assert!(
        asked_to_connect(&said, provider),
        "the proxy was not asked to carry this question to the provider: {}",
        String::from_utf8_lossy(&said)
    );
    assert!(
        what_was_said(at_the_provider).is_none(),
        "the question went straight to the provider on a machine told to use a proxy"
    );
}

/// **A question on a machine whose description states no proxy goes straight to
/// the provider**, and asks nobody to carry it.
#[test]
fn a_question_on_a_machine_told_of_no_proxy_goes_straight_to_the_provider() {
    let (_nobodys, at_nobodys) = a_listener_that_reports_what_it_was_told();
    let (provider, at_the_provider) = a_listener_that_reports_what_it_was_told();

    let happened = a_question_on_a_real_machine("told-of-no-proxy", "", provider);

    let said = what_was_said(at_the_provider).unwrap_or_else(|| {
        panic!(
            "the question did not reach the provider on a machine with no proxy: {}",
            happened.said
        )
    });
    assert!(
        began_a_conversation_of_its_own(&said),
        "the provider was addressed as though it were somebody's proxy: {}",
        String::from_utf8_lossy(&said)
    );
    assert!(
        what_was_said(at_nobodys).is_none(),
        "a question reached an address nobody's description named"
    );
}

/// **A machine told to ask an automatic configuration, and unable to, refuses
/// the question** — in the words `alo-proxy` already has, and with nothing
/// sent to the provider.
///
/// The refusal that matters on a network where the proxy is a rule rather than
/// a route: going straight out instead would be this machine sending a
/// company's traffic around the company's own rule with nobody told.
#[test]
fn a_configuration_this_machine_cannot_work_out_refuses_the_question() {
    let (provider, at_the_provider) = a_listener_that_reports_what_it_was_told();
    let happened = a_question_on_a_machine(
        "told-to-ask-a-configuration",
        "\n[proxy]\n\
         goes-through = \"a-configuration\"\n\
         configuration = \"http://wpad.example.com/proxy.config\"\n",
        provider,
        TheRentedEvaluator::at(Path::new("/nowhere/alo-agentd-has-no-evaluator-here")),
    );

    let strings = in_english();
    assert_eq!(
        happened.said,
        alo_proxy::NotOnTheRoad::CouldNotBeWorkedOut(alo_proxy::NotEvaluated::NothingEvaluatesIt)
            .said(&strings)
            .text(),
        "the question was not refused in the words alo-proxy already has"
    );
    assert!(
        what_was_said(at_the_provider).is_none(),
        "a question was sent to the provider by a machine that could not work out its way there"
    );
    assert_eq!(
        happened
            .record
            .answering(&Asking::anything().only(Only::Egress))
            .count(),
        0,
        "something was written down as having left, and nothing did"
    );
}

/// **The indicator names the provider, never the proxy** — held by reading the
/// line, and by finding it to be *the same line* the same question makes on a
/// machine with no proxy at all.
///
/// That equality is the whole claim. *It went to the proxy* is not what a person
/// needs to know: on a machine sold on sovereignty the indicator answers *where
/// did my work go*, and on a company network the answer is the same whether or
/// not a proxy is in the middle of it.
#[test]
fn the_line_the_indicator_shows_names_the_provider_whichever_way_the_road_went() {
    let strings = in_english();
    let (proxy, _at_the_proxy) = a_listener_that_reports_what_it_was_told();
    let (provider, _at_the_provider) = a_listener_that_reports_what_it_was_told();

    let through =
        a_question_on_a_real_machine("indicator-through", &a_section_naming(proxy), provider)
            .the_line_the_indicator_showed(&strings);
    let straight = a_question_on_a_real_machine("indicator-straight", "", provider)
        .the_line_the_indicator_showed(&strings);

    assert_eq!(
        through, straight,
        "the line depends on whether a proxy was in the middle of the road"
    );
    assert_eq!(
        through,
        alo_egress::Destination::of(&InferenceSource::Hosted {
            provider: THE_PROVIDER.to_owned(),
            region: Region::Unknown,
        })
        .unwrap()
        .shown(&strings),
        "the line is not the one the provider's own source renders"
    );
    assert!(through.contains(THE_PROVIDER), "{through}");
    assert!(
        !through.contains(&proxy.port().to_string()),
        "the line names the proxy: {through}"
    );
}

/// **A question answered by a runtime on this machine is never sent through a
/// proxy at all** — the answer comes back in the model's own words, and the
/// company's proxy is not knocked on.
///
/// Not a second rule beside `alo_proxy::the_way`'s: a question answered here
/// leaves nothing, so there is no road for a proxy to be asked about, and
/// `crate::doing` reads the road out only on the provider's arm. The same
/// answer from the other end is in `crate::the_road_out`, where a provider at
/// this machine's own address goes straight out whatever the description says.
#[test]
fn a_question_answered_by_a_runtime_on_this_machine_is_never_sent_through_the_proxy() {
    let (proxy, at_the_proxy) = a_listener_that_reports_what_it_was_told();
    let described = written_by_an_administrator(&a_description_with(&a_section_naming(proxy)));
    let mut questions = Questions::already_found(
        Chosen::of(Which::Brought, "my-finetune").unwrap(),
        a_runtime_saying(Ok("four are unpaid".to_owned())),
        TheBound::Nobodys,
    )
    .through(TheRoadOut::of(described.proxy().cloned()));

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

    assert!(
        matches!(&said, ToAnAgent::Answered { text, .. } if text.contains("four are unpaid")),
        "the question this machine answers itself was not answered: {said:?}"
    );
    assert!(
        what_was_said(at_the_proxy).is_none(),
        "a question answered on this machine was carried to the company's proxy"
    );
    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::Egress))
            .count(),
        0,
        "a question answered on this machine was written down as having left"
    );
}

/// **A proxy in this process's environment points this machine's question
/// nowhere.**
///
/// The road out is said on every request, including when it is straight, so a
/// variable in whatever process this service happens to be running in cannot
/// decide where somebody's question goes. Run in a second copy of this test
/// binary because setting an environment variable is `unsafe` under Rust 2024
/// and the workspace forbids `unsafe`; the child is
/// [`a_question_reaches_the_provider_under_whatever_the_environment_says`],
/// which is `#[ignore]`d so that the suite runs it only through this.
#[test]
fn a_proxy_in_the_environment_points_this_machines_question_nowhere() {
    let (environments, at_the_environments) = a_listener_that_reports_what_it_was_told();
    let pointed_at = format!("http://{environments}");

    let mut child = Command::new(std::env::current_exe().unwrap());
    for named in [
        "ALL_PROXY",
        "all_proxy",
        "HTTPS_PROXY",
        "https_proxy",
        "HTTP_PROXY",
        "http_proxy",
    ] {
        child.env(named, &pointed_at);
    }
    let ran = child
        .args([
            "--exact",
            "--ignored",
            "--nocapture",
            "the_proxy_on_the_road_a_question_takes::\
             a_question_reaches_the_provider_under_whatever_the_environment_says",
        ])
        .output()
        .unwrap();

    assert!(
        ran.status.success(),
        "the question did not reach the provider while the environment named somewhere else:\n{}\n{}",
        String::from_utf8_lossy(&ran.stdout),
        String::from_utf8_lossy(&ran.stderr),
    );
    assert!(
        what_was_said(at_the_environments).is_none(),
        "a question went to the proxy this process's environment names, which nobody chose"
    );
}

/// The child of the test above: one question, on a machine whose description
/// names no proxy, while the environment names one.
///
/// `#[ignore]`d on purpose — run under the ordinary suite it would prove
/// nothing, because nothing would have set the variables it is about.
#[test]
#[ignore = "run by a_proxy_in_the_environment_points_this_machines_question_nowhere, with the environment set"]
fn a_question_reaches_the_provider_under_whatever_the_environment_says() {
    let (provider, at_the_provider) = a_listener_that_reports_what_it_was_told();
    let happened = a_question_on_a_real_machine("under-an-environment", "", provider);

    let said = what_was_said(at_the_provider).unwrap_or_else(|| {
        panic!(
            "the question did not reach the provider this machine's own settings name: {}",
            happened.said
        )
    });
    assert!(
        began_a_conversation_of_its_own(&said),
        "the provider was asked to carry the question somewhere, which is what a variable in \
         this process's environment would have made of it: {}",
        String::from_utf8_lossy(&said)
    );
}

/// What was said at that door, waiting for the listener's own answer —
/// [`None`] where nobody came.
fn what_was_said(listener: JoinHandle<Option<Vec<u8>>>) -> Option<Vec<u8>> {
    listener.join().unwrap()
}

/// Whether what arrived is this machine asking a proxy to carry a question to
/// `provider`, which is the first line of the `CONNECT` conversation.
fn asked_to_connect(said: &[u8], provider: SocketAddr) -> bool {
    String::from_utf8_lossy(said).starts_with(&format!("CONNECT {provider} "))
}

/// Whether what arrived is a conversation begun with **this** party rather than
/// through it: a TLS handshake, which every `https://` provider is spoken to
/// with and which a `CONNECT` request is not.
///
/// One byte decides it — a TLS record of type *handshake* — and asserting on it
/// rather than merely on *not `CONNECT`* is what makes the straight road a
/// positive claim instead of the absence of one.
fn began_a_conversation_of_its_own(said: &[u8]) -> bool {
    said.first() == Some(&0x16)
}
