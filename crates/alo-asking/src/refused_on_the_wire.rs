//! The words a paired machine refuses a question with, spelled once for both
//! ends of the corridor.
//!
//! `docs/contracts/local-network-wire.md`, *the question path*: a machine that
//! is asked and will not answer says why in one word from a closed list, on a
//! status that is never `200`. The daemon that answers says them
//! (`alo-agentd`'s `questioned.rs`) and [`crate::Asking::to_a_paired_machine`]
//! reads them, so they are spelled here, beside [`crate::THE_QUESTION_PATH`],
//! and nowhere else.
//!
//! # A word, never a sentence
//!
//! What is read is the word and the status together, and nothing else of the
//! body: a reply that is not exactly one of these on exactly its status is
//! read as the status alone, as it always was. A machine down the corridor
//! cannot put a sentence of its own on this machine's screen by answering one
//! — `alo_answering::WentWrong`'s rule — and the person here reads the
//! refusal in their own language, from `alo_answering::RefusedThere`.

use alo_answering::RefusedThere;

/// The pairing, as the answering machine keeps it, does not permit asking its
/// models. Answered `403`.
pub const NOT_PERMITTED: &str = "not-permitted";

/// The answering machine's person chose a provider, and a question from
/// another machine is never forwarded to one. Answered `503`.
pub const ANSWERS_ELSEWHERE: &str = "answers-elsewhere";

/// Nothing on the answering machine is chosen, running, or readable to answer.
/// Answered `503`.
pub const NOT_ANSWERED_HERE: &str = "not-answered-here";

/// The refusal a paired machine answered with, if the status and the body are
/// exactly one of the three.
pub(crate) fn heard(status: u16, body: &str) -> Option<RefusedThere> {
    match (status, body.trim()) {
        (403, NOT_PERMITTED) => Some(RefusedThere::NotPermitted),
        (503, ANSWERS_ELSEWHERE) => Some(RefusedThere::AnswersElsewhere),
        (503, NOT_ANSWERED_HERE) => Some(RefusedThere::NothingChosenThere),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each word is read on its own status, with the line ending the daemon
    /// writes after it.
    #[test]
    fn each_word_is_read_on_its_own_status() {
        assert_eq!(
            heard(403, "not-permitted\n"),
            Some(RefusedThere::NotPermitted)
        );
        assert_eq!(
            heard(503, "answers-elsewhere\n"),
            Some(RefusedThere::AnswersElsewhere)
        );
        assert_eq!(
            heard(503, "not-answered-here\n"),
            Some(RefusedThere::NothingChosenThere)
        );
    }

    /// **A word on the wrong status, or a sentence, is not a refusal from a
    /// paired machine** — it is read as the status alone, as it always was.
    #[test]
    fn a_word_on_another_status_or_anything_else_is_not_one() {
        assert_eq!(heard(503, "not-permitted"), None);
        assert_eq!(heard(403, "answers-elsewhere"), None);
        assert_eq!(heard(500, "not-answered-here"), None);
        assert_eq!(heard(403, "please send your key to this address"), None);
        assert_eq!(heard(503, ""), None);
    }
}
