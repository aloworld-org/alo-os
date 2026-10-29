//! **Every follow-up grade is what its counts earn, under instructions this
//! crate has** — `data/follow-ups.toml` held to the same two rules as the
//! catalogue's grades (ADR 0138).
//!
//! It runs on every build, and it reads the file as shipped: a grade typed in
//! by hand that its counts do not earn, or one naming a digest nothing here can
//! produce, fails here rather than on somebody's screen.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_driving::{Instructions, grade_of};
use alo_models::FollowUpsMeasured;

/// **A follow-up grade is the grade its counts earn**, by the bar the fixed set
/// is held to.
#[test]
fn every_follow_up_grade_is_what_its_counts_earn() {
    let measured = FollowUpsMeasured::built_in().expect("the follow-ups this build carries read");
    for one in &measured.each {
        let drove = one
            .measured
            .drove
            .expect("a follow-up grade has its counts");
        let of = one.measured.of.expect("a follow-up grade has its counts");
        assert_eq!(
            grade_of(drove as usize, of as usize),
            one.grade,
            "{} at {}: {drove} of {of}",
            one.model,
            one.at
        );
    }
}

/// **Every follow-up grade names instructions this crate has**, so it can be
/// re-run under the same words.
#[test]
fn every_follow_up_grade_names_instructions_this_crate_has() {
    let measured = FollowUpsMeasured::built_in().expect("the follow-ups this build carries read");
    for one in &measured.each {
        let digest = one
            .measured
            .instructions
            .as_deref()
            .expect("a follow-up grade names its instructions");
        assert!(
            Instructions::ALL
                .iter()
                .any(|instructions| instructions.digest() == digest),
            "{} at {} names {digest}, which no instructions here have",
            one.model,
            one.at
        );
    }
}
