//! The three verbs an agent proposes a change to the network through.
//!
//! ★ *System verbs through the privileged broker: printers, network, updates,
//! storage.* The network is a change to the whole machine — how everything on it
//! reaches anywhere — so there are three verbs and every one of them **is a
//! change**: it waits for one approval of the sentence naming it, and that
//! approval makes the change once, through the privileged broker, and by no
//! other road.
//!
//! | verb | the sentence a person approves |
//! |---|---|
//! | `join_network` | *join the Wi-Fi network Home, and this conversation keeps its connection* |
//! | `forget_network` | *forget the Wi-Fi network Office, so this machine no longer joins it on its own, and this conversation keeps its connection* |
//! | `switch_wireless` | *turn Wi-Fi off, and this conversation loses its connection until this machine is connected again* |
//!
//! # A network by the name it announces, and never by anything else
//!
//! The argument is the name — one name, no path, no control character, at most
//! thirty-two characters as a Wi-Fi name is — so the sentence reads the way the
//! network is listed. Which network that name is, is asked of the network
//! manager at the moment the change is carried out (`crate::choosing`); a name no
//! network has, or two networks share, changes nothing.
//!
//! # No verb has anywhere to put a password
//!
//! Every argument is a name of at most thirty-two characters or one of a list of
//! options the verb wrote down, and none of them is called anything a password
//! could be passed as. Joining a protected network asks the person for its
//! password, in their own session, where the agent cannot read it
//! (`alo_networks::secret_agent`). `tests` below walk every argument of every
//! verb for this, and ADR 0049 is the decision.
//!
//! # Each says what it does to this conversation
//!
//! `this_conversation` is an argument, and the sentence names it, so a change
//! that would cut the connection the conversation is answered over says so in
//! the one sentence a person approves (`crate::cutting`). What the agent sends
//! there is checked against what is true before the proposal is made
//! (`crate::proposable`) and again before it is carried out.
//!
//! # Why none needs a grant
//!
//! A grant is over a path or an application, and a network is neither: the
//! change is to the machine's own network settings, which no grant covers. What
//! protects the person is the approval itself — one sentence, naming the network
//! and what the change does to the conversation, answered once — and the reason
//! is written into each declaration and in ADR 0049 §2, as
//! `docs/contracts/agent-verbs.md` rule 5 asks.
//!
//! # What is deliberately not a verb
//!
//! **Setting the proxy** — a person sets it in Settings (`crate::set_proxy_by_hand`),
//! and an agent proposes no proxy: one address that every connection this
//! machine makes goes through is not a sentence to be persuaded into approving
//! (ADR 0049 §3). **Configuring a VPN** — not in v0.5, and its absence is
//! recorded there too. **Listing the networks nearby** — a list of the networks
//! around a machine is where it is.

use alo_broker::Switch;
use alo_capability::{
    Arg, Authorised, Call, Effect, Offered, Requires, Takes, Value, Verb, VerbError, Verbs,
    VerbsError,
};

use crate::refusing::NotChanged;
use crate::wanted::{Change, ThisConversation, Wanted};
use crate::words;

/// The name the verb that joins a network is asked for by.
pub const JOIN_NETWORK: &str = "join_network";

/// The name the verb that forgets a network is asked for by.
pub const FORGET_NETWORK: &str = "forget_network";

/// The name the verb that turns Wi-Fi on or off is asked for by.
pub const SWITCH_WIRELESS: &str = "switch_wireless";

/// The most characters a network's name may be: a Wi-Fi name is at most
/// thirty-two bytes.
pub const LONGEST_NAME: usize = alo_networks::LONGEST_NAME;

/// The reason none of the three needs a grant, carried in each declaration.
const WHY_NO_GRANT: &str = "a network is neither a path nor an application, the change is to this \
                            machine's own network settings, and the person approves this one \
                            change by the sentence naming the network and what it does to this \
                            conversation's connection";

/// Why a network verb could not be declared.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Declaring {
    /// A declaration that does not satisfy the contract.
    #[error(transparent)]
    Verb(#[from] VerbError),
    /// A name already on the list.
    #[error(transparent)]
    List(#[from] VerbsError),
}

/// The three verbs, as a list of their own.
///
/// # Errors
/// [`Declaring`], which the verbs as written cannot cause.
pub fn network_verbs() -> Result<Verbs, Declaring> {
    let mut verbs = Verbs::default();
    declare_into(&mut verbs)?;
    Ok(verbs)
}

/// Put the three verbs on an existing list.
///
/// # Errors
/// [`Declaring::List`] if the list already has a verb of one of these names.
pub fn declare_into(verbs: &mut Verbs) -> Result<(), Declaring> {
    let network = || {
        Arg::taking(
            words::NETWORK,
            words::THE_NETWORK,
            Takes::name(LONGEST_NAME),
        )
    };
    for (name, purpose, first, sentence) in [
        (
            JOIN_NETWORK,
            words::JOIN_PURPOSE,
            network(),
            words::JOIN_SENTENCE,
        ),
        (
            FORGET_NETWORK,
            words::FORGET_PURPOSE,
            network(),
            words::FORGET_SENTENCE,
        ),
        (
            SWITCH_WIRELESS,
            words::WIRELESS_PURPOSE,
            Arg::taking(
                words::WIRELESS,
                words::THE_WIRELESS,
                Takes::choice([
                    Offered::called(Switch::On.written(), words::ON),
                    Offered::called(Switch::Off.written(), words::OFF),
                ]),
            ),
            words::WIRELESS_SENTENCE,
        ),
    ] {
        verbs.declare(Verb::checked(
            name,
            purpose,
            Effect::Change,
            vec![
                first,
                Arg::taking(
                    words::THIS_CONVERSATION,
                    words::THE_CONVERSATION,
                    Takes::choice([
                        Offered::called(
                            ThisConversation::KeepsItsConnection.option(),
                            words::KEEPS_ITS_CONNECTION,
                        ),
                        Offered::called(
                            ThisConversation::LosesItsConnection.option(),
                            words::LOSES_ITS_CONNECTION,
                        ),
                    ]),
                ),
            ],
            Requires::nothing_because(WHY_NO_GRANT),
            sentence,
        )?)?;
    }
    Ok(())
}

/// The change a call asks for, and what it says it does to the conversation.
///
/// # Errors
/// [`NotChanged::NotANetworkChange`] for another verb's call, or one whose
/// arguments are missing or of the wrong kind.
pub fn wanted(call: &Call) -> Result<Wanted, NotChanged> {
    let chosen = |argument: &str| match call.value(argument) {
        Some(Value::Choice { chosen, .. }) => Some(chosen.as_str()),
        _ => None,
    };
    let named = || match call.value(words::NETWORK) {
        Some(Value::Name(called)) => Some(called.clone()),
        _ => None,
    };
    let change = match call.verb() {
        JOIN_NETWORK => named().map(Change::Join),
        FORGET_NETWORK => named().map(Change::Forget),
        SWITCH_WIRELESS => chosen(words::WIRELESS)
            .and_then(Switch::read)
            .map(Change::Wireless),
        _ => None,
    };
    let says = chosen(words::THIS_CONVERSATION).and_then(ThisConversation::chosen);
    match (change, says) {
        (Some(change), Some(says)) => Ok(Wanted::of(change, says)),
        _ => Err(NotChanged::NotANetworkChange),
    }
}

/// The change a person approved, from the authority that approval became.
///
/// # Errors
/// [`NotChanged::NotANetworkChange`] for another verb's authority, or one that
/// came from no approval.
pub fn approved(authorised: &Authorised) -> Result<(Wanted, u64), NotChanged> {
    let approval = authorised
        .from_approval()
        .ok_or(NotChanged::NotANetworkChange)?;
    Ok((wanted(authorised.call())?, approval.as_u64()))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;
    use alo_capability::{CallError, Given};

    /// The three names.
    const EVERY_VERB: [&str; 3] = [JOIN_NETWORK, FORGET_NETWORK, SWITCH_WIRELESS];

    /// **Three verbs, every one a change, none needing a grant, each saying
    /// why.**
    #[test]
    fn three_verbs_every_one_a_change_that_needs_no_grant_and_says_why() {
        let verbs = network_verbs().unwrap();
        assert_eq!(verbs.len(), 3);
        for name in EVERY_VERB {
            let verb = verbs.of(name).unwrap();
            assert_eq!(verb.effect(), Effect::Change, "{name}");
            assert!(
                matches!(verb.requires(), Requires::Nothing { reason } if reason.contains("approves")),
                "{name}"
            );
        }
    }

    /// **No network verb has anywhere to put a password.** Every argument of
    /// every verb is a name no longer than a Wi-Fi network's, or a choice among
    /// options the verb wrote down; none is named for a secret; and the options
    /// are exactly the ones this file offers — so nothing a person could type
    /// as a password is a value any of them takes.
    #[test]
    fn no_network_verb_has_anywhere_to_put_a_password() {
        let verbs = network_verbs().unwrap();
        for name in EVERY_VERB {
            let verb = verbs.of(name).unwrap();
            for argument in verb.args() {
                let called = argument.name().to_lowercase();
                for secret in [
                    "password",
                    "passphrase",
                    "psk",
                    "secret",
                    "key",
                    "credential",
                ] {
                    assert!(
                        !called.contains(secret),
                        "{name} has an argument called {called}"
                    );
                }
                match argument.takes() {
                    Takes::Name { longest } => {
                        assert_eq!(argument.name(), words::NETWORK, "{name}");
                        assert_eq!(*longest, LONGEST_NAME, "{name}");
                    }
                    Takes::Choice(options) => {
                        let offered: Vec<&str> = options.iter().map(Offered::name).collect();
                        assert!(
                            offered == ["on", "off"]
                                || offered == ["keeps_its_connection", "loses_its_connection"],
                            "{name} offers {offered:?}"
                        );
                    }
                    other => panic!("{name} takes {other:?}"),
                }
            }
            for extra in ["password", "psk", "passphrase"] {
                let mut given = vec![
                    (
                        words::THIS_CONVERSATION,
                        Given::text("keeps_its_connection"),
                    ),
                    (extra, Given::text("correct horse battery")),
                ];
                given.push(if name == SWITCH_WIRELESS {
                    (words::WIRELESS, Given::text("on"))
                } else {
                    (words::NETWORK, Given::text("Home"))
                });
                assert!(verbs.call(name, &given).is_err(), "{name} took a {extra}");
            }
        }
    }

    /// **The sentence names the network and what the change does to this
    /// conversation**, filled from the validated arguments.
    #[test]
    fn the_sentence_a_person_approves_says_what_happens_to_this_conversation() {
        let verbs = network_verbs().unwrap();
        let strings = in_english();
        for (name, first, conversation, sentence) in [
            (
                JOIN_NETWORK,
                (words::NETWORK, " Home "),
                "keeps_its_connection",
                "join the Wi-Fi network Home, and this conversation keeps its connection",
            ),
            (
                FORGET_NETWORK,
                (words::NETWORK, "Office"),
                "loses_its_connection",
                "forget the Wi-Fi network Office, so this machine no longer joins it on its own, \
                 and this conversation loses its connection until this machine is connected again",
            ),
            (
                SWITCH_WIRELESS,
                (words::WIRELESS, "off"),
                "loses_its_connection",
                "turn Wi-Fi off, and this conversation loses its connection until this machine is \
                 connected again",
            ),
        ] {
            let call = verbs
                .call(
                    name,
                    &[
                        (first.0, Given::text(first.1)),
                        (words::THIS_CONVERSATION, Given::text(conversation)),
                    ],
                )
                .unwrap();
            assert!(call.waits_for_approval(), "{name}");
            assert_eq!(call.sentence(&strings).text(), sentence);
        }
    }

    /// **Nothing shaped like a path, a device, a command or a second line
    /// becomes a call**, and neither does a switch or a consequence that is not
    /// one of the options.
    #[test]
    fn anything_but_a_name_or_an_offered_option_never_becomes_a_call() {
        let verbs = network_verbs().unwrap();
        let keeps = || {
            (
                words::THIS_CONVERSATION,
                Given::text("keeps_its_connection"),
            )
        };
        for given in [
            Given::text("/etc/NetworkManager/system-connections/Home.nmconnection"),
            Given::text("Home\nreboot"),
            Given::text(""),
            Given::text("x".repeat(LONGEST_NAME + 1)),
            Given::number(7),
        ] {
            for name in [JOIN_NETWORK, FORGET_NETWORK] {
                assert!(
                    verbs
                        .call(name, &[(words::NETWORK, given.clone()), keeps()])
                        .is_err(),
                    "{name} took {given:?}"
                );
            }
        }
        assert!(
            verbs
                .call(
                    SWITCH_WIRELESS,
                    &[(words::WIRELESS, Given::text("on")), keeps()]
                )
                .is_ok()
        );
        for (wireless, conversation) in [
            ("airplane", "keeps_its_connection"),
            ("ON", "keeps_its_connection"),
            ("on", "who_knows"),
            ("on", "Keeps its connection"),
        ] {
            let call = verbs.call(
                SWITCH_WIRELESS,
                &[
                    (words::WIRELESS, Given::text(wireless)),
                    (words::THIS_CONVERSATION, Given::text(conversation)),
                ],
            );
            assert!(
                call.is_err(),
                "switch_wireless took {wireless} and {conversation}"
            );
        }
    }

    /// **No verb configures a VPN, sets a proxy or its password, lists networks
    /// or runs anything**, so an agent can ask for none of them here.
    ///
    /// The proxy's password is on this list for ADR 0060's sake as well as ADR
    /// 0049 §3's: a person may now give their own machine's proxy the password
    /// it asks for, and **no agent reaches any of it**. There are three verbs,
    /// they are the three below, and the whole list is held to three by
    /// `three_verbs_every_one_a_change_that_needs_no_grant_and_says_why`.
    #[test]
    fn no_verb_configures_a_vpn_sets_a_proxy_or_lists_networks() {
        let verbs = network_verbs().unwrap();
        for name in [
            "configure_vpn",
            "connect_vpn",
            "add_vpn",
            "set_proxy",
            "set_proxy_password",
            "set_proxy_credential",
            "sign_in_to_proxy",
            "list_networks",
            "scan_networks",
            "join_network_with_password",
            "set_network_password",
        ] {
            assert!(
                matches!(verbs.call(name, &[]), Err(CallError::NoSuchVerb { .. })),
                "{name}"
            );
        }
        assert_eq!(verbs.len(), EVERY_VERB.len(), "a fourth verb was declared");
        for name in EVERY_VERB {
            assert!(!name.contains("proxy"), "{name}");
        }
    }

    /// **A call reads back as exactly the change it names**, and a call to
    /// anything else is not a change to the network.
    #[test]
    fn a_call_reads_back_as_the_change_it_names() {
        let verbs = network_verbs().unwrap();
        let call = verbs
            .call(
                SWITCH_WIRELESS,
                &[
                    (words::WIRELESS, Given::text("off")),
                    (
                        words::THIS_CONVERSATION,
                        Given::text("loses_its_connection"),
                    ),
                ],
            )
            .unwrap();
        assert_eq!(
            wanted(&call),
            Ok(Wanted::of(
                Change::Wireless(Switch::Off),
                ThisConversation::LosesItsConnection
            ))
        );
    }

    /// A name already taken is not replaced.
    #[test]
    fn a_name_already_taken_is_not_replaced() {
        let mut verbs = network_verbs().unwrap();
        assert!(matches!(declare_into(&mut verbs), Err(Declaring::List(_))));
    }
}
