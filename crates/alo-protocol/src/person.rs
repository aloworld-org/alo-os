//! What a person's shell sends, on behalf of the person in front of it.
//!
//! Eleven requests. Two of them are the same act — answering a change that was
//! put to them in one sentence — and ADR 0001 §5 says a person approves a
//! sentence rather than a session, so there is nothing here that approves more
//! than one thing, nothing that approves everything from an agent, and nothing
//! that stands until it is revoked.
//!
//! The third is the one that makes the other two usable: what is waiting.
//!
//! # The four about pairing are the person's, and carry no address
//!
//! ADR 0003: a pairing is made by two people, each on their own machine, and
//! it is enumerated, revocable in one action and expiring. So proposing,
//! confirming and revoking one are on this door and refused on the agent's in
//! the same words as an approval — an agent that could pair its own machine
//! with another would be an agent widening what may ask it. What a proposal
//! carries is the other machine's **identity**, the enumerated list and the
//! duration, and nothing else: the identity is what discovery finds a machine
//! by, and the daemon looks for it on the network at the moment. There is no
//! field for an address, a port or a name, so nothing on this wire can point
//! the machine at something discovery never measured. A confirmation carries
//! the code the person was shown, so that what is confirmed is what was shown
//! (ADR 0031).
//!
//! # Which paired machine answers is the person's too
//!
//! ADR 0008 puts where a question is answered with the person, and a machine
//! this one is paired with is one of the places. `choose-machine-to-answer`
//! carries the other machine's **identity** and nothing else — no address, for
//! the pairing requests' reason, and no model, because which model answers
//! there is that machine's person's setting. The daemon holds the choice to
//! the pairings it keeps before anything is written, and an agent sending it
//! is refused in the same words as an approval: an agent that could choose
//! where questions go would be choosing where its own questions leave for.
//!
//! # What a paired machine is called is the person's, and decides nothing
//!
//! `name-machine` gives a machine this one is paired with a name, by its
//! identity, and `clear-machine-name` takes the name away. The name is what the
//! person reads — on the list, on the indicator, in the record — and nothing
//! else: no request finds, dials or proves a machine by one, which is why every
//! other request here still names a machine by its identity, and the name never
//! crosses to the other machine (ADR 0003). An agent sending either is refused in
//! the words an approval gets: an agent that could name a machine could put one
//! machine's name on another machine's evidence.
//!
//! # Which workspaces are nearby is the person's to be shown, and reaches none
//!
//! `workspaces` asks which self-hosted workspaces discovery finds on the local
//! network at the moment, and carries nothing: **no address**, because a
//! workspace is found rather than configured, and a field for one would be the
//! DNS step the promise removes arriving as a text box. What comes back is
//! each workspace with the address discovery measured, and finding one confers
//! nothing (ADR 0003) — there is no request on either door that reaches a
//! workspace because it was found. An agent asking is refused in the words an
//! approval gets: the network around the person is the person's to be shown.
//!
//! # Opening a found workspace is the person's act, by identity alone
//!
//! `open-workspace` names a workspace **by the identity it was found by** and
//! carries nothing else. The daemon looks around on the link at that moment and
//! answers with the one address that workspace answered from then — never one
//! a shell kept from a list that has since aged, which would be a typed address
//! by another route. It hands that address to the person's session and dials
//! nothing itself; what connects is the workspace client, under the person's
//! own account, and nothing here is a pairing or a sign-in (ADR 0003). An agent
//! sending it is refused in the words an approval gets: an agent that could
//! open a workspace would be choosing where the person's session connects.
//!
//! # A number is not a handle
//!
//! Both of these carry a `u64`, and it is deliberately not an
//! `alo_capability::ProposalId`. That type has no public constructor, so one
//! cannot be made off the wire — a number that arrived has to be **found**
//! among the changes actually waiting, and a number naming nothing is
//! `alo_capability::AnswerError::NothingWaiting` rather than something this
//! crate invented. [`FromAPerson::number`] hands the number over and the turn
//! does the finding.
//!
//! It is `alo_capability::GrantId`'s rule met one crate out: an answer to a
//! stale list must fail rather than land somewhere it was not aimed.
//!
//! # `waiting` is a read of the turn, and it is the person's
//!
//! This file used to say there was no `waiting` here *yet*, on the ground that
//! what a shell draws is something the daemon answers rather than something a
//! person asks for. Half of that was right, and the half that was wrong is the
//! half that matters: a daemon answers what it was asked, and a shell that
//! never asked would be drawing whatever it happened to have been told — which
//! is nothing at all if it was started, restarted or attached after the change
//! was proposed.
//!
//! So it is a request, and it is on this door rather than the agent's, because
//! what is waiting is what the **person** has been asked. An agent reaching for
//! it is refused in the same words as an agent trying to approve something,
//! because it is the same fact about the same list.
//! [`crate::ToAPerson::waiting`] is what comes back, and it carries the
//! sentence with every number: a number on its own would be an approval of
//! something nobody read.
//!
//! It carries **nothing** on the way in: no agent, no number, no moment. What
//! is waiting is what this turn has put to this person, and a field would be a
//! way to ask about somebody else's.
//!
//! # `granted` is a knock, and it is the person's for the same reason
//!
//! A grant is made and revoked on the person's side of the machine — a folder
//! picker, and the surface that lists what is granted — and until this request
//! existed a grant made while the daemon was running reached it at the next
//! sign-in. [`FromAPerson::Granted`] is how it reaches a running one, and what
//! it carries is **nothing at all**: not the grant, not the folder, not how long
//! it lasts, not which one was revoked.
//!
//! That is what keeps law 2 and ADR 0001 §5 true of this door. The daemon
//! answers the knock by reading the person's own file again, under the same
//! rules about who may have written it that it reads at start-up, so nothing on
//! the wire has widened anything — a message that arrived from somewhere else
//! could still cause only the re-reading of a file it cannot write. Which door a
//! caller is on is `alo-agentd`'s, and an agent knocking here is refused in the
//! same words as an agent trying to approve something.

use crate::asked::Asked;
use crate::frame;
use crate::refusing::NotUnderstood;

/// One thing a person's shell sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FromAPerson {
    /// They approved the change waiting under this number. Worth exactly one
    /// execution, which is `alo_capability::Approved::redeem`'s and not this
    /// crate's.
    Approve {
        /// The number they answered.
        number: u64,
    },
    /// They said no. Nothing is carried about why, because nothing was asked.
    Decline {
        /// The number they answered.
        number: u64,
    },
    /// What is waiting for them to answer.
    ///
    /// A read of the turn rather than an answer to it — see this file's header
    /// — and one of the two requests on either door that carry nothing at all.
    Waiting,
    /// What is granted has changed, so read it again.
    ///
    /// The other request that carries nothing, and the one where that is the
    /// whole design: a knock rather than a payload. See this file's header.
    Granted,
    /// They propose pairing with a machine discovery found.
    ///
    /// The identity, the enumerated list as the wire spells it, and the
    /// duration — and nothing that could name a machine discovery did not
    /// measure. See this file's header.
    Pair {
        /// The other machine, by its identity.
        machine: String,
        /// What it would be permitted to ask for.
        may: Vec<String>,
        /// How long the pairing would last, in seconds.
        seconds: u64,
    },
    /// They confirm the proposal waiting with a machine, having compared the
    /// code with the other person.
    ConfirmPairing {
        /// The other machine, by its identity.
        machine: String,
        /// The code they were shown.
        code: String,
    },
    /// They revoke the pairing with a machine, in one action.
    RevokePairing {
        /// The other machine, by its identity.
        machine: String,
    },
    /// What is paired, and what is waiting to be.
    Pairings,
    /// They choose a machine this one is paired with to answer their
    /// questions, by its identity.
    ///
    /// No model and no address: see this file's header.
    ChooseMachineToAnswer {
        /// The other machine, by its identity.
        machine: String,
    },
    /// They give a machine this one is paired with a name, by its identity.
    ///
    /// The name decides nothing: see this file's header.
    NameMachine {
        /// The other machine, by its identity.
        machine: String,
        /// What they call it.
        called: String,
    },
    /// They take a machine's name away, by its identity.
    ClearMachineName {
        /// The other machine, by its identity.
        machine: String,
    },
    /// Which workspaces are on the local network at the moment.
    ///
    /// Carries nothing — no address — and reaches nothing: see this file's
    /// header.
    Workspaces,
    /// They open a workspace discovery found, by its identity.
    ///
    /// No address: the daemon measures where it answers at that moment. See
    /// this file's header.
    OpenWorkspace {
        /// Which workspace, by the identity it was found by.
        machine: String,
    },
}

impl FromAPerson {
    /// Read one line as something a person answered.
    ///
    /// # Errors
    /// [`NotUnderstood`] — the envelope's four refusals, and
    /// [`NotUnderstood::NotForAPerson`] for a well-formed request that only an
    /// agent makes during a turn.
    pub fn read(line: &str) -> Result<Self, NotUnderstood> {
        match frame::message(line)? {
            Asked::Approve { number } => Ok(Self::Approve { number }),
            Asked::Decline { number } => Ok(Self::Decline { number }),
            Asked::Waiting {} => Ok(Self::Waiting),
            Asked::Granted {} => Ok(Self::Granted),
            Asked::Pair {
                machine,
                may,
                seconds,
            } => Ok(Self::Pair {
                machine,
                may,
                seconds,
            }),
            Asked::ConfirmPairing { machine, code } => Ok(Self::ConfirmPairing { machine, code }),
            Asked::RevokePairing { machine } => Ok(Self::RevokePairing { machine }),
            Asked::Pairings {} => Ok(Self::Pairings),
            Asked::ChooseMachineToAnswer { machine } => Ok(Self::ChooseMachineToAnswer { machine }),
            Asked::NameMachine { machine, called } => Ok(Self::NameMachine { machine, called }),
            Asked::ClearMachineName { machine } => Ok(Self::ClearMachineName { machine }),
            Asked::Workspaces {} => Ok(Self::Workspaces),
            Asked::OpenWorkspace { machine } => Ok(Self::OpenWorkspace { machine }),
            Asked::Read { .. } | Asked::Propose { .. } | Asked::Ask { .. } => {
                Err(NotUnderstood::NotForAPerson)
            }
        }
    }

    /// This answer as the line that carries it.
    ///
    /// # Errors
    /// A `serde_json::Error`, which an answer cannot cause. See
    /// `frame.rs` for why it is handed back rather than swallowed.
    pub fn written(&self) -> Result<String, serde_json::Error> {
        frame::line(self.clone().into())
    }

    /// The number they answered, for the two that answer one.
    ///
    /// A number and not a handle: see this file's header. Nothing for
    /// [`FromAPerson::Waiting`], which answers no change and asks about all of
    /// them.
    #[must_use]
    pub fn number(&self) -> Option<u64> {
        match self {
            Self::Approve { number } | Self::Decline { number } => Some(*number),
            Self::Waiting
            | Self::Granted
            | Self::Pair { .. }
            | Self::ConfirmPairing { .. }
            | Self::RevokePairing { .. }
            | Self::Pairings
            | Self::ChooseMachineToAnswer { .. }
            | Self::NameMachine { .. }
            | Self::ClearMachineName { .. }
            | Self::Workspaces
            | Self::OpenWorkspace { .. } => None,
        }
    }

    /// Whether this names a machine or takes its name away.
    ///
    /// Told apart for a daemon choosing what to answer them against: a name is
    /// neither a turn's nor the grants file's, and it is answered whether or
    /// not a turn is under way.
    #[must_use]
    pub fn is_about_a_name(&self) -> bool {
        matches!(
            self,
            Self::NameMachine { .. } | Self::ClearMachineName { .. }
        )
    }

    /// Whether this is about a pairing rather than about a turn or the grants.
    ///
    /// The four the local network added, told apart for a daemon choosing
    /// what to answer them against: a pairing is neither a turn's nor the
    /// grants file's, and it is answered whether or not a turn is under way.
    #[must_use]
    pub fn is_about_a_pairing(&self) -> bool {
        matches!(
            self,
            Self::Pair { .. }
                | Self::ConfirmPairing { .. }
                | Self::RevokePairing { .. }
                | Self::Pairings
        )
    }

    /// Whether they said yes.
    #[must_use]
    pub fn is_yes(&self) -> bool {
        matches!(self, Self::Approve { .. })
    }

    /// Whether this asks about the turn rather than answering something in it.
    ///
    /// A convenience for a daemon choosing what to do next: what is waiting is
    /// read off the turn and changes nothing. It is not the only request on this
    /// door that spends no approval — [`FromAPerson::Granted`] spends none
    /// either — but it is the only one that is a **question about the turn**,
    /// and the knock is deliberately not one: it is about the person's own list
    /// of grants, which outlives every turn.
    #[must_use]
    pub fn is_a_question_about_the_turn(&self) -> bool {
        matches!(self, Self::Waiting)
    }
}

impl From<FromAPerson> for Asked {
    fn from(answered: FromAPerson) -> Self {
        match answered {
            FromAPerson::Approve { number } => Self::Approve { number },
            FromAPerson::Decline { number } => Self::Decline { number },
            FromAPerson::Waiting => Self::Waiting {},
            FromAPerson::Granted => Self::Granted {},
            FromAPerson::Pair {
                machine,
                may,
                seconds,
            } => Self::Pair {
                machine,
                may,
                seconds,
            },
            FromAPerson::ConfirmPairing { machine, code } => Self::ConfirmPairing { machine, code },
            FromAPerson::RevokePairing { machine } => Self::RevokePairing { machine },
            FromAPerson::Pairings => Self::Pairings {},
            FromAPerson::ChooseMachineToAnswer { machine } => {
                Self::ChooseMachineToAnswer { machine }
            }
            FromAPerson::NameMachine { machine, called } => Self::NameMachine { machine, called },
            FromAPerson::ClearMachineName { machine } => Self::ClearMachineName { machine },
            FromAPerson::Workspaces => Self::Workspaces {},
            FromAPerson::OpenWorkspace { machine } => Self::OpenWorkspace { machine },
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

    /// The four, off the wire.
    #[test]
    fn the_four_a_persons_shell_may_send_read_back() {
        let yes = FromAPerson::read(r#"{"format":1,"asks":{"approve":{"number":7}}}"#).unwrap();
        assert_eq!(yes, FromAPerson::Approve { number: 7 });
        assert!(yes.is_yes());
        assert_eq!(yes.number(), Some(7));
        assert!(!yes.is_a_question_about_the_turn());

        let no = FromAPerson::read(r#"{"format":1,"asks":{"decline":{"number":7}}}"#).unwrap();
        assert!(!no.is_yes());
        assert_eq!(no.number(), Some(7));

        let waiting = FromAPerson::read(r#"{"format":1,"asks":{"waiting":{}}}"#).unwrap();
        assert_eq!(waiting, FromAPerson::Waiting);
        assert!(waiting.is_a_question_about_the_turn());
        assert_eq!(waiting.number(), None);
        assert!(!waiting.is_yes());

        let granted = FromAPerson::read(r#"{"format":1,"asks":{"granted":{}}}"#).unwrap();
        assert_eq!(granted, FromAPerson::Granted);
        assert_eq!(granted.number(), None);
        assert!(!granted.is_yes());
        assert!(
            !granted.is_a_question_about_the_turn(),
            "saying what is granted has changed is not a question about the turn"
        );
    }

    /// **Saying what is granted has changed answers no change and carries no
    /// grant.** It spends no approval, so it must not look like one: a number
    /// here would be a shell answering a question by saying something else.
    #[test]
    fn saying_what_is_granted_changed_answers_nothing_and_grants_nothing() {
        for message in [
            r#"{"format":1,"asks":{"granted":{"number":7}}}"#,
            r#"{"format":1,"asks":{"granted":{"folder":"/home/anna/Invoices"}}}"#,
            r#"{"format":1,"asks":{"granted":{"agent":"@files","seconds":3600}}}"#,
        ] {
            assert_eq!(
                FromAPerson::read(message),
                Err(NotUnderstood::NotReadable),
                "{message}"
            );
        }
    }

    /// **A person's side is not a way in for a verb.** The division goes both
    /// ways: a shell is where a person answers, and a request that runs
    /// something arriving on it is refused rather than carried out on their
    /// behalf.
    #[test]
    fn a_request_an_agent_makes_is_not_something_a_person_sends() {
        for message in [
            r#"{"format":1,"asks":{"read":{"verb":"list_folder","given":[]}}}"#,
            r#"{"format":1,"asks":{"propose":{"verb":"rename_file","given":[]}}}"#,
            r#"{"format":1,"asks":{"ask":{"question":"how many?"}}}"#,
        ] {
            assert_eq!(
                FromAPerson::read(message),
                Err(NotUnderstood::NotForAPerson),
                "{message}"
            );
        }
    }

    /// **Nothing approves more than one thing.** An approval is of one
    /// sentence, so there is no shape on the wire for *approve these*, *approve
    /// everything* or *approve whatever this agent asks next*.
    #[test]
    fn nothing_a_person_sends_approves_more_than_one_change() {
        for message in [
            r#"{"format":1,"asks":{"approve":{"numbers":[7,8]}}}"#,
            r#"{"format":1,"asks":{"approve":{"number":7,"and":8}}}"#,
            r#"{"format":1,"asks":{"approve-all":{"agent":"@files"}}}"#,
            r#"{"format":1,"asks":{"approve":{"agent":"@files"}}}"#,
        ] {
            assert_eq!(
                FromAPerson::read(message),
                Err(NotUnderstood::NotReadable),
                "{message}"
            );
        }
    }

    /// A number off the wire is a number, and finding what it names is the
    /// turn's — so there is no way here to make a handle to something that was
    /// never waiting.
    #[test]
    fn a_number_is_carried_and_not_turned_into_a_handle() {
        let nothing_waiting =
            FromAPerson::read(r#"{"format":1,"asks":{"approve":{"number":9999}}}"#).unwrap();
        assert_eq!(nothing_waiting.number(), Some(9999));
    }

    /// **The four about pairing are a person's**, read off the wire, and none
    /// of them answers a change or asks about the turn.
    #[test]
    fn the_four_about_pairing_are_a_persons_and_answer_no_change() {
        let pair = FromAPerson::read(
            r#"{"format":1,"asks":{"pair":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","may":["models"],"seconds":86400}}}"#,
        )
        .unwrap();
        assert_eq!(
            pair,
            FromAPerson::Pair {
                machine: "0f1e2d3c4b5a69788796a5b4c3d2e1f0".to_owned(),
                may: vec!["models".to_owned()],
                seconds: 86_400,
            }
        );
        let confirm = FromAPerson::read(
            r#"{"format":1,"asks":{"confirm-pairing":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","code":"482910"}}}"#,
        )
        .unwrap();
        let revoke = FromAPerson::read(
            r#"{"format":1,"asks":{"revoke-pairing":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0"}}}"#,
        )
        .unwrap();
        let pairings = FromAPerson::read(r#"{"format":1,"asks":{"pairings":{}}}"#).unwrap();
        assert_eq!(pairings, FromAPerson::Pairings);
        for one in [pair, confirm, revoke, pairings] {
            assert!(one.is_about_a_pairing(), "{one:?}");
            assert_eq!(one.number(), None, "{one:?}");
            assert!(!one.is_yes(), "{one:?}");
            assert!(!one.is_a_question_about_the_turn(), "{one:?}");
        }
        assert!(!FromAPerson::Waiting.is_about_a_pairing());
        assert!(!FromAPerson::Granted.is_about_a_pairing());
    }

    /// **Choosing a machine to answer is the person's, and an agent sending it
    /// is refused in the words an approval gets**: it answers no change, asks
    /// nothing about the turn, and is not one of the four about a pairing.
    #[test]
    fn choosing_a_machine_to_answer_is_a_persons_and_refused_to_an_agent() {
        let line = r#"{"format":1,"asks":{"choose-machine-to-answer":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0"}}}"#;
        let chosen = FromAPerson::read(line).unwrap();
        assert_eq!(
            chosen,
            FromAPerson::ChooseMachineToAnswer {
                machine: "0f1e2d3c4b5a69788796a5b4c3d2e1f0".to_owned(),
            }
        );
        assert_eq!(chosen.number(), None);
        assert!(!chosen.is_yes());
        assert!(!chosen.is_a_question_about_the_turn());
        assert!(!chosen.is_about_a_pairing());
        assert_eq!(
            crate::FromAnAgent::read(line),
            Err(NotUnderstood::NotForAnAgent)
        );
    }

    /// **Naming a machine and clearing its name are a person's, and an agent
    /// sending either is refused in the words an approval gets**: neither
    /// answers a change, asks about the turn, or is about a pairing.
    #[test]
    fn naming_a_machine_is_a_persons_and_refused_to_an_agent() {
        let naming = r#"{"format":1,"asks":{"name-machine":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","called":"the studio machine"}}}"#;
        let clearing = r#"{"format":1,"asks":{"clear-machine-name":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0"}}}"#;
        let named = FromAPerson::read(naming).unwrap();
        assert_eq!(
            named,
            FromAPerson::NameMachine {
                machine: "0f1e2d3c4b5a69788796a5b4c3d2e1f0".to_owned(),
                called: "the studio machine".to_owned(),
            }
        );
        let cleared = FromAPerson::read(clearing).unwrap();
        for one in [&named, &cleared] {
            assert!(one.is_about_a_name(), "{one:?}");
            assert!(!one.is_about_a_pairing(), "{one:?}");
            assert_eq!(one.number(), None);
            assert!(!one.is_yes());
            assert!(!one.is_a_question_about_the_turn());
        }
        assert!(!FromAPerson::Pairings.is_about_a_name());
        for line in [naming, clearing] {
            assert_eq!(
                crate::FromAnAgent::read(line),
                Err(NotUnderstood::NotForAnAgent),
                "{line}"
            );
        }
    }

    /// **Asking which workspaces are nearby is the person's, carries nothing,
    /// and an agent asking is refused in the words an approval gets.**
    #[test]
    fn asking_which_workspaces_are_nearby_is_a_persons_and_refused_to_an_agent() {
        let line = r#"{"format":1,"asks":{"workspaces":{}}}"#;
        let asked = FromAPerson::read(line).unwrap();
        assert_eq!(asked, FromAPerson::Workspaces);
        assert_eq!(asked.number(), None);
        assert!(!asked.is_yes());
        assert!(!asked.is_a_question_about_the_turn());
        assert!(!asked.is_about_a_pairing());
        assert!(!asked.is_about_a_name());
        assert_eq!(
            crate::FromAnAgent::read(line),
            Err(NotUnderstood::NotForAnAgent)
        );
    }

    /// **An address a person types is never dialled as a workspace**: there is
    /// no field on `workspaces` for one, and no request that names a workspace
    /// by an address — each shape is refused as not a request at all.
    #[test]
    fn an_address_typed_as_a_workspace_is_not_a_request() {
        for message in [
            r#"{"format":1,"asks":{"workspaces":{"address":"192.168.1.20:8443"}}}"#,
            r#"{"format":1,"asks":{"workspaces":{"at":"mail.axon.example"}}}"#,
            r#"{"format":1,"asks":{"open-workspace":{"address":"192.168.1.20:8443"}}}"#,
            r#"{"format":1,"asks":{"workspace":{"url":"https://mail.axon.example"}}}"#,
        ] {
            assert_eq!(
                FromAPerson::read(message),
                Err(NotUnderstood::NotReadable),
                "{message}"
            );
            assert_eq!(
                crate::FromAnAgent::read(message),
                Err(NotUnderstood::NotReadable),
                "{message}"
            );
        }
    }

    /// **A proposal cannot name where a machine is**: an address in it is a
    /// message this crate refuses to read, in the same words as any field
    /// nobody declared.
    #[test]
    fn a_proposal_naming_an_address_is_not_a_request() {
        assert_eq!(
            FromAPerson::read(
                r#"{"format":1,"asks":{"pair":{"machine":"m","may":["models"],"seconds":60,"address":"192.168.1.20:7610"}}}"#,
            ),
            Err(NotUnderstood::NotReadable)
        );
    }

    /// A shell and a daemon built from this crate cannot disagree about the
    /// format.
    #[test]
    fn what_a_person_writes_this_crate_reads_back() {
        for answered in [
            FromAPerson::Approve { number: 1 },
            FromAPerson::Decline { number: 2 },
            FromAPerson::Waiting,
            FromAPerson::Granted,
            FromAPerson::Pair {
                machine: "0f1e2d3c4b5a69788796a5b4c3d2e1f0".to_owned(),
                may: vec!["models".to_owned()],
                seconds: 86_400,
            },
            FromAPerson::ConfirmPairing {
                machine: "0f1e2d3c4b5a69788796a5b4c3d2e1f0".to_owned(),
                code: "482910".to_owned(),
            },
            FromAPerson::RevokePairing {
                machine: "0f1e2d3c4b5a69788796a5b4c3d2e1f0".to_owned(),
            },
            FromAPerson::Pairings,
            FromAPerson::ChooseMachineToAnswer {
                machine: "0f1e2d3c4b5a69788796a5b4c3d2e1f0".to_owned(),
            },
            FromAPerson::NameMachine {
                machine: "0f1e2d3c4b5a69788796a5b4c3d2e1f0".to_owned(),
                called: "the studio machine".to_owned(),
            },
            FromAPerson::ClearMachineName {
                machine: "0f1e2d3c4b5a69788796a5b4c3d2e1f0".to_owned(),
            },
            FromAPerson::Workspaces,
            FromAPerson::OpenWorkspace {
                machine: "0f1e2d3c4b5a69788796a5b4c3d2e1f0".to_owned(),
            },
        ] {
            let written = answered.written().unwrap();
            assert_eq!(FromAPerson::read(&written).unwrap(), answered);
        }
    }

    /// **Opening a found workspace is the person's, names it by its identity
    /// alone, and an agent sending it is refused in the words an approval
    /// gets.**
    #[test]
    fn opening_a_workspace_is_a_persons_by_identity_and_refused_to_an_agent() {
        let line = r#"{"format":1,"asks":{"open-workspace":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0"}}}"#;
        let asked = FromAPerson::read(line).unwrap();
        assert_eq!(
            asked,
            FromAPerson::OpenWorkspace {
                machine: "0f1e2d3c4b5a69788796a5b4c3d2e1f0".to_owned()
            }
        );
        assert_eq!(asked.number(), None);
        assert!(!asked.is_yes());
        assert!(!asked.is_a_question_about_the_turn());
        assert!(!asked.is_about_a_pairing());
        assert!(!asked.is_about_a_name());
        assert_eq!(
            crate::FromAnAgent::read(line),
            Err(NotUnderstood::NotForAnAgent)
        );
    }

    /// **A request to open a workspace that carries an address is not a
    /// request**: beside the identity, instead of it, or as a URL or a port —
    /// every shape is refused as unreadable on both doors, so an address a
    /// shell kept or a person typed has nowhere to arrive.
    #[test]
    fn opening_a_workspace_by_an_address_is_not_a_request() {
        for message in [
            r#"{"format":1,"asks":{"open-workspace":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","answers_at":"192.168.1.20:8443"}}}"#,
            r#"{"format":1,"asks":{"open-workspace":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","address":"192.168.1.20"}}}"#,
            r#"{"format":1,"asks":{"open-workspace":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","port":8443}}}"#,
            r#"{"format":1,"asks":{"open-workspace":{"answers_at":"192.168.1.20:8443"}}}"#,
            r#"{"format":1,"asks":{"open-workspace":{"url":"https://mail.axon.example"}}}"#,
            r#"{"format":1,"asks":{"open-workspace":{}}}"#,
        ] {
            assert_eq!(
                FromAPerson::read(message),
                Err(NotUnderstood::NotReadable),
                "{message}"
            );
            assert_eq!(
                crate::FromAnAgent::read(message),
                Err(NotUnderstood::NotReadable),
                "{message}"
            );
        }
    }
}
