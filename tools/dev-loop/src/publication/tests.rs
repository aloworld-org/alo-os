//! Real local Git remotes exercise races, conflicts and failed integration gates.
use super::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

struct Fixture {
    root: PathBuf,
    first: PathBuf,
    second: PathBuf,
    base: String,
}

fn command(directory: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .current_dir(directory)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

fn commit(directory: &Path, file: &str, content: &str) -> Result<()> {
    fs::write(directory.join(file), content)?;
    command(directory, &["add", "--all"])?;
    command(
        directory,
        &[
            "-c",
            "user.name=Git test fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-m",
            "test: fixture change",
        ],
    )?;
    Ok(())
}

impl Fixture {
    fn new() -> Result<Self> {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let root =
            std::env::temp_dir().join(format!("alo-publication-{}-{stamp}", std::process::id()));
        fs::create_dir(&root)?;
        command(
            &root,
            &["init", "--bare", "--initial-branch=main", "remote.git"],
        )?;
        command(&root, &["clone", "remote.git", "first"])?;
        let first = root.join("first");
        commit(&first, "shared.txt", "base\n")?;
        command(&first, &["push", "origin", "main"])?;
        command(&root, &["clone", "remote.git", "second"])?;
        let second = root.join("second");
        // Rebase needs a committer identity too; these are isolated test repos.
        for directory in [&first, &second] {
            command(directory, &["config", "user.name", "Git test fixture"])?;
            command(
                directory,
                &["config", "user.email", "fixture@example.invalid"],
            )?;
            command(directory, &["config", "commit.gpgsign", "false"])?;
        }
        let base = command(&first, &["rev-parse", "HEAD"])?;
        Ok(Self {
            root,
            first,
            second,
            base,
        })
    }

    fn publish_second(&self, file: &str) -> Result<()> {
        commit(&self.second, file, "second worker\n")?;
        command(&self.second, &["push", "origin", "main"])?;
        Ok(())
    }

    fn remote_head(&self) -> Result<String> {
        command(&self.root.join("remote.git"), &["rev-parse", "main"])
    }
}

#[test]
fn incoming_work_is_retested_and_both_changes_reach_main() -> Result<()> {
    let fixture = Fixture::new()?;
    commit(&fixture.first, "first.txt", "first worker\n")?;
    fixture.publish_second("second.txt")?;
    let mut checks = 0;
    let head = publish(
        &fixture.base,
        |args| command(&fixture.first, args),
        || {
            checks += 1;
            assert!(fixture.first.join("first.txt").exists());
            assert!(fixture.first.join("second.txt").exists());
            Ok(())
        },
    )?;
    assert_eq!(checks, 1);
    assert_eq!(head, fixture.remote_head()?);
    Ok(())
}

#[test]
fn a_failed_combined_gate_leaves_remote_unchanged() -> Result<()> {
    let fixture = Fixture::new()?;
    commit(&fixture.first, "first.txt", "first worker\n")?;
    fixture.publish_second("second.txt")?;
    let before = fixture.remote_head()?;
    let result = publish(
        &fixture.base,
        |args| command(&fixture.first, args),
        || Err("test failure".into()),
    );
    assert!(result.is_err());
    assert_eq!(before, fixture.remote_head()?);
    assert!(fixture.first.join("first.txt").exists());
    Ok(())
}

#[test]
fn an_integrated_repair_is_gated_before_it_can_reach_the_remote() -> Result<()> {
    use crate::recovery::{self, Action, Outcome};
    let fixture = Fixture::new()?;
    commit(&fixture.first, "first.txt", "first worker\n")?;
    fixture.publish_second("second.txt")?;
    let before = fixture.remote_head()?;
    let mut repaired = false;
    let mut regated = false;
    let head = publish(
        &fixture.base,
        |args| command(&fixture.first, args),
        || {
            recovery::finish(Some("combined check failed".into()), |action| {
                assert_eq!(fixture.remote_head()?, before);
                match action {
                    Action::Work { attempt, .. } => {
                        assert_eq!(attempt, 1);
                        fs::write(fixture.first.join("repair.txt"), "repaired\n")?;
                        repaired = true;
                        Ok(Outcome::Ready("fix(shell): integrated repair".into()))
                    }
                    Action::Verify => {
                        assert!(repaired);
                        assert!(fixture.first.join("first.txt").exists());
                        assert!(fixture.first.join("second.txt").exists());
                        assert_eq!(
                            fs::read_to_string(fixture.first.join("repair.txt"))?,
                            "repaired\n"
                        );
                        regated = true;
                        Ok(Outcome::Passed)
                    }
                }
            })?;
            assert!(regated);
            commit(&fixture.first, "repair.txt", "repaired\n")?;
            Ok(())
        },
    )?;
    assert!(regated);
    assert_eq!(head, fixture.remote_head()?);
    assert_eq!(
        command(
            &fixture.root.join("remote.git"),
            &["show", "main:repair.txt"]
        )?,
        "repaired\n"
    );
    Ok(())
}

#[test]
fn conflicting_changes_are_preserved_without_a_push() -> Result<()> {
    let fixture = Fixture::new()?;
    commit(&fixture.first, "shared.txt", "first worker\n")?;
    fixture.publish_second("shared.txt")?;
    let before = fixture.remote_head()?;
    let mut checks = 0;
    let result = publish(
        &fixture.base,
        |args| command(&fixture.first, args),
        || {
            checks += 1;
            Ok(())
        },
    );
    assert!(result.is_err());
    assert_eq!(checks, 0);
    assert_eq!(before, fixture.remote_head()?);
    assert!(
        !command(&fixture.first, &["diff", "--name-only", "--diff-filter=U"])?
            .trim()
            .is_empty()
    );
    Ok(())
}

#[test]
fn an_update_between_fetch_and_push_is_integrated_on_retry() -> Result<()> {
    let fixture = Fixture::new()?;
    commit(&fixture.first, "first.txt", "first worker\n")?;
    let mut raced = false;
    let mut checks = 0;
    let head = publish(
        &fixture.base,
        |args| {
            if args.first() == Some(&"push") && !raced {
                fixture.publish_second("second.txt")?;
                raced = true;
            }
            command(&fixture.first, args)
        },
        || {
            checks += 1;
            Ok(())
        },
    )?;
    assert!(raced);
    assert_eq!(checks, 1);
    assert_eq!(head, fixture.remote_head()?);
    Ok(())
}

#[test]
fn an_unchanged_remote_does_not_repeat_gates() -> Result<()> {
    let fixture = Fixture::new()?;
    commit(&fixture.first, "first.txt", "first worker\n")?;
    let mut checks = 0;
    let head = publish(
        &fixture.base,
        |args| command(&fixture.first, args),
        || {
            checks += 1;
            Ok(())
        },
    )?;
    assert_eq!(checks, 0);
    assert_eq!(head, fixture.remote_head()?);
    Ok(())
}

#[test]
fn a_rejected_push_without_a_remote_change_is_not_retried() -> Result<()> {
    let fixture = Fixture::new()?;
    commit(&fixture.first, "first.txt", "first worker\n")?;
    let mut pushes = 0;
    let outcome = publish(
        &fixture.base,
        |args| {
            if args.first() == Some(&"push") {
                pushes += 1;
                return Err("branch policy rejected the push".into());
            }
            command(&fixture.first, args)
        },
        || Ok(()),
    );
    assert!(outcome.is_err());
    assert_eq!(pushes, 1);
    assert_eq!(fixture.base, fixture.remote_head()?);
    Ok(())
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // Only the uniquely created test directory is eligible for cleanup.
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn repeated_races_are_bounded_and_keep_the_local_task() -> Result<()> {
    let fixture = Fixture::new()?;
    commit(&fixture.first, "first.txt", "first worker\n")?;
    let mut pushes = 0;
    let outcome = publish(
        &fixture.base,
        |args| {
            if args.first() == Some(&"push") {
                pushes += 1;
                fixture.publish_second(&format!("second-{pushes}.txt"))?;
            }
            command(&fixture.first, args)
        },
        || Ok(()),
    );
    assert!(outcome.is_err());
    assert_eq!(pushes, 3);
    assert!(fixture.first.join("first.txt").exists());
    assert!(
        command(
            &fixture.root.join("remote.git"),
            &["show", "main:first.txt"]
        )
        .is_err()
    );
    Ok(())
}
