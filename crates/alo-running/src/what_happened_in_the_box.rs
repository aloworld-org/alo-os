//! What a run did, in a form the record can take.
//!
//! ADR 0064 keeps three protections on at every level, and
//! `alo_capability::AlwaysOn` lists them. Two of them — the record and the
//! snapshot — are kept by whoever called this crate, not by this crate. So the
//! obligation this file carries is to make keeping them **possible**: a caller
//! that cannot say what ran cannot record what ran.
//!
//! # Why this exists rather than an exit status being returned
//!
//! A status alone says how a command ended and nothing about what it was. The
//! record has to carry what was run, what it was allowed to reach, and how it
//! finished, because *the agent ran something* and *the agent ran this, bounded
//! to these folders* are the same sentence to a program and completely
//! different sentences to the person reading their own record a week later.
//!
//! **And it says it ran in a box.** [`WhatHappened::the_kernel_was_holding_it`]
//! is not decoration: `docs/features.md` promises that at full trust a run whose
//! boundary could not be applied may happen anyway, *told that the kernel is not
//! watching it, and the record says so*. A record that cannot distinguish the
//! two cases cannot keep that promise, so the field is here from the first
//! version even though this crate only ever sets it true.

use std::{ffi::OsString, path::PathBuf, process::ExitStatus};

/// What one run did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatHappened {
    /// The command that ran.
    pub program: PathBuf,
    /// What it was given, unchanged.
    pub arguments: Vec<OsString>,
    /// Everywhere the kernel let it reach.
    pub was_allowed: Vec<PathBuf>,
    /// How it ended.
    pub ended: ExitStatus,
    /// Whether the kernel was holding the boundary while it ran.
    ///
    /// Always true from this crate, which runs nothing it cannot bound. It is a
    /// field rather than an assumption because the record has to be able to say
    /// *the kernel was not watching this one* about a run at full trust, and a
    /// record that only ever says one thing cannot say the other.
    pub the_kernel_was_holding_it: bool,
}

impl WhatHappened {
    /// Whether the command itself reported success.
    ///
    /// Separate from whether the **run** succeeded, which is the difference
    /// `crate::NotRun` draws: a program that returns 1 ran perfectly and said
    /// no, and only one of those two is this crate's business.
    #[must_use]
    pub fn the_command_was_happy(&self) -> bool {
        self.ended.success()
    }
}
