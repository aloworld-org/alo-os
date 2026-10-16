//! **A privileged thing in the code must be granted by the unit that runs it**,
//! and nothing compared those two files until this test did.
//!
//! # What this cost, and why a comment would not have stopped it
//!
//! On 2026-10-14 this crate was taught to start a person's desktop as them,
//! with `Command::uid()`. It compiled, clippy passed, rustdoc passed, 72 tests
//! passed, CI passed, and it merged. On a booted machine it said:
//!
//! ```text
//! alo-sessiond: this machine would not start /usr/bin/alo-desktop
//!               for account 1000: Permission denied (os error 13)
//! ```
//!
//! The reason was in a file that had been read and not compared:
//!
//! ```text
//! alo-sessiond.service:42   CapabilityBoundingSet=      <- nothing granted
//! alo-sessiond.service:44   NoNewPrivileges=yes
//! ```
//!
//! **A process with no capabilities cannot become another user.** Both files
//! were individually right: the unit is correct to grant nothing — ADR 0018's
//! *one privileged component* argument — and the code is a reasonable thing to
//! write. The fault lived in the gap between them, which is exactly where no
//! test was looking.
//!
//! An evening of work was built on an approach a thirty-second reading would
//! have disqualified. This is that reading, run at the gate.
//!
//! # If this test has stopped you
//!
//! You have written code that changes uid or gid in a crate whose unit grants
//! no capability to do it. **Adding the capability is a decision, not a fix**:
//! this unit holds none deliberately, and a privileged process that can become
//! anybody is the thing ADR 0018 exists to prevent. Either the work belongs in
//! a component that already has the privilege, or it does not need to change
//! user at all — the desktop, in the end, did not.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// The repository root, from this crate's own manifest directory.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("the repository root")
        .to_path_buf()
}

/// The unit that runs this crate on a machine.
const THE_UNIT: &str = "image/usr/lib/systemd/system/alo-sessiond.service";

/// Ways code becomes somebody else.
const BECOMING_SOMEBODY_ELSE: [&str; 3] = [".uid(", ".gid(", "setuid"];

/// Every `.rs` file in this crate's `src`.
fn every_source(at: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(at) else {
        return found;
    };
    for entry in entries {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            found.extend(every_source(&path));
        } else if path.extension().is_some_and(|it| it == "rs") {
            found.push(path);
        }
    }
    found
}

#[test]
fn nothing_here_becomes_another_user_while_its_unit_grants_no_capability() {
    let repository = the_repository();
    let unit_at = repository.join(THE_UNIT);
    let unit = std::fs::read_to_string(&unit_at)
        .unwrap_or_else(|why| panic!("this crate's own unit at {}: {why}", unit_at.display()));

    // **The positive control.** If the unit cannot be read as expected, every
    // conclusion below is drawn from an empty string.
    assert!(
        unit.contains("ExecStart="),
        "{} does not look like a unit file, so what follows proves nothing",
        unit_at.display()
    );

    // An empty `CapabilityBoundingSet=` grants nothing. A line with anything
    // after the `=` grants something, and then this test's premise is gone and
    // whoever changed it should say so here.
    let grants_nothing = unit
        .lines()
        .any(|line| line.trim() == "CapabilityBoundingSet=");
    let no_new_privileges = unit
        .lines()
        .any(|line| line.trim() == "NoNewPrivileges=yes");

    if !grants_nothing && !no_new_privileges {
        // The unit has been given privileges deliberately. That is a decision
        // somebody made, and this test is not the place to second-guess it.
        return;
    }

    let mut asking: Vec<String> = Vec::new();
    for file in every_source(&repository.join("crates/alo-sessiond/src")) {
        let text = std::fs::read_to_string(&file).expect("this crate's own source");
        for (number, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or(line);
            for way in BECOMING_SOMEBODY_ELSE {
                if code.contains(way) {
                    asking.push(format!(
                        "{}:{}  {}",
                        file.strip_prefix(&repository).unwrap_or(&file).display(),
                        number + 1,
                        line.trim()
                    ));
                }
            }
        }
    }

    assert!(
        asking.is_empty(),
        "this crate changes uid or gid, and `{THE_UNIT}` grants it no capability to do so \
         (`CapabilityBoundingSet=` is empty, `NoNewPrivileges=yes`):\n  {}\n\n\
         **A process with no capabilities cannot become another user.** On 2026-10-14 this exact \
         pair compiled, passed clippy, rustdoc, 72 tests and CI, merged, and then failed on a \
         booted machine with `Permission denied (os error 13)` — after an evening of work built \
         on an approach that a thirty-second reading of the unit would have disqualified.\n\n\
         Adding the capability is a decision and not a fix: this unit holds none deliberately, \
         and ADR 0018's *one privileged component* argument is why. Either the work belongs \
         somewhere that already has the privilege, or it does not need to change user at all.",
        asking.join("\n  ")
    );
}
