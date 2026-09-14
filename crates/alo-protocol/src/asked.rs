//! Everything that can arrive, in one closed list.
//!
//! Eleven requests, and there is no twelfth. What makes this file worth having on
//! its own is that it is deliberately **not** public: the two types a caller of
//! this crate ever holds are [`FromAnAgent`](crate::FromAnAgent) and
//! [`FromAPerson`](crate::FromAPerson), and this is the list they are each cut
//! out of.
//!
//! # Why the list is one thing and the doors are two
//!
//! Approving a change is the person's answer. If the same door that takes
//! *propose this change* also took *approve number 7*, then an agent holding
//! that door could approve its own proposal, and ADR 0001 §5 — one approval,
//! one execution, given by a person — would be true of the capability model and
//! false of the socket in front of it. So there are two doors, they answer with
//! two different types, and neither can produce the other's.
//!
//! Keeping the list itself in one place is what makes that a division rather
//! than two lists that could drift: a twelfth request has to be given to one
//! door or the other before this crate will compile, and a request that is not
//! one of the eleven is not a request at all.
//!
//! **Which side of a socket a caller is really on is not this crate's
//! question.** That is peer credentials on a Unix socket, and it is
//! `alo-agentd`'s. What is settled here is that once the daemon knows, it reads
//! with the door for that side and there is no way for a message to cross.
//!
//! # Law 2, at the one place a caller can reach
//!
//! Nothing in this list can carry a command. A request names a **verb** — a
//! string that is looked up against the closed list this machine offers, by
//! `alo_capability::Verbs::call` and nowhere else — and gives
//! [`Argument`](crate::Argument)s, which are text or a whole number. There is
//! no request that carries a path to an executable, a script, a shell line or
//! anything that could be shaped into one, because there is no field for one to
//! arrive in.
//!
//! This crate also never makes a call. It hands back the name and the values,
//! and a turn puts them through the registry — so the type that means
//! *validated* is still made in one place, by the crate that owns the list.

use serde::{Deserialize, Serialize};

use crate::argument::Argument;

/// Everything a client can put on the wire.
///
/// `pub(crate)`: see this file's header. A public version of this type would be
/// a value holding *approve* that an agent's door could hand back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum Asked {
    /// A read, which answers inside the turn.
    Read {
        /// The name of the verb, as it was asked for.
        verb: String,
        /// What was given for each of its arguments.
        given: Vec<Argument>,
    },
    /// A change, to be put to the person in one sentence.
    Propose {
        /// The name of the verb, as it was asked for.
        verb: String,
        /// What was given for each of its arguments.
        given: Vec<Argument>,
    },
    /// A question put to a model.
    ///
    /// The question and nothing else. **Where it is answered is not on the
    /// wire**: ADR 0008 says the person decides that, and a request that named
    /// a place would be an agent choosing which machine its question is
    /// answered on. It arrives at the turn's door the way the grants do.
    Ask {
        /// What is being asked.
        question: String,
    },
    /// The person approved the change waiting under this number.
    Approve {
        /// The number they answered.
        number: u64,
    },
    /// The person said no to the change waiting under this number.
    Decline {
        /// The number they answered.
        number: u64,
    },
    /// What is waiting for the person to answer.
    ///
    /// The one request item 21b added, and the only one on either door that
    /// asks a question about the turn rather than doing something in it. It
    /// carries nothing: what is waiting is *what this turn has put to this
    /// person*, and a field naming an agent, a number or a moment would be a
    /// way to ask about somebody else's.
    Waiting {},
    /// What is granted has changed, and the daemon should read its own list
    /// again.
    ///
    /// **A knock and not a payload**, which is the whole of why it is safe for
    /// this to be on the wire at all. It carries no grant, no path, no reach and
    /// no duration, so the only thing it can cause is the service re-reading the
    /// file the person's own side has already written — under the same rules
    /// about who may have written it that it reads at start-up. A request that
    /// carried a grant would be a request that widened one, and ADR 0001 §3 says
    /// a grant is made by a person picking a folder and by nothing else.
    ///
    /// It carries nothing on the way in for [`Asked::Waiting`]'s reason as well:
    /// a field naming a path, an agent or a moment would be a way to say *read
    /// this bit* or *as of then*, and neither is a thing anybody may ask for.
    Granted {},
    /// The person proposes pairing with a machine discovery found.
    ///
    /// **It carries nothing that could name a machine discovery did not
    /// measure**: the identity is what a machine is found by, and the daemon
    /// looks for it on the network at the moment rather than dialling anything
    /// typed. There is no field for an address, so nothing here can point the
    /// machine anywhere (ADR 0003). `may` is the enumerated list, as the wire
    /// spells each arm, and `seconds` is the duration, stated where the
    /// pairing is made rather than hidden in a constant.
    Pair {
        /// The other machine, by its identity.
        machine: String,
        /// What it would be permitted to ask for.
        may: Vec<String>,
        /// How long the pairing would last.
        seconds: u64,
    },
    /// The person confirms the proposal waiting with a machine, having been
    /// shown the code.
    ///
    /// The code travels back so that what is confirmed is what was shown: a
    /// confirmation for a code that does not match the proposal waiting is
    /// refused, which is what stops a shell confirming whatever happens to be
    /// waiting.
    ConfirmPairing {
        /// The other machine, by its identity.
        machine: String,
        /// The six digits the person compared with the other person's.
        code: String,
    },
    /// The person revokes the pairing with a machine, in one action.
    RevokePairing {
        /// The other machine, by its identity.
        machine: String,
    },
    /// What is paired, and what is waiting to be.
    ///
    /// Carries nothing, for [`Asked::Waiting`]'s reason: the list is this
    /// machine's, and a field would be a way to ask about another's.
    Pairings {},
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_capability::Given;

    /// The first seven, as they are written on the wire; the four about pairing
    /// are below.
    #[test]
    fn the_seven_read_back_as_what_was_written() {
        let read: Asked =
            serde_json::from_str(r#"{"read":{"verb":"list_folder","given":[]}}"#).unwrap();
        assert!(matches!(read, Asked::Read { .. }));

        let propose: Asked = serde_json::from_str(
            r#"{"propose":{"verb":"rename_file","given":[{"named":"name","is":"a.pdf"}]}}"#,
        )
        .unwrap();
        assert_eq!(
            propose,
            Asked::Propose {
                verb: "rename_file".to_owned(),
                given: vec![Argument::of("name", Given::text("a.pdf"))],
            }
        );

        let ask: Asked = serde_json::from_str(r#"{"ask":{"question":"how many?"}}"#).unwrap();
        assert_eq!(
            ask,
            Asked::Ask {
                question: "how many?".to_owned()
            }
        );

        let approve: Asked = serde_json::from_str(r#"{"approve":{"number":7}}"#).unwrap();
        assert_eq!(approve, Asked::Approve { number: 7 });

        let decline: Asked = serde_json::from_str(r#"{"decline":{"number":7}}"#).unwrap();
        assert_eq!(decline, Asked::Decline { number: 7 });

        let waiting: Asked = serde_json::from_str(r#"{"waiting":{}}"#).unwrap();
        assert_eq!(waiting, Asked::Waiting {});

        let granted: Asked = serde_json::from_str(r#"{"granted":{}}"#).unwrap();
        assert_eq!(granted, Asked::Granted {});
    }

    /// **Saying that what is granted has changed carries no grant.** It is a
    /// knock rather than a payload: every one of these is a way of saying *and
    /// here is what to grant*, and none of them is a request. What the daemon
    /// does with the knock is read the person's own file again.
    #[test]
    fn saying_what_is_granted_changed_cannot_carry_a_grant() {
        for message in [
            r#"{"granted":{"folder":"/home/anna/Invoices"}}"#,
            r#"{"granted":{"agent":"@files"}}"#,
            r#"{"granted":{"seconds":3600}}"#,
            r#"{"granted":{"revoke":7}}"#,
            r#"{"granted":{"since":1760000000}}"#,
        ] {
            assert!(serde_json::from_str::<Asked>(message).is_err(), "{message}");
        }
    }

    /// **Asking what is waiting asks about nothing but this turn.** A field
    /// naming an agent, a number or a moment would be a request to look at
    /// somebody else's changes, so there is no field at all.
    #[test]
    fn asking_what_is_waiting_cannot_name_whose() {
        for message in [
            r#"{"waiting":{"agent":"@files"}}"#,
            r#"{"waiting":{"turn":3}}"#,
            r#"{"waiting":{"since":1760000000}}"#,
            r#"{"waiting":{"number":7}}"#,
        ] {
            assert!(serde_json::from_str::<Asked>(message).is_err(), "{message}");
        }
    }

    /// The four about pairing, as they are written on the wire.
    #[test]
    fn the_four_about_pairing_read_back_as_what_was_written() {
        let pair: Asked = serde_json::from_str(
            r#"{"pair":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","may":["models"],"seconds":86400}}"#,
        )
        .unwrap();
        assert_eq!(
            pair,
            Asked::Pair {
                machine: "0f1e2d3c4b5a69788796a5b4c3d2e1f0".to_owned(),
                may: vec!["models".to_owned()],
                seconds: 86_400,
            }
        );
        let confirm: Asked = serde_json::from_str(
            r#"{"confirm-pairing":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","code":"482910"}}"#,
        )
        .unwrap();
        assert!(matches!(confirm, Asked::ConfirmPairing { ref code, .. } if code == "482910"));
        let revoke: Asked = serde_json::from_str(
            r#"{"revoke-pairing":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0"}}"#,
        )
        .unwrap();
        assert!(matches!(revoke, Asked::RevokePairing { .. }));
        let pairings: Asked = serde_json::from_str(r#"{"pairings":{}}"#).unwrap();
        assert_eq!(pairings, Asked::Pairings {});
    }

    /// **A proposal cannot name where a machine is.** The identity is what
    /// discovery finds a machine by and the daemon looks for it at the moment;
    /// an address, a port or a name would be a way to point the machine at
    /// something discovery never measured (ADR 0003). And a confirmation
    /// cannot be sent without the code it is confirming, nor with a number in
    /// place of one.
    #[test]
    fn a_proposal_cannot_name_an_address_and_a_confirmation_needs_its_code() {
        for message in [
            r#"{"pair":{"machine":"m","may":["models"],"seconds":60,"address":"192.168.1.20"}}"#,
            r#"{"pair":{"machine":"m","may":["models"],"seconds":60,"port":7610}}"#,
            r#"{"pair":{"machine":"m","may":["models"],"seconds":60,"name":"the studio"}}"#,
            r#"{"pair":{"machine":"m","may":["models"]}}"#,
            r#"{"pair":{"machine":"m","seconds":60}}"#,
            r#"{"confirm-pairing":{"machine":"m"}}"#,
            r#"{"confirm-pairing":{"machine":"m","code":482910}}"#,
            r#"{"revoke-pairing":{}}"#,
            r#"{"pairings":{"machine":"m"}}"#,
        ] {
            assert!(serde_json::from_str::<Asked>(message).is_err(), "{message}");
        }
    }

    /// **There is no twelfth.** A name that is not one of the eleven has nowhere
    /// to land, which is the shape law 2 takes at this boundary: a caller
    /// cannot invent a request any more than it can invent a verb.
    #[test]
    fn a_request_that_is_not_one_of_the_eleven_is_not_a_request() {
        for message in [
            r#"{"run":{"command":"rm -rf /"}}"#,
            r#"{"exec":{"verb":"sh","given":[]}}"#,
            r#"{"grant":{"path":"/"}}"#,
            r#"{"begin":{"agent":"@files"}}"#,
            r#"{"end":{}}"#,
        ] {
            assert!(serde_json::from_str::<Asked>(message).is_err(), "{message}");
        }
    }

    /// **A question names no place.** ADR 0008 puts that decision with the
    /// person, so there is no field for a request to put one in.
    #[test]
    fn a_question_cannot_name_where_it_is_answered() {
        for message in [
            r#"{"ask":{"question":"how many?","of":"a-provider"}}"#,
            r#"{"ask":{"question":"how many?","model":"mistral"}}"#,
            r#"{"ask":{"question":"how many?","where":"this-machine"}}"#,
        ] {
            assert!(serde_json::from_str::<Asked>(message).is_err(), "{message}");
        }
    }

    /// **The same argument twice survives the wire**, so that the refusal in
    /// `alo-capability` is reachable. A JSON object would have lost one of them
    /// before anything could refuse it.
    #[test]
    fn an_argument_given_twice_arrives_twice() {
        let both: Asked = serde_json::from_str(
            r#"{"read":{"verb":"read_file","given":[{"named":"file","is":"/a"},{"named":"file","is":"/b"}]}}"#,
        )
        .unwrap();
        assert_eq!(
            both,
            Asked::Read {
                verb: "read_file".to_owned(),
                given: vec![
                    Argument::of("file", Given::text("/a")),
                    Argument::of("file", Given::text("/b")),
                ],
            }
        );
    }

    /// A field nobody declared is refused rather than ignored, at the request
    /// as well as at the argument.
    #[test]
    fn a_field_nobody_declared_is_refused() {
        for message in [
            r#"{"read":{"verb":"list_folder","given":[],"as":"root"}}"#,
            r#"{"approve":{"number":7,"because":"i said so"}}"#,
        ] {
            assert!(serde_json::from_str::<Asked>(message).is_err(), "{message}");
        }
    }

    /// A number is a whole number, and a negative one is not a proposal.
    #[test]
    fn a_proposal_is_answered_by_a_whole_number() {
        for message in [
            r#"{"approve":{"number":-1}}"#,
            r#"{"approve":{"number":1.5}}"#,
            r#"{"approve":{"number":"7"}}"#,
        ] {
            assert!(serde_json::from_str::<Asked>(message).is_err(), "{message}");
        }
    }
}
