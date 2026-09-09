//! One line from an agent, carried out against the turn it belongs to.
//!
//! Everything an agent can ask for is `alo_protocol::FromAnAgent`, everything
//! that can come back is `alo_protocol::ToAnAgent`, and every decision between
//! the two is `alo_turn::Turning`'s. This file is the join, and its whole value
//! is that there is exactly one of it: a second road from a message to a turn
//! would be a second place law 2 has to be got right.
//!
//! # Nothing here decides anything
//!
//! The name of a verb goes to `alo_capability::Verbs::call` exactly as it
//! arrived — not trimmed, not lower-cased, not looked at. Whether a request is
//! a read or a change is decided twice and neither time here:
//! `alo_capability::Authorised::read` refuses a change offered as a read, and
//! `alo_capability::Proposal::checked` refuses a read offered for approval.
//! `FromAnAgent::waits_for_a_person` exists and is deliberately not consulted,
//! because a door that chose from it would be a third answer to ADR 0001 §5
//! that could disagree with the other two.
//!
//! # A refusal crosses in the words of whoever refused it
//!
//! Every failure inside a turn is `alo_turn::NotDone::said`, rendered with the
//! machine's own vocabulary and handed over as one sentence. This file words
//! two things and nothing else, and both are things no turn can say: a machine
//! where nobody has chosen anything to answer questions, and a question that
//! was put nowhere because parts of alo OS disagreed about where it should go.
//!
//! # A question to a model goes to what the person chose
//!
//! `crate::questions` is what the person's settings and this machine's runtime
//! come out of, and it answers one of four things: nothing was chosen, nothing
//! is running, the settings file does not hold, or here is the choice and here
//! is what answers it. Only the last reaches a turn, and the other three are
//! sentences somebody already says — this file writes none of them.
//!
//! *Nothing here has been chosen to answer questions* is not a placeholder and
//! never was: it is what a person who has picked neither a model nor a provider
//! is told for as long as alo OS exists.

use std::time::{Duration, SystemTime};

use alo_answering::Answering;
use alo_asking::Hosted;
use alo_capability::{AnswerError, Grants, ProposalId};
use alo_models::RuntimeError;
use alo_protocol::{FromAnAgent, ToAnAgent};
use alo_strings::{Filling, Said, Strings};
use alo_turn::{Answers, NoAnswer, Turning};

use alo_secrets::NotStored;

use crate::questions::{Questions, WhatAnswers};
use crate::words::{
    NO_KEY_FOR_THIS_PROVIDER, NO_KEYRING_FOR_A_PROVIDER, NOTHING_ANSWERS_QUESTIONS,
    NOTHING_WAS_ASKED, THE_KEYRING_IS_LOCKED, THE_KEYRING_REFUSED_US, Word,
};

/// Read one line as something an agent asked, and do it.
///
/// Answers with what to say back, always: a message that was not a request, a
/// verb nobody declared and a grant that ran out are all answers rather than
/// silences, which is `docs/contracts/daemon-protocol.md`'s *refused in words
/// and never dropped*.
pub fn what_an_agent_said(
    line: &str,
    turning: &mut Turning<'_, '_>,
    questions: &mut Questions,
    grants: &Grants,
    strings: &Strings,
    standing: Duration,
    now: SystemTime,
) -> ToAnAgent {
    match FromAnAgent::read(line) {
        Ok(asked) => carried_out(&asked, turning, questions, grants, strings, standing, now),
        Err(why) => ToAnAgent::refused(&why.said(strings)),
    }
}

/// The three things an agent can ask for, each through the turn's own door.
fn carried_out(
    asked: &FromAnAgent,
    turning: &mut Turning<'_, '_>,
    questions: &mut Questions,
    grants: &Grants,
    strings: &Strings,
    standing: Duration,
    now: SystemTime,
) -> ToAnAgent {
    match asked {
        FromAnAgent::Read { verb, .. } => {
            match turning.reading(verb, &asked.given(), grants, now) {
                Ok(answer) => ToAnAgent::did(&answer),
                Err(why) => ToAnAgent::refused(&why.said(strings)),
            }
        }
        FromAnAgent::Propose { verb, .. } => {
            match turning.proposing(verb, &asked.given(), grants, standing, now) {
                Ok(number) => waiting_under(turning, number, strings, now),
                Err(why) => ToAnAgent::refused(&why.said(strings)),
            }
        }
        FromAnAgent::Ask { question } => put_to_a_model(question, turning, questions, strings, now),
    }
}

/// Which sentence a refusal is, one per state.
///
/// A `match` with no wildcard, so a fifth state added to `alo_secrets` fails to
/// compile here rather than quietly becoming whichever arm a catch-all named.
/// `alo_secrets::NotStored` deliberately carries no words of its own — the four
/// sentences belong beside the daemon that says them, which is here.
const fn said_about(why: NotStored) -> Word {
    match why {
        NotStored::Unavailable => NO_KEYRING_FOR_A_PROVIDER,
        NotStored::Locked => THE_KEYRING_IS_LOCKED,
        NotStored::Missing => NO_KEY_FOR_THIS_PROVIDER,
        NotStored::Denied => THE_KEYRING_REFUSED_US,
    }
}

/// A question, put to whatever this person chose — or refused in one sentence.
///
/// The three refusals are three different things to go and fix, and each is
/// worded by whoever knows it: *choose something* is this crate's, because
/// this crate is where a machine that has been asked and has nothing is;
/// *the runtime is not reachable* is `alo-models`', because a runtime being up
/// is its fact and it already has the sentence; *your settings file says this*
/// is `alo-choosing`'s, and names the file and the line.
fn put_to_a_model(
    question: &str,
    turning: &mut Turning<'_, '_>,
    questions: &mut Questions,
    strings: &Strings,
    now: SystemTime,
) -> ToAnAgent {
    // Taken before the turn's answer borrows `questions`, and per question
    // rather than at startup: somebody may sign in after this daemon did.
    let keyring = questions.whose_keyring();
    match questions.what_answers() {
        WhatAnswers::Nothing => {
            ToAnAgent::refused(&strings.say(&NOTHING_ANSWERS_QUESTIONS.key(), &Filling::nothing()))
        }
        WhatAnswers::NotRunning => ToAnAgent::refused(&RuntimeError::Unreachable.said(strings)),
        WhatAnswers::NotSet(why) => ToAnAgent::refused(&why.said(strings)),
        // **The second of the three choices**, and alo's own service is this
        // one too (ADR 0014). A provider that needs a credential is now asked
        // for one: `alo_models::SecretRef` names where the key lives and
        // `alo_secrets` is what that name refers to. When the store will not
        // give it up the question is **not sent**, and it is not sent anywhere
        // else either — each of the four refusals is its own sentence, because
        // they are four different things for a person to go and do.
        WhatAnswers::FromAProvider {
            provider,
            model,
            places,
        } => {
            // Held out here because `Hosted::provider` borrows it, and it must
            // outlive the ask. This is the only place in the daemon where a
            // credential exists at all, and it lives no longer than the turn.
            let held;
            let key = match provider.key.as_ref() {
                None => None,
                Some(reference) => match Questions::a_key_from(&keyring, reference) {
                    Ok(secret) => {
                        held = secret;
                        Some(&held)
                    }
                    // Nothing was sent, and `NotStored::nothing_was_sent` is
                    // the type's own statement of that.
                    Err(why) => {
                        return ToAnAgent::refused(
                            &strings.say(&said_about(why).key(), &Filling::nothing()),
                        );
                    }
                },
            };
            // The source is the provider's own, read off the provider the
            // person's list resolved — never assumed, and never `ThisMachine`
            // because the address happened to look local.
            match Answering::chosen(provider.source(), places.policy()) {
                Ok(permission) => match turning.asking(
                    question,
                    model,
                    permission,
                    &Answers::Provider(Hosted::provider(provider, key)),
                    &places,
                    now,
                ) {
                    Ok(answer) => ToAnAgent::answered(
                        answer.text(),
                        &answer.came_from(strings),
                        answer.model(),
                    ),
                    Err(why) => ToAnAgent::refused(&nothing_answered(&why, strings)),
                },
                Err(why) => ToAnAgent::refused(&why.said(strings)),
            }
        }
        WhatAnswers::OnThisMachine {
            chosen,
            runtime,
            places,
        } => match chosen.asking(Some(places.policy())) {
            // Nothing is composed out of what a model said, here or anywhere:
            // the text crosses as the model's own words, and the line naming
            // where it came from is a sentence of ours beside it.
            Ok(permission) => match turning.asking(
                question,
                chosen.model(),
                permission,
                &Answers::Runtime(runtime),
                &places,
                now,
            ) {
                Ok(answer) => {
                    ToAnAgent::answered(answer.text(), &answer.came_from(strings), answer.model())
                }
                Err(why) => ToAnAgent::refused(&nothing_answered(&why, strings)),
            },
            // What an organisation permits, refusing what the person chose —
            // and the sentence names the rule rather than the machine.
            Err(why) => ToAnAgent::refused(&why.said(strings)),
        },
    }
}

/// The sentence for a question that was not answered.
///
/// [`alo_turn::NoAnswer`] words all of them but one. The exception is a
/// miswiring, which is this repository disagreeing with itself and has no
/// sentence of its own because there is nothing for a person to do about it —
/// so this crate says the one thing that is true and useful: it went nowhere,
/// and it is not theirs to fix.
fn nothing_answered(why: &NoAnswer, strings: &Strings) -> Said {
    why.said(strings)
        .unwrap_or_else(|| strings.say(&NOTHING_WAS_ASKED.key(), &Filling::nothing()))
}

/// The change that is now waiting, with the sentence the person will be asked.
///
/// `alo_protocol::ToAnAgent::proposed` takes the change rather than its number,
/// so an answer cannot be composed without the sentence — which means the
/// change has to be found among what is really waiting rather than assumed from
/// the number that was just handed back.
///
/// A change that is not there is a change that was proposed to stand for no
/// time at all. It is refused with the capability model's own sentence for a
/// number nothing is waiting under, rather than with a second one written here
/// that would say the same thing differently.
fn waiting_under(
    turning: &Turning<'_, '_>,
    number: ProposalId,
    strings: &Strings,
    now: SystemTime,
) -> ToAnAgent {
    turning
        .waiting_at(now)
        .find(|waiting| waiting.id == number)
        .map_or_else(
            || {
                ToAnAgent::refused(
                    &AnswerError::NothingWaiting {
                        number: number.as_u64(),
                    }
                    .said(strings),
                )
            },
            |waiting| ToAnAgent::proposed(waiting, strings, now),
        )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    // Named only here: the daemon itself never writes the type, it passes
    // along whatever `Questions` was built with.
    use crate::questions::WhoseKeyring;
    use crate::testing::{
        a_directory_of_our_own, a_message, a_runtime_saying, hour, noon, nothing_has_been_chosen,
        on_a_machine, on_a_machine_that_answers,
    };
    use alo_choosing::{Chosen, Which};
    use alo_keyring_fixture::AKeyringOfOurOwn;
    use alo_record::Record;

    /// This machine's own address, which is what a provider's has to be: a
    /// loopback one reports as this machine and the provider door refuses it.
    fn our_own_address() -> std::net::IpAddr {
        let asking = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();
        match asking
            .connect("192.0.2.1:9")
            .and_then(|()| asking.local_addr())
        {
            Ok(ours) => ours.ip(),
            Err(_) => std::net::IpAddr::from([127, 0, 0, 1]),
        }
    }

    /// **Each way a key is not handed over is its own sentence**, and no two
    /// are the same.
    ///
    /// The four states exist so that a person is sent to the right place: sign
    /// in, unlock, add the key, or find out why the machine said no. Two of
    /// them rendering the same sentence would put that right back, quietly, and
    /// nothing else in the system would notice.
    ///
    /// Every state is checked because `said_about` matches without a wildcard:
    /// this array and that `match` are the two halves of *no refusal falls
    /// through to somebody else's words*.
    #[test]
    fn every_way_a_key_is_not_handed_over_is_a_different_sentence() {
        let strings = crate::testing::in_english();
        let every = [
            NotStored::Unavailable,
            NotStored::Locked,
            NotStored::Missing,
            NotStored::Denied,
        ];

        let said: Vec<(NotStored, String)> = every
            .iter()
            .map(|why| {
                let sentence = strings
                    .say(&said_about(*why).key(), &Filling::nothing())
                    .text()
                    .to_owned();
                assert!(
                    !sentence.is_empty(),
                    "{why:?} renders nothing, so a person would be refused in silence"
                );
                // The claim the type exists to make, held up for every state
                // rather than for the one that happened to be convenient.
                assert!(why.nothing_was_sent(), "{why:?} claims something was sent");
                (*why, sentence)
            })
            .collect();

        for (one_why, one) in &said {
            for (other_why, other) in &said {
                assert!(
                    one_why == other_why || one != other,
                    "{one_why:?} and {other_why:?} say the same thing, so a person is                      sent to the wrong place for one of them: {one}"
                );
            }
        }
    }

    /// A key that is a credential to nothing, and never leaves this machine.
    const A_SYNTHETIC_KEY: &str = "sk-live-AGENTD-FIXTURE-ONLY-91c4";

    /// Put a synthetic key in the fixture's keyring, through a client of its own.
    fn stored_in(keyring: &AKeyringOfOurOwn, reference: &str, secret: &str) {
        let connection = zbus::blocking::connection::Builder::address(keyring.address().as_str())
            .unwrap()
            .build()
            .unwrap();
        let service = secret_service::blocking::SecretService::connect_with_existing(
            secret_service::EncryptionType::Dh,
            connection,
        )
        .unwrap();
        let mut attributes = std::collections::HashMap::new();
        attributes.insert("xdg:schema", "dev.alo.Provider");
        attributes.insert("reference", reference);
        service
            .get_default_collection()
            .unwrap()
            .create_item(
                "a provider key this test invented",
                attributes,
                secret.as_bytes(),
                true,
                "text/plain",
            )
            .unwrap();
    }

    /// A listener this test owns, which reports whether anybody connected to it.
    ///
    /// On this machine's own interface rather than loopback, because that is the
    /// only kind of address the Provider door will carry a key to.
    fn a_listener_that_reports_connections() -> (std::net::SocketAddr, std::thread::JoinHandle<bool>)
    {
        let listener =
            std::net::TcpListener::bind(std::net::SocketAddr::new(our_own_address(), 0)).unwrap();
        let at = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();
        let heard = std::thread::spawn(move || {
            let until = std::time::Instant::now() + Duration::from_secs(2);
            while std::time::Instant::now() < until {
                if listener.accept().is_ok() {
                    return true;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            false
        });
        (at, heard)
    }

    /// Settings naming two providers, with the person having chosen the first.
    fn a_person_who_chose(
        called: &str,
        chosen: std::net::SocketAddr,
        other: std::net::SocketAddr,
    ) -> std::path::PathBuf {
        let config = a_directory_of_our_own(called);
        let folder = config.join(alo_choosing::THE_FOLDER);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(
            folder.join(alo_choosing::THE_SETTINGS),
            format!(
                "format = 2

[answers]
provider = {{ name = \"Mine\", model = \"a-model\" }}

[[provider]]
name = \"Mine\"
endpoint = \"https://{chosen}\"

[[provider]]
name = \"Theirs\"
endpoint = \"https://{other}\"
"
            ),
        )
        .unwrap();
        config
    }

    /// **The key is fetched from the person's own keyring, and only the
    /// provider they chose is connected to.**
    ///
    /// A real `gnome-keyring-daemon` on a bus of the fixture's own holds a
    /// synthetic key. The daemon reads the person's settings, asks that keyring
    /// for `provider/Mine`, and opens a connection to the address they wrote
    /// down — and to nothing else.
    ///
    /// # What this proves, and what it deliberately stops short of
    ///
    /// The endpoint is `https://` at this machine's own address rather than
    /// loopback, because the production path allows nothing else: a loopback
    /// provider makes `Provider::source` say *this machine*, and
    /// `Asking::to_a_provider` refuses that outright as `Miswired::NotAProvider`
    /// — a provider on this machine belongs to the `Served` door. And
    /// `Provider::checked` refuses plain `http://` to anywhere that is not this
    /// machine. Those two together mean **the only address the Provider door
    /// will carry a key to is a real TLS endpoint.**
    ///
    /// The listener here speaks no TLS, so the handshake fails and no answer
    /// comes back. **The connection is the assertion**, and it is enough for
    /// what this test is about: nothing is connected to until a key has been
    /// obtained, so a connection happening at all is the key having come out of
    /// a real keyring. That the *bytes* carrying it are correct is asserted a
    /// layer down, where `alo-asking` reads them off its own server.
    ///
    /// Reading them off **this** server needs a certificate the daemon trusts,
    /// and it trusts the compiled-in Mozilla roots and nothing else — see
    /// `docs/autonomy/updates/` for that, which is an owner's decision rather
    /// than something a test may arrange.
    #[test]
    fn the_key_is_fetched_and_only_the_chosen_provider_is_connected_to() {
        let keyring = AKeyringOfOurOwn::started("agentd-reaches");
        stored_in(&keyring, "provider/Mine", A_SYNTHETIC_KEY);

        let (chosen, connected) = a_listener_that_reports_connections();
        let (other, nobody) = a_listener_that_reports_connections();
        let config = a_person_who_chose("reaches", chosen, other);

        let mut questions = Questions::of_a_session(
            Some(config.into_os_string()),
            None,
            alo_models::Catalogue::built_in().unwrap(),
            None,
            WhoseKeyring::On(keyring.bus()),
        );

        let mut record = Record::default();
        let said = on_a_machine_that_answers(&mut record, |turning, _grants, strings| {
            put_to_a_model(
                "may the tenant sublet?",
                turning,
                &mut questions,
                strings,
                noon(),
            )
        });

        assert!(
            connected.join().unwrap(),
            "the provider the person chose was never connected to, so the key never left the              keyring"
        );
        assert!(
            !nobody.join().unwrap(),
            "a provider the person did not choose was connected to"
        );

        // **And the key really was obtained.** The ask fails here because a
        // plain socket speaks no TLS, which is expected — but it must not have
        // failed for want of a credential, and those are four sentences this
        // crate can name exactly.
        let refusal = said.refusal().unwrap();
        let strings = crate::testing::in_english();
        for word in [
            NO_KEYRING_FOR_A_PROVIDER,
            THE_KEYRING_IS_LOCKED,
            NO_KEY_FOR_THIS_PROVIDER,
            THE_KEYRING_REFUSED_US,
        ] {
            assert_ne!(
                refusal.text(),
                strings.say(&word.key(), &Filling::nothing()).text(),
                "the question was refused for want of a key, so nothing was proved about                  fetching one"
            );
        }
    }

    /// **Every way the store says no ends with nothing sent to any provider.**
    ///
    /// Each state is produced for real: no bus at all, a real keyring with no
    /// such key, a real collection that is locked, and a running bus that
    /// refuses to carry the message. None is injected, and the difference
    /// matters — an injected transport failure would prove that a broken socket
    /// sends nothing, which nobody doubted.
    ///
    /// The assertion is the same for all four and is the promise the four states
    /// exist to keep: **the provider is never connected to, and nothing else
    /// answers in its place.**
    #[test]
    fn no_store_refusal_reaches_a_provider_or_falls_back() {
        let missing = AKeyringOfOurOwn::started("agentd-missing");

        let locked = AKeyringOfOurOwn::started("agentd-locked");
        stored_in(&locked, "provider/Mine", A_SYNTHETIC_KEY);
        {
            let connection =
                zbus::blocking::connection::Builder::address(locked.address().as_str())
                    .unwrap()
                    .build()
                    .unwrap();
            let service = secret_service::blocking::SecretService::connect_with_existing(
                secret_service::EncryptionType::Dh,
                connection,
            )
            .unwrap();
            service.get_default_collection().unwrap().lock().unwrap();
        }

        let denied = AKeyringOfOurOwn::started_where_the_bus_can_refuse("agentd-denied");
        stored_in(&denied, "provider/Mine", A_SYNTHETIC_KEY);
        denied.stop_letting_anyone_reach_the_keyring();

        for (which, keyring) in [
            ("no store at all", WhoseKeyring::Nobodys),
            ("a store with no such key", WhoseKeyring::On(missing.bus())),
            ("a locked store", WhoseKeyring::On(locked.bus())),
            ("a store that refused us", WhoseKeyring::On(denied.bus())),
        ] {
            let (chosen, nobody) = a_listener_that_reports_connections();
            let (other, also_nobody) = a_listener_that_reports_connections();
            let config = a_person_who_chose(&format!("refused-{}", chosen.port()), chosen, other);

            let mut questions = Questions::of_a_session(
                Some(config.into_os_string()),
                None,
                alo_models::Catalogue::built_in().unwrap(),
                None,
                keyring,
            );

            let mut record = Record::default();
            let said = on_a_machine_that_answers(&mut record, |turning, _grants, strings| {
                put_to_a_model(
                    "may the tenant sublet?",
                    turning,
                    &mut questions,
                    strings,
                    noon(),
                )
            });

            assert!(
                said.refusal().is_some(),
                "{which} was not refused at all, so a question went out without a key"
            );
            let refusal = said.refusal().unwrap();
            assert!(
                !refusal.is_a_bug(),
                "{which} was reported as a fault in alo OS: {refusal:?}"
            );
            assert!(
                !nobody.join().unwrap(),
                "{which}: the chosen provider was connected to without a key"
            );
            assert!(
                !also_nobody.join().unwrap(),
                "{which}: a provider nobody chose was connected to, which is a fallback"
            );
        }
    }

    /// **A provider that needs a credential sends nothing at all**, and says
    /// why.
    ///
    /// The second of the three model choices, on a machine with **no credential
    /// store**: this `Questions` names `WhoseKeyring::Nobodys`, which is a
    /// machine whose image ships no Secret Service, and no bus is opened at
    /// all. So the question is refused at the moment of asking — **not sent
    /// without its key**, and not answered anywhere else instead.
    ///
    /// The refusal is checked against the **vocabulary** rather than a phrase
    /// typed here. An earlier version quoted the English, and reworking that
    /// sentence broke a test whose subject had not changed — a test that
    /// asserts wording is a test about the wording.
    ///
    /// The provider's address is a listener this test owns, on this machine's
    /// own interface rather than loopback, because a loopback address reports
    /// as this machine and the provider door refuses that outright. **The
    /// assertion is that it was never connected to.**
    #[test]
    fn a_provider_that_needs_a_key_is_refused_and_nothing_is_sent() {
        let listener =
            std::net::TcpListener::bind(std::net::SocketAddr::new(our_own_address(), 0)).unwrap();
        let at = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();
        let heard = std::thread::spawn(move || {
            let until = std::time::Instant::now() + Duration::from_secs(2);
            while std::time::Instant::now() < until {
                if listener.accept().is_ok() {
                    return true;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            false
        });

        let config = a_directory_of_our_own("needs-a-key");
        let folder = config.join(alo_choosing::THE_FOLDER);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(
            folder.join(alo_choosing::THE_SETTINGS),
            format!(
                "format = 2

[answers]
provider = {{ name = \"Mine\", model = \"a-model\" }}

                 [[provider]]
name = \"Mine\"
endpoint = \"https://{at}\"
"
            ),
        )
        .unwrap();
        let mut questions = Questions::of_a_session(
            Some(config.into_os_string()),
            None,
            alo_models::Catalogue::built_in().unwrap(),
            None,
            WhoseKeyring::Nobodys,
        );

        let mut record = Record::default();
        let said = on_a_machine_that_answers(&mut record, |turning, _grants, strings| {
            put_to_a_model(
                "may the tenant sublet?",
                turning,
                &mut questions,
                strings,
                noon(),
            )
        });

        let refusal = said.refusal().unwrap();
        assert_eq!(
            refusal.text(),
            crate::testing::in_english()
                .say(&NO_KEYRING_FOR_A_PROVIDER.key(), &Filling::nothing())
                .text(),
            "the refusal was not the one for a machine with no store: {refusal:?}"
        );
        assert!(!refusal.is_a_bug(), "{refusal:?}");

        assert!(
            !heard.join().unwrap(),
            "a question was sent to a provider whose key this machine cannot reach"
        );
        // And nothing was written down as having left, because nothing did.
        assert_eq!(
            record
                .answering(&alo_record::Asking::anything().only(alo_record::Only::Egress))
                .count(),
            0
        );
    }

    /// **A read answers inside the turn**, and what comes back is what the
    /// machine found rather than a promise to find it.
    #[test]
    fn a_read_is_carried_out_and_answered() {
        on_a_machine("a-read", |turning, grants, strings, folder, _| {
            let said = what_an_agent_said(
                &a_message(&format!(
                    r#"{{"read":{{"verb":"list_folder","given":[{{"named":"folder","is":"{}"}}]}}}}"#,
                    folder.display()
                )),
                turning,
                &mut nothing_has_been_chosen(),
                grants,
                strings,
                hour(),
                noon(),
            );

            assert!(matches!(
                said.done(),
                Some(alo_protocol::Done::Listed { things, .. }) if things.len() == 1
            ));
        });
    }

    /// **A change comes back as a number and the sentence it waits on**, so the
    /// agent has something to show the person rather than a handle only this
    /// machine understands.
    #[test]
    fn a_change_comes_back_with_the_sentence_it_waits_on() {
        on_a_machine("a-change", |turning, grants, strings, _, invoice| {
            let said = what_an_agent_said(
                &a_message(&format!(
                    r#"{{"propose":{{"verb":"rename_file","given":[{{"named":"file","is":"{}"}},{{"named":"name","is":"march-final.pdf"}}]}}}}"#,
                    invoice.display()
                )),
                turning,
                &mut nothing_has_been_chosen(),
                grants,
                strings,
                hour(),
                noon(),
            );

            assert!(said.waits_for_a_person());
            assert_eq!(turning.waiting_at(noon()).count(), 1);
            assert!(invoice.is_file(), "a proposal moved a file");
        });
    }

    /// **A verb nobody declared is turned away by the closed list**, and the
    /// name reaches it exactly as it was written — `/bin/sh` and all.
    #[test]
    fn a_verb_nobody_declared_is_refused_by_the_list() {
        on_a_machine("no-such-verb", |turning, grants, strings, _, _| {
            let said = what_an_agent_said(
                &a_message(r#"{"read":{"verb":"/bin/sh","given":[]}}"#),
                turning,
                &mut nothing_has_been_chosen(),
                grants,
                strings,
                hour(),
                noon(),
            );

            let refusal = said.refusal().unwrap();
            assert!(refusal.text().contains("/bin/sh"), "{refusal:?}");
        });
    }

    /// **An agent cannot answer its own question**, and the refusal is
    /// `alo-protocol`'s rather than one written here: the two doors are two
    /// before anything reaches a turn.
    #[test]
    fn an_agent_that_approves_something_is_refused_before_the_turn() {
        on_a_machine("self-approval", |turning, grants, strings, _, invoice| {
            what_an_agent_said(
                &a_message(&format!(
                    r#"{{"propose":{{"verb":"rename_file","given":[{{"named":"file","is":"{}"}},{{"named":"name","is":"gone.pdf"}}]}}}}"#,
                    invoice.display()
                )),
                turning,
                &mut nothing_has_been_chosen(),
                grants,
                strings,
                hour(),
                noon(),
            );

            let said = what_an_agent_said(
                &a_message(r#"{"approve":{"number":1}}"#),
                turning,
                &mut nothing_has_been_chosen(),
                grants,
                strings,
                hour(),
                noon(),
            );

            assert!(
                said.refusal()
                    .unwrap()
                    .text()
                    .contains("cannot answer a question that was put to a person")
            );
            assert_eq!(
                turning.waiting_at(noon()).count(),
                1,
                "the change stopped waiting for the person"
            );
            assert!(invoice.is_file(), "an agent approved its own change");
        });
    }

    /// **A message that is not a request is answered in words**, with
    /// `alo-protocol`'s own sentence, and nothing reaches the turn.
    #[test]
    fn a_message_that_is_not_a_request_is_answered_and_reaches_no_turn() {
        on_a_machine("gibberish", |turning, grants, strings, _, _| {
            for line in [
                "not json at all",
                r#"{"format":9,"asks":{"read":{"verb":"list_folder","given":[]}}}"#,
                r#"{"format":1,"asks":{"run":{"command":"rm -rf /"}}}"#,
            ] {
                let said = what_an_agent_said(
                    line,
                    turning,
                    &mut nothing_has_been_chosen(),
                    grants,
                    strings,
                    hour(),
                    noon(),
                );
                assert!(said.refusal().is_some(), "{line}");
            }
            assert!(!turning.is_closed());
        });
    }

    /// **A question for a model is refused because nothing has been chosen to
    /// answer one**, which is the true state of a machine nobody has set a
    /// model or a provider on — and the sentence names the panel to go to.
    #[test]
    fn a_question_for_a_model_says_nothing_has_been_chosen_to_answer_one() {
        on_a_machine("a-question", |turning, grants, strings, _, _| {
            let said = what_an_agent_said(
                &a_message(r#"{"ask":{"question":"what is in this contract?"}}"#),
                turning,
                &mut nothing_has_been_chosen(),
                grants,
                strings,
                hour(),
                noon(),
            );

            let refusal = said.refusal().unwrap();
            assert!(refusal.text().contains("Settings"), "{refusal:?}");
            assert!(
                !refusal.is_a_bug(),
                "the sentence this crate says is not one it declared"
            );
        });
    }

    /// **A change that stands for no time at all is not waiting**, so what
    /// comes back is the capability model's own sentence for a number nothing
    /// is waiting under rather than a proposal nobody could answer.
    #[test]
    fn a_change_that_waits_for_no_time_is_not_answered_as_waiting() {
        on_a_machine("no-standing", |turning, grants, strings, _, invoice| {
            let said = what_an_agent_said(
                &a_message(&format!(
                    r#"{{"propose":{{"verb":"rename_file","given":[{{"named":"file","is":"{}"}},{{"named":"name","is":"never.pdf"}}]}}}}"#,
                    invoice.display()
                )),
                turning,
                &mut nothing_has_been_chosen(),
                grants,
                strings,
                Duration::from_secs(0),
                noon(),
            );

            assert!(!said.waits_for_a_person());
            assert!(said.refusal().is_some());
            assert_eq!(turning.waiting_at(noon()).count(), 0);
        });
    }

    /// A machine where this person chose these weights and this answers them.
    fn holding(model: &str, said: Result<String, RuntimeError>) -> Questions {
        Questions::already_found(
            Chosen::of(Which::Brought, model).unwrap(),
            a_runtime_saying(said),
            None,
        )
    }

    /// **A question reaches the model the person chose, and the answer comes
    /// back in the model's own words** — with the model named beside it, so
    /// whoever is reading knows what answered.
    #[test]
    fn a_question_is_put_to_what_the_person_chose_and_the_answer_comes_back() {
        let mut record = Record::default();
        on_a_machine_that_answers(&mut record, |turning, _, strings| {
            let said = what_an_agent_said(
                &a_message(r#"{"ask":{"question":"what is in this contract?"}}"#),
                turning,
                &mut holding("my-finetune", Ok("a sublet clause".to_owned())),
                &Grants::default(),
                strings,
                hour(),
                noon(),
            );

            assert!(
                matches!(
                    &said,
                    ToAnAgent::Answered { text, model, .. }
                        if text == "a sublet clause" && model == "my-finetune"
                ),
                "{said:?}"
            );
        });

        // Law 1's other half: it was answered here, so the entry says so and
        // nothing on it is about a destination.
        assert_eq!(record.len(), 1, "a question left no record");
    }

    /// **A model that does not answer is a refusal in words**, and the sentence
    /// is `alo-models`' own rather than one written in this file.
    #[test]
    fn a_model_that_does_not_answer_is_refused_in_the_runtimes_own_words() {
        let mut record = Record::default();
        on_a_machine_that_answers(&mut record, |turning, _, strings| {
            let said = what_an_agent_said(
                &a_message(r#"{"ask":{"question":"what is in this contract?"}}"#),
                turning,
                &mut holding("my-finetune", Err(RuntimeError::Unreachable)),
                &Grants::default(),
                strings,
                hour(),
                noon(),
            );

            let refusal = said.refusal().unwrap();
            assert!(!refusal.is_a_bug(), "{refusal:?}");
            assert!(!refusal.text().contains("Settings"), "{refusal:?}");
        });

        // Nothing was sent and nothing was answered, so there is nothing an
        // entry could truthfully say — `alo-turn`'s decision, honoured here.
        assert_eq!(
            record.len(),
            0,
            "a question that went nowhere left a record"
        );
    }

    /// **A settings file that does not hold is refused with its own sentence**,
    /// naming the file — not with *you have chosen nothing*, which would be
    /// false and would send the person to a panel that already agrees with them.
    #[test]
    fn a_settings_file_that_does_not_hold_names_itself_rather_than_settings() {
        let config = a_directory_of_our_own("doing-bad-settings");
        let folder = config.join(alo_choosing::THE_FOLDER);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(
            folder.join(alo_choosing::THE_SETTINGS),
            "format = 1\n[answers\n",
        )
        .unwrap();
        let mut questions = Questions::of_a_session(
            Some(config.clone().into_os_string()),
            None,
            alo_models::Catalogue::built_in().unwrap(),
            None,
            WhoseKeyring::Nobodys,
        );

        let mut record = Record::default();
        on_a_machine_that_answers(&mut record, |turning, _, strings| {
            let said = what_an_agent_said(
                &a_message(r#"{"ask":{"question":"what is in this contract?"}}"#),
                turning,
                &mut questions,
                &Grants::default(),
                strings,
                hour(),
                noon(),
            );

            let refusal = said.refusal().unwrap();
            assert!(!refusal.is_a_bug(), "{refusal:?}");
            assert!(
                refusal.text().contains(&config.display().to_string()),
                "{refusal:?}"
            );
        });
    }

    /// **A question is never a change**, whatever answered it: nothing waits
    /// for the person and the turn is still open for the next thing.
    #[test]
    fn a_question_answered_leaves_nothing_waiting_for_a_person() {
        let mut record = Record::default();
        on_a_machine_that_answers(&mut record, |turning, _, strings| {
            let said = what_an_agent_said(
                &a_message(r#"{"ask":{"question":"what is in this contract?"}}"#),
                turning,
                &mut holding("my-finetune", Ok("a sublet clause".to_owned())),
                &Grants::default(),
                strings,
                hour(),
                noon(),
            );

            assert!(!said.waits_for_a_person());
            assert_eq!(turning.waiting_at(noon()).count(), 0);
            assert!(!turning.is_closed());
        });
    }
}
