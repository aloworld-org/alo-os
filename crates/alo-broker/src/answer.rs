//! What the door says back: one line, and never a sentence.
//!
//! `carried`, `refused <why>`, or `not-kept`. The caller is `alo-agentd`, which
//! has a person and a language in front of it; the broker has neither, so what
//! crosses back is a word from a closed list and the turn says what it means.
//! The reasons are exactly `alo_record::AtTheBroker`'s, written as the record
//! writes them, so the answer and the entry for one request cannot disagree.

use alo_record::AtTheBroker;

/// What the door answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    /// The verb was handed on, and carried out.
    Carried,
    /// It was refused, and this is why. The refusal has been written down.
    Refused(AtTheBroker),
    /// Nothing could be written down, so nothing was carried out — and nothing
    /// will be until this broker is started again with somewhere to keep its
    /// record.
    NotKept,
}

/// Every reason, in the order `alo_record::AtTheBroker` declares them.
const EVERY_REASON: [AtTheBroker; 7] = [
    AtTheBroker::NotTheAgentService,
    AtTheBroker::NotARequest,
    AtTheBroker::NotOneOfItsVerbs,
    AtTheBroker::NotApproved,
    AtTheBroker::ApprovalSpent,
    AtTheBroker::ApprovalLapsed,
    AtTheBroker::NotCarried,
];

/// A reason, as the record's tag spells it.
const fn spelt(why: AtTheBroker) -> &'static str {
    match why {
        AtTheBroker::NotTheAgentService => "not-the-agent-service",
        AtTheBroker::NotARequest => "not-a-request",
        AtTheBroker::NotOneOfItsVerbs => "not-one-of-its-verbs",
        AtTheBroker::NotApproved => "not-approved",
        AtTheBroker::ApprovalSpent => "approval-spent",
        AtTheBroker::ApprovalLapsed => "approval-lapsed",
        AtTheBroker::NotCarried => "not-carried",
    }
}

impl Answer {
    /// The line, without its newline.
    #[must_use]
    pub fn written(&self) -> String {
        match self {
            Self::Carried => "carried".to_owned(),
            Self::Refused(why) => format!("refused {}", spelt(*why)),
            Self::NotKept => "not-kept".to_owned(),
        }
    }

    /// Read what the door answered, or nothing if it is not an answer.
    #[must_use]
    pub fn read(line: &str) -> Option<Self> {
        match line {
            "carried" => Some(Self::Carried),
            "not-kept" => Some(Self::NotKept),
            _ => {
                let why = line.strip_prefix("refused ")?;
                EVERY_REASON
                    .into_iter()
                    .find(|reason| spelt(*reason) == why)
                    .map(Self::Refused)
            }
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
    use alo_record::Entry;
    use std::time::SystemTime;

    /// **Every answer reads back as written.**
    #[test]
    fn every_answer_reads_back_as_written() {
        let every = EVERY_REASON
            .into_iter()
            .map(Answer::Refused)
            .chain([Answer::Carried, Answer::NotKept]);
        for answer in every {
            assert_eq!(Answer::read(&answer.written()), Some(answer));
        }
        for line in [
            "",
            "refused",
            "refused ",
            "refused nothing",
            "Carried",
            "ok",
        ] {
            assert_eq!(Answer::read(line), None, "{line}");
        }
    }

    /// **The answer and the record spell a reason the same way**, so what the
    /// turn is told and what the record keeps cannot drift apart.
    #[test]
    fn a_reason_is_spelt_as_the_record_spells_it() {
        for why in EVERY_REASON {
            let entry = Entry::refused_by_the_broker(None, None, why, SystemTime::UNIX_EPOCH);
            let kept = serde_json::to_string(&entry).unwrap();
            let tag = format!("\"refused\":\"{}\"", spelt(why));
            assert!(kept.contains(&tag), "{kept} does not say {tag}");
        }
    }
}
