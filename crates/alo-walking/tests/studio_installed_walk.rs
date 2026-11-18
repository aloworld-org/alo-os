//! The private installed recording flow is reachable only on an explicit walk.

#![expect(
    clippy::indexing_slicing,
    reason = "an invalid section boundary must fail the walk specification test"
)]

use alo_walking::step::Step;
use alo_walking::the_first_walk;

#[test]
fn private_studio_steps_are_scoped_after_the_terminal_and_before_the_pointer() {
    let walk = the_first_walk();
    let steps = walk.steps();
    let gate = steps.iter().position(|step| {
        matches!(
            step,
            Step::OnlyWithFor {
                variable: "ALO_WALK_PRIVATE_STUDIO",
                ..
            }
        )
    });
    assert!(gate.is_some(), "no opt-in installed Studio section");
    let Some(at) = gate else { return };
    let Step::OnlyWithFor { steps: count, .. } = steps[at] else {
        return;
    };
    let before = &steps[..at];
    assert!(before.iter().any(|step| matches!(step,
        Step::Look { name, .. } if name == "the-terminal-maximised")));
    let inside = &steps[at + 1..at + 1 + count];
    assert!(inside.iter().any(|step| matches!(step, Step::Type(command)
        if command.contains("sha256sum -c checksums.txt")
            && command.contains("flatpak install --user")
            && command.contains("STUDIO_PRIVATE_INSTALLED_WALK=1 python3 studio-session")
            && command.contains("systemd-cat -t studio-walk"))));
    assert!(inside.iter().any(|step| matches!(step,
        Step::ExpectLine { phrase, and_says: Some(also), since: alo_walking::Since::Mark, .. }
        if phrase == "Studio installed flow exit" && also == "exit 0")));
    assert!(
        steps[at + 1 + count..]
            .iter()
            .any(|step| matches!(step, Step::Pointer(_))),
        "the opt-in must leave the existing pointer walk reachable"
    );
}

#[test]
fn private_installed_flow_never_reads_a_provider_key_or_grants_an_agent_execution() {
    let walk = the_first_walk();
    let steps = walk.steps();
    assert!(
        steps.iter().any(|step| matches!(
            step,
            Step::OnlyWithFor {
                variable: "ALO_WALK_PRIVATE_STUDIO",
                ..
            }
        )),
        "the private section must exist before its boundary can be checked"
    );
    for (at, step) in steps.iter().enumerate() {
        if let Step::OnlyWithFor {
            variable: "ALO_WALK_PRIVATE_STUDIO",
            steps: count,
        } = step
        {
            assert!(
                !steps[at + 1..at + 1 + count]
                    .iter()
                    .any(|step| matches!(step, Step::TypeSecret(_) | Step::OnlyWith(_)))
            );
        }
    }
}
