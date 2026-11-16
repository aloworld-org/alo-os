//! **What the model is shown after the person answered a change.**
//!
//! Measured with `alo-every-try` on 2026-11-16: shown the daemon's bare
//! `did`, a model proposed an approved move again; shown *Done after the
//! person approved it: …* in full access's `done-without-asking` shape, it
//! kept going after one-change requests that full access ends in words (open
//! an application 2 of 5 at *ask each time*, on #1326's head 70d0986e).
//!
//! So a change that ran after an approval is shown in **the same short shape
//! full access uses** — what the change did, and one sentence — with two
//! things changed so that nothing the model is shown is untrue:
//!
//! - **its kind is `done`**, never `done-without-asking`: the person was
//!   asked;
//! - **its sentence is *Done: {sentence}.***
//!   (`alo_capability::words::DONE_AFTER_AN_APPROVAL`), never full access's
//!   *Done without asking you first …*, which a model could repeat to the
//!   person.
//!
//! The line is for the model only, in the language the code is written in,
//! and is never read back as a change done without asking
//! (`crate::what_happened::done_without_asking_in` does not parse it), so the
//! agent never tells the person about it: their own shell already did.
//!
//! A decline and a refusal at the approval are shown as
//! `alo_protocol::what_the_agent_says_back::the_step_after_an_answer` shows
//! them: a refusal.

use alo_capability::words::DONE_AFTER_AN_APPROVAL;
use alo_protocol::{ToAPerson, ToAnAgent};
use alo_strings::{Filling, Strings, Vocabulary};

/// The kind a step after an approved change is shown under.
pub const DONE: &str = "done";

/// The line the model is shown as the step after a change the person
/// answered, whose sentence was `sentence` — [`None`] for an answer an approve
/// or a decline does not earn.
#[must_use]
pub fn the_step_after_an_answer(told: &ToAPerson, sentence: &str) -> Option<String> {
    let (ToAPerson::Did(did) | ToAPerson::DidAndCannotBeUndone(did)) = told else {
        return alo_protocol::what_the_agent_says_back::the_step_after_an_answer(told, sentence);
    };
    let said = the_words()?.say(
        &DONE_AFTER_AN_APPROVAL.key(),
        &Filling::of("sentence", sentence.to_owned()),
    );
    // **Full access's shape, under an honest kind**: the protocol writes the
    // message, and only the name of its kind is changed.
    let written = ToAnAgent::done_without_asking(did.clone(), &said)
        .written()
        .ok()?;
    let mut line: serde_json::Value = serde_json::from_str(&written).ok()?;
    let tells = line.get_mut("tells")?.as_object_mut()?;
    let body = tells.remove("done-without-asking")?;
    tells.insert(DONE.to_owned(), body);
    Some(line.to_string())
}

/// The capability model's own words, in the language the code is written in.
fn the_words() -> Option<Strings> {
    let mut vocabulary = Vocabulary::empty();
    alo_capability::declare_into(&mut vocabulary).ok()?;
    Some(Strings::of(vocabulary))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_protocol::Done;

    /// **Nothing shown after an approval says it was done without asking**,
    /// in the label or in the words; it says what was done, under `done`.
    #[test]
    fn an_approved_change_is_never_shown_as_done_without_asking() {
        let did = Done::Renamed { now_at: None };
        let sentence = "Rename march.pdf to march-final.pdf";
        for told in [
            ToAPerson::Did(did.clone()),
            ToAPerson::DidAndCannotBeUndone(did.clone()),
        ] {
            let shown = the_step_after_an_answer(&told, sentence).unwrap();
            assert!(!shown.contains("without asking"), "{shown}");
            assert!(!shown.contains("without-asking"), "{shown}");
            assert!(!shown.contains("full access"), "{shown}");
            assert!(shown.contains(&format!("\"{DONE}\"")), "{shown}");
            assert!(shown.contains("Done: "), "{shown}");
            assert!(shown.contains(sentence), "{shown}");
            // And it is never read back as a change done without asking.
            assert!(
                crate::what_happened::done_without_asking_in(&[alo_instructing::AStep::of(
                    "{}", &shown
                )])
                .is_empty()
            );
        }
    }

    /// **Full access's shape, with only the kind's name changed**: the same
    /// message as full access would write for this change and this sentence.
    #[test]
    fn the_shape_is_full_access_with_the_kind_renamed() {
        let did = Done::Renamed { now_at: None };
        let shown = the_step_after_an_answer(&ToAPerson::Did(did.clone()), "x").unwrap();
        let mut vocabulary = Vocabulary::empty();
        alo_capability::declare_into(&mut vocabulary).unwrap();
        let said = Strings::of(vocabulary).say(
            &DONE_AFTER_AN_APPROVAL.key(),
            &Filling::of("sentence", "x".to_owned()),
        );
        let full_access_shape = ToAnAgent::done_without_asking(did, &said)
            .written()
            .unwrap();
        let mut expected: serde_json::Value = serde_json::from_str(&full_access_shape).unwrap();
        let tells = expected
            .get_mut("tells")
            .and_then(serde_json::Value::as_object_mut)
            .unwrap();
        let body = tells.remove("done-without-asking").unwrap();
        tells.insert(DONE.to_owned(), body);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&shown).unwrap(),
            expected
        );
    }

    /// **A decline is shown as a refusal**, never as something done.
    #[test]
    fn a_decline_is_shown_as_a_refusal() {
        let shown = the_step_after_an_answer(&ToAPerson::Declined, "Move a.txt to b").unwrap();
        assert!(
            matches!(ToAnAgent::read(&shown), Ok(ToAnAgent::Refused(_))),
            "{shown}"
        );
    }
}
