//! Terminal input is established before the private walk installs anything.
//!
//! These are walk-data and simulated-machine checks, not installed evidence.
//! Maximising closes the menu onto its Window actions opener; Escape then
//! leaves that band. A picture of the maximised terminal cannot prove a shell
//! accepted text. Only a fresh journal acknowledgement of a submitted harmless
//! command establishes that prerequisite.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "invalid walk boundaries or untypeable test input must fail this specification test"
)]

use std::time::Duration;

use alo_walking::strokes::Stroke;
use alo_walking::studio_installed_walk::then_installed_studio;
use alo_walking::{Chord, Frame, Machine, MachineError, Pace, Since, Step, Verdict, Walk};

const ACCEPTED: &str = "Studio terminal command accepted";
const ACKNOWLEDGE: &str =
    "printf 'Studio terminal command accepted\\n' | systemd-cat -t studio-walk";

fn private_walk() -> Walk {
    then_installed_studio(Walk::named("terminal preflight"))
}

#[test]
fn the_private_walk_leaves_window_controls_before_typing_into_the_terminal() {
    let walk = private_walk();
    let before_typing: Vec<_> = walk
        .steps()
        .iter()
        .take_while(|step| !matches!(step, Step::Type(_)))
        .filter_map(|step| match step {
            Step::Press(key) => Some(key),
            _ => None,
        })
        .collect();
    assert_eq!(
        before_typing,
        [&Chord::key("esc")],
        "maximising leaves the keyboard on Window actions; Escape must return it to the terminal"
    );
}

#[test]
fn a_separate_submitted_harmless_command_is_acknowledged_before_installation() {
    let walk = private_walk();
    let steps = walk.steps();
    let accepted = steps.iter().position(|step| {
        matches!(step,
        Step::ExpectLine { phrase, since: Since::Mark, within, a_retry_may_pass: false, .. }
        if phrase == ACCEPTED && !within.is_zero() && *within <= Duration::from_secs(30))
    });
    let accepted =
        accepted.expect("missing fresh terminal-command acknowledgement bounded to 30 seconds");
    let before = &steps[..accepted];
    assert!(before.iter().any(|step| matches!(step, Step::Mark)));
    let commands: Vec<_> = before
        .iter()
        .filter_map(|step| match step {
            Step::Type(command) => Some(command),
            _ => None,
        })
        .collect();
    assert_eq!(
        commands.len(),
        1,
        "installation must not be typed before shell acceptance"
    );
    let command = commands[0];
    assert_eq!(
        command.as_str(),
        ACKNOWLEDGE,
        "the acknowledgement must be emitted by the actual submitted shell command"
    );
    assert!(
        matches!(before.last(), Some(Step::Press(key)) if *key == Chord::key("ret")),
        "typing a command without submitting it establishes nothing"
    );
    assert!(steps[accepted + 1..].iter().any(|step| matches!(step,
        Step::Type(command) if command.contains("flatpak install --user"))));
}

/// A silent terminal, optionally with an acknowledgement left in its old log.
/// It deliberately does not simulate a shell accepting any of the input.
struct SilentTerminal {
    old_log: Vec<u8>,
    now: Duration,
    strokes: Vec<Stroke>,
}

impl Machine for SilentTerminal {
    fn stroke(&mut self, stroke: &Stroke) -> Result<(), MachineError> {
        self.strokes.push(stroke.clone());
        Ok(())
    }
    fn point_at(&mut self, _: (usize, usize)) -> Result<(), MachineError> {
        Ok(())
    }
    fn button(&mut self, _: bool) -> Result<(), MachineError> {
        Ok(())
    }
    fn frame(&mut self) -> Result<Frame, MachineError> {
        Ok(Frame::filled(4, 4, [0, 0, 0]))
    }
    fn keep_picture(&mut self, name: &str) -> Result<String, MachineError> {
        Ok(name.to_owned())
    }
    fn log(&mut self) -> Result<Vec<u8>, MachineError> {
        Ok(self.old_log.clone())
    }
    fn wait(&mut self, duration: Duration) {
        self.now += duration;
    }
    fn elapsed(&self) -> Duration {
        self.now
    }
}

#[test]
fn absent_or_stale_shell_acceptance_refuses_early_and_never_types_the_install_command() {
    for old_log in [
        Vec::new(),
        format!("studio-walk: {ACCEPTED}\n").into_bytes(),
    ] {
        let walk = private_walk();
        let mut machine = SilentTerminal {
            old_log,
            now: Duration::ZERO,
            strokes: Vec::new(),
        };
        let report = alo_walking::walk(
            &mut machine,
            &walk,
            &|_| Some("1".to_owned()),
            Pace {
                hold: Duration::ZERO,
                between: Duration::ZERO,
            },
        );
        assert!(!report.passed());
        assert!(
            machine.now <= Duration::from_secs(30),
            "unavailable shell input must not spend the 20-minute recording budget"
        );
        let keys: Vec<_> = machine
            .strokes
            .iter()
            .filter(|stroke| stroke.down)
            .map(|stroke| stroke.key)
            .collect();
        let first_command = walk
            .steps()
            .iter()
            .find_map(|step| match step {
                Step::Type(command) => Some(command),
                _ => None,
            })
            .expect("the walk must submit its harmless preflight");
        let mut expected = vec!["esc"];
        expected.extend(
            alo_walking::keys::typing(first_command)
                .expect("typeable preflight")
                .iter()
                .flat_map(|chord| {
                    alo_walking::strokes::of(
                        chord,
                        Pace {
                            hold: Duration::ZERO,
                            between: Duration::ZERO,
                        },
                    )
                })
                .filter(|stroke| stroke.down)
                .map(|stroke| stroke.key),
        );
        expected.push("ret");
        assert_eq!(
            keys, expected,
            "no installation/controller input may follow a missing acknowledgement"
        );
        let checks = report.checks();
        assert!(
            checks.iter().any(|check| check.verdict == Verdict::Failed
                && check.evidence.contains("was not said within")),
            "{}",
            report.as_text()
        );
        assert!(
            checks
                .iter()
                .any(|check| check.verdict == Verdict::NotReached),
            "recording completion must be reported as not reached"
        );
    }
}
