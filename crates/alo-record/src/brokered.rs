//! What the privileged broker did with a request, kept like everything else.
//!
//! ADR 0001 §2 puts printers, the network, updates and storage behind a broker
//! with its own closed verb list, and `crates/alo-broker` is that broker. It
//! decides one thing — that a request is one of its verbs, exactly, from the
//! agent's service, under an approval a turn issued and nobody has spent — and
//! every answer it gives is written down here **before** it is given, the
//! refusals as carefully as the requests it hands on.
//!
//! # Why the broker's entries are their own kind
//!
//! Every other kind of entry about a verb is made from an `alo_capability`
//! value — a call that validated, an authorisation the grants gave. The broker
//! holds neither, and must not: the turn in `alo-agentd` made the call, asked
//! the person, and wrote its own `ran` entry when the approval was spent. What
//! the broker adds is the other half of the same moment on the other side of a
//! privilege boundary, so a review can put the two side by side by approval
//! number and see that nothing crossed which a person had not approved.
//!
//! **It names no agent.** The broker is told an approval, not whose grants it
//! was proposed under; that name is on the turn's entry for the same approval,
//! and writing a second one here would be the broker vouching for something it
//! was never shown.
//!
//! **Why a request was refused is a closed list, not a sentence.** Every reason
//! the broker has is one of [`AtTheBroker`]'s, it is kept as its tag, and the
//! words a person reads for it are chosen where the record is read — so the
//! record carries no English and no text a caller wrote, apart from a verb name
//! that went through [`Line`].
//!
//! Additive, and `format` stays `1` — `docs/contracts/record-file.md`'s *a new
//! kind of `happened` is additive* is the decision.

use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::entry::Entry;
use crate::happened::Happened;
use crate::line::Line;

/// Why the broker refused a request, as the record keeps it.
///
/// In the order the broker asks: who is at the door, whether the line is a
/// request, whether it names one of the broker's verbs exactly, and whether the
/// approval it carries is genuine, unspent and still fresh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AtTheBroker {
    /// The kernel said the caller is not the agent's service.
    NotTheAgentService,
    /// What arrived is not a request at all.
    NotARequest,
    /// It is a request, and not for one of the broker's verbs with arguments of
    /// exactly the shapes that verb takes.
    NotOneOfItsVerbs,
    /// The approval it carries was not issued for this verb and these arguments.
    NotApproved,
    /// The approval was genuine and has already been spent.
    ApprovalSpent,
    /// The approval was genuine and is too old, or claims a moment that has not
    /// happened yet.
    ApprovalLapsed,
    /// Every step above passed, and the machine could not carry it out.
    NotCarried,
}

impl Entry {
    /// The broker handed one of its verbs on to be carried out, under an
    /// approval it verified and has now spent.
    ///
    /// `verb` is the name on the broker's own closed list; `from_approval` is
    /// the number the turn's token was issued for, which is the same number on
    /// the turn's own entry.
    #[must_use]
    pub fn handed_on_by_the_broker(verb: &str, from_approval: u64, at: SystemTime) -> Self {
        Self::new(
            at,
            Happened::Brokered {
                verb: Some(Line::of(verb)),
                from_approval: Some(from_approval),
                refused: None,
            },
        )
    }

    /// The broker refused a request.
    ///
    /// `verb` is absent when the request was never read as far as a verb — a
    /// caller the kernel said was somebody else is not listened to at all.
    /// `from_approval` is present only when the approval was genuine: a forged
    /// token names no approval anybody gave, and keeping the number it claimed
    /// would put a claim into the record as a fact.
    #[must_use]
    pub fn refused_by_the_broker(
        verb: Option<&str>,
        from_approval: Option<u64>,
        why: AtTheBroker,
        at: SystemTime,
    ) -> Self {
        Self::new(
            at,
            Happened::Brokered {
                verb: verb.map(Line::of),
                from_approval,
                refused: Some(why),
            },
        )
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::explain::{Asking, Only};
    use crate::record::Record;
    use std::time::Duration;

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// **A request handed on names its approval and nobody's authority**, and
    /// is not counted as something stopped.
    #[test]
    fn a_request_handed_on_names_the_approval_and_no_agent() {
        let entry = Entry::handed_on_by_the_broker("printers.add", 7, noon());
        assert_eq!(entry.agent(), None);
        assert_eq!(entry.happened().from_approval(), Some(7));
        assert!(!entry.happened().was_stopped());
        assert!(!entry.happened().caused_egress());
        assert_eq!(entry.at(), noon());
    }

    /// **A refusal is found by the question a review asks**: refusals only, and
    /// by the approval it claimed when that approval was genuine.
    #[test]
    fn a_refusal_is_a_refusal_and_is_found_as_one() {
        let mut record = Record::default();
        record.keep(Entry::refused_by_the_broker(
            Some("printers.add"),
            Some(7),
            AtTheBroker::ApprovalSpent,
            noon(),
        ));
        record.keep(Entry::refused_by_the_broker(
            None,
            None,
            AtTheBroker::NotTheAgentService,
            noon(),
        ));
        record.keep(Entry::handed_on_by_the_broker("printers.add", 7, noon()));

        let refusals = Asking::anything().only(Only::Refusals);
        assert_eq!(record.answering(&refusals).count(), 2);
        let seven = Asking::anything().from_approval(7);
        assert_eq!(record.answering(&seven).count(), 2);
    }

    /// **It reads back as it was written**, with the reason as its tag rather
    /// than as a sentence in anybody's language.
    #[test]
    fn it_reads_back_as_written_with_the_reason_as_a_tag() {
        let written = Entry::refused_by_the_broker(
            Some("network.radio"),
            None,
            AtTheBroker::NotApproved,
            noon(),
        );
        let json = serde_json::to_string(&written).unwrap();
        assert!(json.contains("\"not-approved\""), "{json}");
        assert!(json.contains("\"brokered\""), "{json}");
        let read: Result<Entry, _> = serde_json::from_str(&json);
        assert_eq!(read.ok(), Some(written));
    }
}
