//! What the backend reads about this machine at every request.
//!
//! The grants a person made, and what *what opens what* is answered from. Both
//! are read **at every request** and never held between them, which is what
//! makes a revocation felt at an application's next request rather than at the
//! backend's next restart: there is no copy here to go stale.
//!
//! Either can fail to read — a grants file that is not there, a person's
//! choices that do not parse — and the backend then refuses the request
//! ([`crate::Unanswered::GrantsUnread`], [`crate::Unanswered::ApplicationsUnread`])
//! rather than answering from nothing. An empty list of grants would refuse
//! everything anyway; an empty list of choices would open a file in an
//! application the person chose against, which is the worse mistake.

use alo_applications::{Chosen, Declared, Installed, WhatOpensWhat};
use alo_capability::Grants;

/// What the backend reads about this machine.
pub trait TheMachine: Send + Sync {
    /// The grants, as they are now — [`None`] when they cannot be read.
    fn grants(&self) -> Option<Grants>;

    /// What is installed, what applications declare and what the person
    /// chose, as they are now — [`None`] when any of them cannot be read.
    fn applications(&self) -> Option<Applications>;
}

/// The three things *what opens this* is answered from, read together.
#[derive(Debug, Clone, Default)]
pub struct Applications {
    /// What is installed.
    pub installed: Installed,
    /// What the installed applications declare they open.
    pub declared: Declared,
    /// What the person chose.
    pub chosen: Chosen,
}

impl Applications {
    /// *What opens what*, answered from these.
    #[must_use]
    pub const fn what_opens(&self) -> WhatOpensWhat<'_> {
        WhatOpensWhat::on(&self.installed, &self.declared, &self.chosen)
    }
}
