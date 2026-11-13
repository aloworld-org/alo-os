//! **What runs a publication's command, and what it reaches**, handed in by
//! whoever owns the machine — so this crate starts no program itself.
//!
//! ADR 0117 put the one place that starts a program in `alo-running`, and the
//! command road of ADR 0149 runs through it: `/bin/sh -c`, as the person, told
//! that nobody is there to answer, for at most two minutes. A publication is a
//! command on that road whose line the machine wrote rather than a model, so
//! this crate asks for something that runs a line and nothing more. The daemon
//! hands in `alo_running::run_as_the_person_around`; a test hands in the same
//! function with a stand-in for `gh`.
//!
//! **What it sends is counted the way every command alo runs is** (ADR 0148,
//! as amended 2026-11-13: *"Yes, count alo's commands"*): the caller's
//! [`AroundACommand`] starts the command inside a control group of its own and
//! looks at the kernel's count while it runs, so the egress indicator shows it
//! sending and the record keeps how many bytes left. There is no second road
//! for a publication's bytes.

use alo_capability::{ACommand, AroundACommand, ItDidNotStart, WhatACommandDid};

/// Runs one command as the person.
pub trait Runs {
    /// Run `line` once, as the person, started and watched by `around`.
    ///
    /// # Errors
    /// [`ItDidNotStart`] when the shell could not be started.
    fn run(
        &self,
        line: &ACommand,
        around: &mut dyn AroundACommand,
    ) -> Result<WhatACommandDid, ItDidNotStart>;
}

/// The programs a publication is carried out with, and the GitHub it reaches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Through {
    /// GitHub's own command, as the shell finds it.
    gh: String,
    /// Where a repository's address starts.
    github: String,
}

/// GitHub's own command, as the image installs it on the path.
pub const GH: &str = "gh";

/// Where every repository on GitHub is.
pub const GITHUB: &str = "https://github.com";

impl Through {
    /// `gh` on the path, and github.com — what every machine uses.
    #[must_use]
    pub fn on_github_com() -> Self {
        Self::at(GH, GITHUB)
    }

    /// `gh` at `gh`, and repositories under `github` — a test's own stand-ins.
    #[must_use]
    pub fn at(gh: &str, github: &str) -> Self {
        Self {
            gh: gh.to_owned(),
            github: github.trim_end_matches('/').to_owned(),
        }
    }

    /// GitHub's own command.
    #[must_use]
    pub fn gh(&self) -> &str {
        &self.gh
    }

    /// Where a repository's address starts.
    #[must_use]
    pub fn github(&self) -> &str {
        &self.github
    }
}
