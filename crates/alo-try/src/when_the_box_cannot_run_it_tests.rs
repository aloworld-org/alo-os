//! The plain sentence for the refused start on a composefs machine, and for
//! nothing else.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::path::PathBuf;

use alo_running::NotRun;
use alo_running::the_helpers_word::StartRefused;
use alo_strings::{Strings, Vocabulary};

use super::WhatItSays;

/// The words `alo-try` speaks.
fn strings() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// The start the box denied after it was joined.
fn denied() -> NotRun {
    NotRun::TheBoxRefusedTheCommand {
        program: PathBuf::from("/usr/bin/ls"),
        refused: StartRefused::PermissionDenied,
    }
}

/// **The refused start on a composefs machine says so plainly**, naming the
/// program and where to go instead.
#[test]
fn a_denied_start_on_composefs_says_so_plainly() {
    let not_run = denied();
    let says = WhatItSays::of(&not_run, true);
    assert_eq!(
        says,
        WhatItSays::NotOnThisComputerYet {
            program: PathBuf::from("/usr/bin/ls")
        }
    );
    assert_eq!(
        says.said(&not_run, &strings()),
        "/usr/bin/ls was not run. The sealed box can't run programs on this computer yet. \
         Choose 'ask each time' in Settings → Running code to run commands after you approve \
         them."
    );
}

/// **Every other refusal keeps its own sentence**: the same start off
/// composefs, a start refused for another reason, and a box that could not be
/// built at all.
#[test]
fn every_other_refusal_keeps_its_own_sentence() {
    let strings = strings();
    let otherwise = NotRun::TheBoxRefusedTheCommand {
        program: PathBuf::from("/usr/bin/ls"),
        refused: StartRefused::Otherwise,
    };
    let unbuilt = NotRun::NothingToReach;
    for (not_run, composefs) in [(denied(), false), (otherwise, true), (unbuilt, true)] {
        let says = WhatItSays::of(&not_run, composefs);
        assert_eq!(says, WhatItSays::ItsOwn, "{not_run}");
        let said = says.said(&not_run, &strings);
        assert!(
            !said.contains("can't run programs on this computer yet"),
            "{said}"
        );
    }
}
