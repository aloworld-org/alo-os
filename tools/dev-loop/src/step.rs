//! One unfinished task, its repair attempts and independently repeated gates.
use crate::{
    Result,
    awake::Awake,
    process::{git, worker},
    recovery::{self, Action, Outcome},
};
use std::{
    fs::{self, File},
    path::Path,
};

/// No worker may change history, stage files, leave conflicts or rewrite its runner.
fn intact(head: &str) -> Result<()> {
    if git(&["rev-parse", "HEAD"])? != head {
        return Err("Worker changed HEAD; review required".into());
    }
    if !git(&["diff", "--cached", "--name-only"])?.trim().is_empty()
        || !git(&["ls-files", "--unmerged"])?.trim().is_empty()
    {
        return Err("Index changed or conflicts remain; preserve work for review".into());
    }
    if !git(&["status", "--porcelain", "--", "tools/dev-loop"])?
        .trim()
        .is_empty()
    {
        return Err("Supervisor changes require interactive review and restart".into());
    }
    Ok(())
}

fn complete_changes() -> Result<()> {
    if git(&["status", "--porcelain"])?.trim().is_empty() {
        return Err("Worker reported completion without changes".into());
    }
    for file in [
        "CHANGELOG.md",
        "ROADMAP.md",
        "docs/autonomy/QUEUE.md",
        "docs/autonomy/STATE.md",
    ] {
        if git(&["diff", "HEAD", "--", file])?.is_empty() {
            return Err(format!("Required progress document was not updated: {file}").into());
        }
    }
    Ok(())
}

pub fn finish(
    state: &Path,
    codex: &str,
    head: &str,
    directory: &Path,
    initial_failure: Option<String>,
    linux: &mut Awake,
) -> Result<String> {
    let mut current = directory.to_path_buf();
    recovery::finish(initial_failure, |action| {
        linux.check()?;
        crate::storage::require_space()?;
        intact(head)?;
        match action {
            Action::Work { attempt, failure } => {
                if state.join("STOP").exists() {
                    return Err(
                        "Owner requested stop; unfinished task preserved, no repair launched"
                            .into(),
                    );
                }
                let context =
                    failure.map(|why| format!("Previous attempt: {}\n{why}", current.display()));
                current = if attempt == 0 {
                    directory.to_path_buf()
                } else {
                    directory.join(format!("repair-{attempt}"))
                };
                fs::create_dir_all(&current)?;
                if attempt > 0 {
                    // Missing mounts/low disk are not code defects a worker may repair.
                    let mut preflight = File::create(current.join("preflight.log"))?;
                    crate::linux_gate::ready(&mut preflight)?;
                }
                crate::status(
                    state,
                    &format!(
                        "{}: attempt {attempt}, logs={}",
                        if attempt == 0 {
                            "WORKING"
                        } else {
                            "RECOVERING"
                        },
                        current.display()
                    ),
                )?;
                let response = current.join("result.txt");
                worker(
                    codex,
                    &response,
                    &current,
                    &recovery::instructions(context.as_deref()),
                )?;
                intact(head)?;
                recovery::reported(&fs::read_to_string(response)?)
            }
            Action::Verify => {
                if let Err(error) = complete_changes() {
                    return Ok(Outcome::Repair(format!(
                        "Task completeness check failed: {error}"
                    )));
                }
                crate::status(
                    state,
                    "VERIFYING: all gates after the current implementation/repair",
                )?;
                let mut log = File::create(current.join("gates.log"))?;
                match crate::gates(&mut log) {
                    Ok(()) => {
                        intact(head)?;
                        Ok(Outcome::Passed)
                    }
                    Err(error) => Ok(Outcome::Repair(format!(
                        "Independent gate failed: {error}. Inspect {}/gates.log",
                        current.display()
                    ))),
                }
            }
        }
    })
}
