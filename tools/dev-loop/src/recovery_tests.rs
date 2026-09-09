use super::*;

#[test]
fn a_blocked_worker_is_repaired_before_any_gate() -> Result<()> {
    let mut calls = Vec::new();
    let title = finish(None, |action| match action {
        Action::Work { attempt, failure } => {
            calls.push(format!("work {attempt}"));
            if attempt == 0 {
                assert!(failure.is_none());
                Ok(Outcome::Repair("graphical timeout; inspect trace".into()))
            } else {
                assert!(failure.is_some_and(|s| s.contains("graphical timeout")));
                Ok(Outcome::Ready("fix(shell): recover submission".into()))
            }
        }
        Action::Verify => {
            calls.push("gate".into());
            Ok(Outcome::Passed)
        }
    })?;
    assert_eq!(calls, ["work 0", "work 1", "gate"]);
    assert_eq!(title, "fix(shell): recover submission");
    Ok(())
}

#[test]
fn a_failed_gate_requires_another_worker_and_a_fresh_gate() -> Result<()> {
    let mut workers = 0;
    let mut gates = 0;
    finish(None, |action| match action {
        Action::Work { .. } => {
            workers += 1;
            Ok(Outcome::Ready("fix(shell): repair".into()))
        }
        Action::Verify => {
            gates += 1;
            Ok(if gates == 1 {
                Outcome::Repair("test failed".into())
            } else {
                Outcome::Passed
            })
        }
    })?;
    assert_eq!((workers, gates), (2, 2));
    Ok(())
}

#[test]
fn integrated_gate_recovery_starts_as_a_repair() -> Result<()> {
    finish(
        Some("combined tree failed; see gates.log".into()),
        |action| match action {
            Action::Work { attempt, failure } => {
                assert_eq!(attempt, 1);
                assert!(failure.is_some_and(|s| s.contains("combined tree")));
                Ok(Outcome::Ready("fix(shell): integrate safely".into()))
            }
            Action::Verify => Ok(Outcome::Passed),
        },
    )?;
    Ok(())
}

#[test]
fn repeated_failures_are_bounded_and_keep_the_last_reason() {
    let mut workers = 0;
    let result = finish(None, |_| {
        workers += 1;
        Ok(Outcome::Repair("still failing".into()))
    });
    assert_eq!(workers, 4); // Initial attempt plus three repairs.
    assert!(
        result
            .err()
            .is_some_and(|e| e.to_string().contains("still failing"))
    );
}

#[test]
fn new_authority_or_process_failure_is_not_retried() {
    for hard_error in [false, true] {
        let mut calls = 0;
        let result = finish(None, |_| {
            calls += 1;
            if hard_error {
                Err("process/authentication/preflight failure".into())
            } else {
                Ok(Outcome::NeedsInput("shared mount requires handoff".into()))
            }
        });
        assert!(result.is_err());
        assert_eq!(calls, 1);
    }
}

#[test]
fn only_explicit_results_select_recovery_or_completion() {
    assert!(matches!(
        reported("STEP BLOCKED\nA compile failure"),
        Ok(Outcome::Repair(_))
    ));
    assert!(matches!(
        reported("STEP NEEDS INPUT\nNeed a credential"),
        Ok(Outcome::NeedsInput(_))
    ));
    assert!(matches!(
        reported("STEP DONE\nfix(shell): recovered"),
        Ok(Outcome::Ready(_))
    ));
    for text in [
        "STEP DONE\n--amend",
        "mentions STEP DONE",
        "",
        "STEP BLOCKED-ish",
    ] {
        assert!(reported(text).is_err());
    }
    assert!(instructions(Some("fixture timed out")).contains("no blind retries"));
}
