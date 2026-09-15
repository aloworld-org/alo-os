//! **Every grade the catalogue ships names instructions `alo-driving` has.**
//!
//! Task 16 of `docs/autonomy/v0-5-the-models-measured-plan.md` and
//! [ADR 0034](../../../docs/decisions/0034-the-instructions-show-every-door-they-ask-a-model-to-choose.md).
//! `alo-models` holds a grade's `instructions` to the shape of a SHA-256; only
//! this crate has the text a digest is of. So this holds every digest in the
//! shipped catalogue to a set of [`alo_driving::Instructions`] — a grade naming
//! text nobody can show a model again is a grade nobody can re-run — and holds a
//! grade under other instructions to naming a set other than its entry's own.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_driving::Instructions;
use alo_models::{Catalogue, MeasuredOn};

/// The instructions a statement names, or the failure to say which.
fn named_by(on: &MeasuredOn, what: &str) -> Instructions {
    let digest = on
        .instructions
        .as_deref()
        .unwrap_or_else(|| panic!("{what} names no instructions"));
    Instructions::of_digest(digest)
        .unwrap_or_else(|| panic!("{what} names instructions alo-driving does not have: {digest}"))
}

#[test]
fn every_grade_the_catalogue_ships_names_instructions_this_crate_has() {
    let shipped = Catalogue::built_in().unwrap();
    let mut named = 0;
    for entry in &shipped.models {
        if let Some(on) = &entry.measured {
            named_by(on, &format!("{}, asked freely", entry.id));
            named += 1;
        }
        let own = entry
            .measured_in_the_envelope
            .as_ref()
            .map(|on| named_by(on, &format!("{}, in the envelope", entry.id)));
        let the_entrys_way = entry.measured_in_the_envelope.as_ref().map(|on| {
            (
                on.instructions.as_deref(),
                on.runtime.as_str(),
                on.held_to.as_deref(),
            )
        });
        for also in &entry.also_at {
            let at = named_by(
                &also.measured_in_the_envelope,
                &format!("{} at {}", entry.id, also.quantisation),
            );
            named += 1;
            for under in &also.also_under {
                let under = named_by(
                    &under.measured_in_the_envelope,
                    &format!(
                        "{} at {}, under other instructions",
                        entry.id, also.quantisation
                    ),
                );
                assert_ne!(under, at, "{} at {}", entry.id, also.quantisation);
                named += 1;
            }
        }
        for also in &entry.also_under {
            named_by(
                &also.measured_in_the_envelope,
                &format!("{}, measured another way", entry.id),
            );
            // Another way is another set of instructions, another runtime, or
            // another shape holding the answer — and never all three the same
            // as the grade it sits beside (ADR 0034, ADR 0035).
            assert_ne!(Some(also.the_way()), the_entrys_way, "{}", entry.id);
            named += 1;
        }
        named += usize::from(own.is_some());
    }
    assert!(named >= 27, "only {named} grades named their instructions");
}

/// **Every grade before ADR 0034 was earned under the first instructions**, and
/// the catalogue says so rather than leaving it to be inferred.
#[test]
fn the_grades_written_before_the_second_instructions_name_the_first() {
    let shipped = Catalogue::built_in().unwrap();
    for entry in &shipped.models {
        let own = [
            entry.measured.as_ref(),
            entry.measured_in_the_envelope.as_ref(),
        ];
        for on in own.into_iter().flatten() {
            assert_eq!(
                Instructions::of_digest(on.instructions.as_deref().expect("named")),
                Some(Instructions::AsFirstWritten),
                "{}: an entry's own grade is the method every entry is compared by",
                entry.id
            );
        }
    }
}

/// **The digest the catalogue decides by is the digest of the words a turn is
/// shown** (task 19, ADR 0037 decision 4).
///
/// `alo-models` writes `THE_WORDS_A_TURN_SHOWS` out rather than computing it, so
/// that the catalogue can say which grade decides without depending on the verb
/// registry. This crate has both the constant and the text, so this is where the
/// two are held to each other: change the text and this fails, rather than the
/// catalogue quietly deciding by a digest nothing earns.
#[test]
fn the_digest_the_catalogue_decides_by_is_the_one_a_turn_is_shown() {
    assert_eq!(
        alo_models::THE_WORDS_A_TURN_SHOWS,
        Instructions::SHOWN_TO_A_TURN.digest(),
        "the words a turn shows have changed; every grade earned under them names text that no \
         longer exists, and the catalogue is deciding by a digest nothing has"
    );
    assert_eq!(
        Instructions::of_digest(alo_models::THE_WORDS_A_TURN_SHOWS),
        Some(Instructions::SHOWN_TO_A_TURN)
    );

    // And at least one shipped entry is decided by it, so the constant is not
    // matching nothing at all.
    let shipped = Catalogue::built_in().unwrap();
    assert!(
        shipped
            .models
            .iter()
            .any(|entry| entry.grade_for_the_turn().1 == alo_models::AskedTheWay::AsATurnAsks)
    );
}
