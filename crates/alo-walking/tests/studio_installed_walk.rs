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
    assert!(
        steps[at + 1 + count..]
            .iter()
            .any(|step| matches!(step, Step::OnlyWith("ALO_WALK_PROVIDER_KEY"))),
        "the absent private option must leave the provider walk reachable"
    );
}

fn has_no_additional_environment_gate(steps: &[Step]) -> bool {
    !steps.iter().any(|step| {
        matches!(
            step,
            Step::TypeSecret(_) | Step::OnlyWith(_) | Step::OnlyWithFor { .. }
        )
    })
}

#[test]
fn private_installed_steps_have_no_additional_environment_gate_or_secret_input() {
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
            assert!(has_no_additional_environment_gate(
                &steps[at + 1..at + 1 + count]
            ));
        }
    }
}

#[test]
fn nested_scoped_environment_and_secret_reads_are_refused() {
    for step in [
        Step::OnlyWithFor {
            variable: "ALO_WALK_PROVIDER_KEY",
            steps: 1,
        },
        Step::OnlyWith("ALO_WALK_PROVIDER_KEY"),
        Step::TypeSecret("ALO_WALK_PROVIDER_KEY"),
    ] {
        assert!(!has_no_additional_environment_gate(&[step]));
    }
}

#[cfg(unix)]
#[test]
fn failures_at_each_person_command_are_reported_without_installing_or_recording_afterward(
) -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt as _;

    struct Temporary(std::path::PathBuf);
    impl Drop for Temporary {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let walk = the_first_walk();
    let Some(command) = walk.steps().iter().find_map(|step| match step {
        Step::Type(command) if command.contains("python3 studio-session") => Some(command),
        _ => None,
    }) else {
        return Err("no installed person-flow command".into());
    };
    assert!(command.contains("set -eu;"));
    assert!(command.contains("sha256sum -c checksums.txt;"));
    for failing in [
        "",
        "curl",
        "sha256sum",
        "flatpak-studio",
        "flatpak-fixture",
        "python3",
    ] {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let root = Temporary(std::env::temp_dir().join(format!(
            "studio-person-command-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        )));
        std::fs::create_dir(&root.0)?;
        let bin = root.0.join("bin");
        let work = root.0.join("work");
        std::fs::create_dir(&bin)?;
        std::fs::create_dir(&work)?;
        for tool in ["curl", "sha256sum", "python3"] {
            let script = format!(
                "#!/bin/sh\nprintf '%s\\n' '{tool}' >> \"$STUDIO_FAKE_TRACE\"\n\
                 if [ \"$STUDIO_FAIL_AT\" = '{tool}' ]; then exit 17; fi\nexit 0\n"
            );
            std::fs::write(bin.join(tool), script)?;
        }
        // Flatpak's local-bundle mode accepts one filename per invocation.
        // A permissive success stub previously hid the real CLI refusal.
        std::fs::write(
            bin.join("flatpak"),
            "#!/bin/sh\n\
             bundles=0; package=''\n\
             for argument; do\n\
               case \"$argument\" in *.flatpak) bundles=$((bundles + 1)); package=$argument;; esac\n\
             done\n\
             if [ \"$bundles\" -ne 1 ]; then\n\
               printf '%s\\n' flatpak-rejected >> \"$STUDIO_FAKE_TRACE\"; exit 23\n\
             fi\n\
             case \"$package\" in\n\
               studio.flatpak) stage=flatpak-studio;;\n\
               fixture.flatpak) stage=flatpak-fixture;;\n\
               *) exit 24;;\n\
             esac\n\
             printf '%s\\n' \"$stage\" >> \"$STUDIO_FAKE_TRACE\"\n\
             if [ \"$STUDIO_FAIL_AT\" = \"$stage\" ]; then exit 17; fi\n\
             exit 0\n",
        )?;
        std::fs::write(
            bin.join("mktemp"),
            "#!/bin/sh\nprintf '%s\\n' \"$STUDIO_FAKE_DIRECTORY\"\n",
        )?;
        std::fs::write(
            bin.join("systemd-cat"),
            "#!/bin/sh\n/bin/cat > \"$STUDIO_FAKE_JOURNAL\"\n",
        )?;
        for entry in std::fs::read_dir(&bin)? {
            std::fs::set_permissions(entry?.path(), std::fs::Permissions::from_mode(0o755))?;
        }
        let trace = root.0.join("trace");
        let journal = root.0.join("journal");
        let result = std::process::Command::new("/bin/bash")
            .arg("--noprofile")
            .arg("--norc")
            .arg("-c")
            .arg(command)
            .env("PATH", &bin)
            .env("STUDIO_FAIL_AT", failing)
            .env("STUDIO_FAKE_DIRECTORY", &work)
            .env("STUDIO_FAKE_TRACE", &trace)
            .env("STUDIO_FAKE_JOURNAL", &journal)
            .output()?;
        assert!(result.status.success(), "fake terminal failed: {result:?}");
        let expected_exit = if failing.is_empty() { 0 } else { 17 };
        assert_eq!(
            std::fs::read_to_string(journal)?,
            format!("Studio installed flow exit {expected_exit}\n"),
            "failure at {failing}"
        );
        let expected_calls = if failing == "curl" {
            vec!["curl"]
        } else {
            let mut calls = vec!["curl"; 7];
            for tool in ["sha256sum", "flatpak-studio", "flatpak-fixture", "python3"] {
                calls.push(tool);
                if tool == failing {
                    break;
                }
            }
            calls
        };
        assert_eq!(
            std::fs::read_to_string(trace)?.lines().collect::<Vec<_>>(),
            expected_calls,
            "failure at {failing}"
        );
    }
    Ok(())
}
