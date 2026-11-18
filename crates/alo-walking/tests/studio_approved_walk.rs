//! Genuine approval input follows provider setup, on the private installed walk.

use alo_walking::{Chord, Since, step::Step, the_first_walk};

#[test]
fn native_studio_approval_is_after_provider_setup_and_before_other_model_work()
-> Result<(), &'static str> {
    let walk = the_first_walk();
    let steps = walk.steps();
    let provider = steps
        .iter()
        .position(|step| {
            matches!(step,
        Step::Look { name, .. } if name == "the-provider-saved")
        })
        .ok_or("no provider setup")?;
    let approval = steps
        .iter()
        .position(|step| {
            matches!(step,
        Step::Section { name, .. } if name == "Studio native approval")
        })
        .ok_or("no real Studio approval section")?;
    let cleanup = steps
        .iter()
        .position(|step| {
            matches!(step,
        Step::Section { name, .. } if name == "Studio approval cleanup")
        })
        .ok_or("no independent cleanup section")?;
    let other = steps
        .iter()
        .position(|step| {
            matches!(step,
        Step::Section { name, .. } if name == "VSCodium at full access")
        })
        .ok_or("no later model section")?;
    assert!(provider < approval && approval < cleanup && cleanup < other);
    let section = steps.get(approval + 1..cleanup).ok_or("invalid section")?;
    assert!(matches!(section.first(), Some(Step::OnlyWithFor {
        variable: "ALO_WALK_PRIVATE_STUDIO", steps: count,
    }) if *count + 1 == section.len()));
    assert!(
        !section
            .iter()
            .any(|step| matches!(step, Step::TypeSecret(_)))
    );
    assert!(section.iter().any(|step| matches!(step, Step::Type(text)
        if text.contains("sha256sum -c checksums.txt")
        && text.contains("STUDIO_PRIVATE_INSTALLED_WALK=1 python3 studio-session --approvals"))));
    Ok(())
}

#[test]
fn actual_asks_and_native_cards_drive_approval_and_stale_grant_refusal() -> Result<(), &'static str>
{
    let walk = the_first_walk();
    let steps = walk.steps();
    for (phase, request) in [
        ("enable", "Enable the microphone in alo Studio."),
        ("withdraw", "Disable the microphone in alo Studio."),
        ("full", "Disable the microphone in alo Studio."),
    ] {
        let ready = format!("Studio approval {phase} request ready");
        let at = steps
            .iter()
            .position(|step| {
                matches!(step,
            Step::ExpectLine { phrase, since: Since::Mark, a_retry_may_pass: false, .. }
            if phrase == &ready)
            })
            .ok_or("missing fresh controller readiness")?;
        let remaining = steps.get(at + 1..).ok_or("missing request")?;
        let ask = remaining
            .iter()
            .position(|step| {
                matches!(step, Step::Type(text)
            if text == request)
            })
            .ok_or("no genuine microphone request")?;
        assert!(remaining.get(..ask).is_some_and(|before| before.iter().any(
            |step| matches!(step, Step::Press(chord) if *chord == Chord::with("meta_l", "a"))
        )));
        if phase != "full" {
            let card = format!("studio-{phase}-approval-card");
            assert!(
                remaining
                    .iter()
                    .any(|step| matches!(step, Step::Look { name, .. }
                if name == &card)),
                "the native card needs a retained picture"
            );
        }
    }
    assert!(steps.iter().any(|step| matches!(step, Step::ExpectLine {
        phrase, and_says: Some(says), since: Since::Mark, ..
    } if phrase == "Studio approved flow exit" && says == "exit 0")));
    Ok(())
}
