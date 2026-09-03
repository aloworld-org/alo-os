//! Everything that can arrive, in one closed list.
//!
//! Five requests, and there is no sixth. What makes this file worth having on
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
//! than two lists that could drift: a sixth request has to be given to one door
//! or the other before this crate will compile, and a request that is not one
//! of the five is not a request at all.
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
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_capability::Given;

    /// The five, as they are written on the wire.
    #[test]
    fn the_five_read_back_as_what_was_written() {
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
    }

    /// **There is no sixth.** A name that is not one of the five has nowhere to
    /// land, which is the shape law 2 takes at this boundary: a caller cannot
    /// invent a request any more than it can invent a verb.
    #[test]
    fn a_request_that_is_not_one_of_the_five_is_not_a_request() {
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
