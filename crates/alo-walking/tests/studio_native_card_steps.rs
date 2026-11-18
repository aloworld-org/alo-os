//! Native consent cannot be replaced by a protocol approval or an old card.

use alo_walking::{Chord, Since, step::Step, the_first_walk};

fn position(steps: &[Step], predicate: impl Fn(&Step) -> bool) -> Result<usize, &'static str> {
    steps.iter().position(predicate).ok_or("missing required native approval step")
}

#[test]
fn each_ask_requires_the_fresh_zero_text_native_card_and_exact_pending_evidence()
-> Result<(), &'static str> {
    let walk = the_first_walk();
    let steps = walk.steps();
    for phase in ["enable", "withdraw"] {
        let ready = format!("Studio approval {phase} request ready");
        let start = position(steps, |step| matches!(step, Step::ExpectLine { phrase, .. }
            if phrase == &ready))?;
        let tail = steps.get(start + 1..).ok_or("no request after readiness")?;
        let observed = format!("Studio approval {phase} observed");
        let end = position(tail, |step| matches!(step, Step::ExpectLine { phrase, .. }
            if phrase == &observed))?;
        let phase_steps = tail.get(..end).ok_or("no bounded phase")?;
        let card = position(phase_steps, |step| matches!(step, Step::ExpectLine {
            phrase, and_says: Some(says), since: Since::Mark,
            a_retry_may_pass: false, ..
        } if phrase == "an approval card is in front of the person, showing"
            && says == "0 characters of text to type, whole"))?;
        let verified = format!("Studio approval {phase} card verified");
        let pending = position(phase_steps, |step| matches!(step, Step::ExpectLine {
            phrase, since: Since::Mark, a_retry_may_pass: false, ..
        } if phrase == &verified))?;
        let picture = format!("studio-{phase}-approval-card");
        let shown = position(phase_steps, |step| matches!(step, Step::Look { name, .. }
            if name == &picture))?;
        assert!(card < pending && pending < shown);
        assert!(phase_steps.get(..card).is_some_and(|before| before.windows(2).any(|pair|
            matches!(pair, [Step::Mark, Step::Press(chord)] if *chord == Chord::key("ret")))));
    }
    Ok(())
}

#[test]
fn native_allow_once_keys_follow_the_retained_card_and_withdrawal_precedes_stale_approval()
-> Result<(), &'static str> {
    let walk = the_first_walk();
    let steps = walk.steps();
    let enable = position(steps, |step| matches!(step, Step::Look { name, .. }
        if name == "studio-enable-approval-card"))?;
    assert!(matches!(steps.get(enable + 1..enable + 4), Some([
        Step::Mark, Step::Press(tab), Step::Press(enter)
    ]) if *tab == Chord::key("tab") && *enter == Chord::key("ret")));
    let withdraw = position(steps, |step| matches!(step, Step::Look { name, .. }
        if name == "studio-withdraw-approval-card"))?;
    assert!(matches!(steps.get(withdraw + 1), Some(Step::Press(settings))
        if *settings == Chord::with("meta_l", "i")));
    assert!(matches!(steps.get(withdraw + 2), Some(Step::ExpectLine {
        phrase, since: Since::Mark, a_retry_may_pass: false, ..
    }) if phrase == "Studio approval withdraw grant revoked"));
    assert!(matches!(steps.get(withdraw + 3), Some(Step::Press(escape))
        if *escape == Chord::key("esc")));
    assert!(matches!(steps.get(withdraw + 4), Some(Step::Look { name, .. })
        if name == "studio-withdraw-card-after-revocation"));
    assert!(matches!(steps.get(withdraw + 5..withdraw + 8), Some([
        Step::Mark, Step::Press(tab), Step::Press(enter)
    ]) if *tab == Chord::key("tab") && *enter == Chord::key("ret")));
    Ok(())
}

#[test]
fn private_native_approval_body_has_one_scoped_gate_and_no_secret_or_protocol_input()
-> Result<(), &'static str> {
    let walk = the_first_walk();
    let steps = walk.steps();
    let section = position(steps, |step| matches!(step, Step::Section { name, .. }
        if name == "Studio native approval"))?;
    let Some(Step::OnlyWithFor { variable: "ALO_WALK_PRIVATE_STUDIO", steps: count }) =
        steps.get(section + 1) else { return Err("missing fixed private gate") };
    let body = steps.get(section + 2..section + 2 + count).ok_or("invalid scoped gate")?;
    assert!(!body.iter().any(|step| matches!(step,
        Step::OnlyWith(_) | Step::OnlyWithFor { .. } | Step::TypeSecret(_))));
    for step in body {
        if let Step::Type(text) = step {
            assert!(!text.contains("\"approve\"") && !text.contains("\"deny\""));
        }
    }
    assert!(matches!(steps.get(section + 2 + count), Some(Step::Section { name, .. })
        if name == "VSCodium at full access"));
    Ok(())
}

#[test]
fn a_failed_native_studio_turn_cannot_feed_a_later_model_request()
-> Result<(), &'static str> {
    let walk = the_first_walk();
    for section in ["VSCodium at full access", "every application at full access",
                    "VSCodium at ask each time"] {
        let step = walk.steps().iter().find(|step| matches!(step,
            Step::Section { name, .. } if name == section)).ok_or("missing later model section")?;
        assert!(matches!(step, Step::Section { needs, .. }
            if needs.contains(&"Studio native approval")),
            "a failed Studio task may still own the daemon connection: {section}");
    }
    assert!(walk.steps().iter().any(|step| matches!(step,
        Step::Section { name, needs } if name == "the Papers sheet at ask each time"
            && needs.contains(&"VSCodium at ask each time"))));
    Ok(())
}
