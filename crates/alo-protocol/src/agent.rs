//! What an agent may ask, during a turn that is already under way.
//!
//! Three requests, and every one of them is something ADR 0001 lets an agent
//! do: read something it was granted, propose a change for the person to
//! approve, and put a question to a model. What comes back is the turn's, and
//! this crate never sees it.
//!
//! # What is deliberately not here
//!
//! **Nothing begins a turn, and nothing ends one.** A turn begins when the
//! person invokes the agent, and what the invocation offered — the window in
//! front of them, the text they had selected, the document they had open — is
//! answered by the compositor at that moment (ADR 0001 §4). A request that
//! carried a context would be an agent handing itself the document it wanted a
//! grant over, which is the one thing *context is offered, never watched* exists
//! to prevent. So there is no `begin` on this list and no `end`: the turn is
//! the connection's, and both ends of it are somebody else's act.
//!
//! **Nothing approves anything.** That is [`crate::person`], and the division
//! is argued in [`crate::asked`].
//!
//! **Nothing names a turn.** A number identifying which turn a request belongs
//! to would be a number an agent could change, and there is no field for one:
//! which turn a message is part of is answered by the connection it arrived on,
//! and that is `alo-agentd`'s.
//!
//! **Nothing names a moment.** Every door in `alo-turn` takes `now`, and it is
//! the machine's clock rather than a value off the wire — a request that named
//! the moment could revive a grant that had expired an hour ago.
//!
//! # What this hands back is what the registry takes
//!
//! [`FromAnAgent::given`] answers with exactly what
//! `alo_capability::Verbs::call` wants, and this crate deliberately does not
//! call it. The type that means *validated* is made by the crate that owns the
//! closed list of verbs, once, and a protocol that made one would be a second
//! place where law 2 has to be got right.

use alo_capability::Given;

use crate::argument::Argument;
use crate::asked::Asked;
use crate::frame;
use crate::refusing::NotUnderstood;

/// One thing an agent asked for during a turn.
///
/// There are three, and a fourth is a change to what an agent can express —
/// which belongs in ADR 0001 and in `docs/contracts/daemon-protocol.md` before
/// it belongs here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FromAnAgent {
    /// A read, which answers inside the turn.
    Read {
        /// The name of the verb, as it was asked for. Nothing has looked it up.
        verb: String,
        /// What was given for each argument. Nothing has validated any of it.
        given: Vec<Argument>,
    },
    /// A change, to be put to the person in one sentence.
    Propose {
        /// The name of the verb, as it was asked for. Nothing has looked it up.
        verb: String,
        /// What was given for each argument. Nothing has validated any of it.
        given: Vec<Argument>,
    },
    /// A question for a model. Where it is answered is the person's (ADR 0008)
    /// and is not on the wire.
    Ask {
        /// What is being asked.
        question: String,
    },
}

impl FromAnAgent {
    /// Read one line as something an agent asked for.
    ///
    /// # Errors
    /// [`NotUnderstood`] — the envelope's four refusals, and
    /// [`NotUnderstood::NotForAnAgent`] for a well-formed request that only a
    /// person makes.
    pub fn read(line: &str) -> Result<Self, NotUnderstood> {
        match frame::message(line)? {
            Asked::Read { verb, given } => Ok(Self::Read { verb, given }),
            Asked::Propose { verb, given } => Ok(Self::Propose { verb, given }),
            Asked::Ask { question } => Ok(Self::Ask { question }),
            Asked::Approve { .. } | Asked::Decline { .. } => Err(NotUnderstood::NotForAnAgent),
        }
    }

    /// This request as the line that carries it.
    ///
    /// # Errors
    /// A `serde_json::Error`, which a request cannot cause. See
    /// [`crate::frame`] for why it is handed back rather than swallowed.
    pub fn written(&self) -> Result<String, serde_json::Error> {
        frame::line(self.clone().into())
    }

    /// The name of the verb, for the two requests that name one.
    ///
    /// As it was written. Looking it up is `alo_capability::Verbs::call`'s, and
    /// so is trimming it.
    #[must_use]
    pub fn verb(&self) -> Option<&str> {
        match self {
            Self::Read { verb, .. } | Self::Propose { verb, .. } => Some(verb),
            Self::Ask { .. } => None,
        }
    }

    /// What was given, in the shape `alo_capability::Verbs::call` takes.
    ///
    /// Duplicates and all: an argument named twice arrives twice, so that
    /// `alo_capability::CallError::SameArgumentTwice` is reachable rather than
    /// having been decided by a JSON reader. [`crate::argument`] is where that
    /// is argued.
    #[must_use]
    pub fn given(&self) -> Vec<(&str, Given)> {
        match self {
            Self::Read { given, .. } | Self::Propose { given, .. } => given
                .iter()
                .map(|argument| (argument.named(), argument.given().clone()))
                .collect(),
            Self::Ask { .. } => Vec::new(),
        }
    }

    /// What was asked of a model, for the one request that asks anything.
    #[must_use]
    pub fn question(&self) -> Option<&str> {
        match self {
            Self::Ask { question } => Some(question),
            Self::Read { .. } | Self::Propose { .. } => None,
        }
    }

    /// Whether this is a change, which waits for a person, rather than a read,
    /// which answers inside the turn.
    ///
    /// A convenience for a daemon choosing a door, and **not** the decision:
    /// `alo_capability::Authorised::read` refuses a change and
    /// `alo_capability::Proposal::checked` refuses a read, whatever this says.
    /// A protocol that decided it would be a second answer to ADR 0001 §5.
    #[must_use]
    pub fn waits_for_a_person(&self) -> bool {
        matches!(self, Self::Propose { .. })
    }
}

impl From<FromAnAgent> for Asked {
    fn from(asked: FromAnAgent) -> Self {
        match asked {
            FromAnAgent::Read { verb, given } => Self::Read { verb, given },
            FromAnAgent::Propose { verb, given } => Self::Propose { verb, given },
            FromAnAgent::Ask { question } => Self::Ask { question },
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

    /// The three, off the wire.
    #[test]
    fn the_three_an_agent_may_ask_read_back() {
        let read = FromAnAgent::read(
            r#"{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/invoices"}]}}}"#,
        )
        .unwrap();
        assert_eq!(read.verb(), Some("list_folder"));
        assert_eq!(
            read.given(),
            vec![("folder", Given::text("/home/anna/invoices"))]
        );
        assert!(!read.waits_for_a_person());

        let propose = FromAnAgent::read(
            r#"{"format":1,"asks":{"propose":{"verb":"rename_file","given":[]}}}"#,
        )
        .unwrap();
        assert!(propose.waits_for_a_person());

        let ask =
            FromAnAgent::read(r#"{"format":1,"asks":{"ask":{"question":"how many?"}}}"#).unwrap();
        assert_eq!(ask.question(), Some("how many?"));
        assert_eq!(ask.verb(), None);
        assert!(ask.given().is_empty());
    }

    /// **An agent cannot approve its own change.** The most important refusal
    /// in this crate: a socket where the side that proposed a change could also
    /// answer it would make ADR 0001 §5 true of the capability model and false
    /// of the door in front of it.
    #[test]
    fn an_agent_cannot_answer_a_question_that_was_put_to_a_person() {
        for message in [
            r#"{"format":1,"asks":{"approve":{"number":7}}}"#,
            r#"{"format":1,"asks":{"decline":{"number":7}}}"#,
        ] {
            assert_eq!(
                FromAnAgent::read(message),
                Err(NotUnderstood::NotForAnAgent),
                "{message}"
            );
        }
    }

    /// **Nothing an agent sends begins a turn, ends one, or names one.** Every
    /// one of these is a request somebody might reasonably expect to exist, and
    /// the reason each does not is in this file's header.
    #[test]
    fn nothing_an_agent_sends_begins_a_turn_names_one_or_names_a_moment() {
        for message in [
            r#"{"format":1,"asks":{"begin":{"agent":"@files"}}}"#,
            r#"{"format":1,"asks":{"end":{}}}"#,
            r#"{"format":1,"asks":{"read":{"verb":"list_folder","given":[],"turn":3}}}"#,
            r#"{"format":1,"asks":{"read":{"verb":"list_folder","given":[],"now":1760000000}}}"#,
            r#"{"format":1,"asks":{"read":{"verb":"list_folder","given":[],"context":"/home/anna/a.pdf"}}}"#,
        ] {
            assert_eq!(
                FromAnAgent::read(message),
                Err(NotUnderstood::NotReadable),
                "{message}"
            );
        }
    }

    /// **Nothing here carries a command**, and it is the absence of a field
    /// rather than a check: law 2 at the one place a caller can reach.
    #[test]
    fn no_request_carries_something_that_could_be_run() {
        for message in [
            r#"{"format":1,"asks":{"read":{"verb":"list_folder","given":[],"then":"rm -rf /"}}}"#,
            r#"{"format":1,"asks":{"run":{"command":"/bin/sh"}}}"#,
            r#"{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":{"run":"sh"}}]}}}"#,
        ] {
            assert_eq!(
                FromAnAgent::read(message),
                Err(NotUnderstood::NotReadable),
                "{message}"
            );
        }
    }

    /// A verb's name arriving as text that happens to look like a command is
    /// still only a name: it is looked up on the closed list, and the list is
    /// what refuses it. This crate does not pretend to be that check.
    #[test]
    fn a_verb_name_is_carried_and_not_judged_here() {
        let asked =
            FromAnAgent::read(r#"{"format":1,"asks":{"read":{"verb":"/bin/sh","given":[]}}}"#)
                .unwrap();
        assert_eq!(asked.verb(), Some("/bin/sh"));
    }

    /// **An argument named twice arrives twice**, so the refusal for it is
    /// reachable in the crate that owns the verbs.
    #[test]
    fn an_argument_named_twice_reaches_the_registry_twice() {
        let asked = FromAnAgent::read(
            r#"{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/a"},{"named":"file","is":"/b"}]}}}"#,
        )
        .unwrap();
        assert_eq!(
            asked.given(),
            vec![("file", Given::text("/a")), ("file", Given::text("/b"))]
        );
    }

    /// A client and a daemon built from this crate cannot disagree about the
    /// format, because the writing and the reading are the same list.
    #[test]
    fn what_an_agent_writes_this_crate_reads_back() {
        for asked in [
            FromAnAgent::Read {
                verb: "list_folder".to_owned(),
                given: vec![Argument::of("folder", Given::text("/home/anna"))],
            },
            FromAnAgent::Propose {
                verb: "rename_file".to_owned(),
                given: vec![Argument::of("name", Given::text("a.pdf"))],
            },
            FromAnAgent::Ask {
                question: "how many invoices are unpaid?".to_owned(),
            },
        ] {
            let written = asked.written().unwrap();
            assert_eq!(FromAnAgent::read(&written).unwrap(), asked);
        }
    }
}
