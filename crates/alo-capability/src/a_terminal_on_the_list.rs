//! **The person's terminal choice, carried on the list it is asked against**
//! ([ADR 0111](../../../docs/decisions/0111-the-person-chooses-whether-their-agent-may-use-the-terminal.md)).
//!
//! [`crate::ATerminalForTheAgent`] is what a person allowed their agent to do
//! with a terminal. Until 2026-11-17 every door that lets something run asked
//! the grants with [`crate::ATerminalForTheAgent::NotAtAll`] written in, so the
//! choice existed as a type and changed nothing: a terminal was refused at
//! every level whatever the person said.
//!
//! The answer now travels **on the list**, exactly as full access does
//! (`crate::full_access`): the service reads the person's file at the moment
//! it matters and asks against a copy that carries what it read. Every
//! question about reach already comes to [`Grants`], so every door — a read, a
//! proposal and the approval that runs it — follows the choice without any of
//! them having to remember to ask a second question.
//!
//! # For one moment, and never kept
//!
//! [`Grants::with_the_terminal_as`] makes a **copy**, and the grants file
//! cannot carry the answer (`#[serde(skip)]`): it lives in the person's own
//! file beside their level for running code
//! (`docs/contracts/how-code-runs-file.md`, `alo-uses-a-terminal`). So
//! switching it off takes effect at the next question, as a revoked grant
//! does, and a grant over a terminal made while it was on **permits nothing**
//! once it is off.
//!
//! # What it allows, and what it does not
//!
//! Reaching a terminal application under a grant, or at full access: reading
//! its window and pressing its controls by name. **Not typing.**
//! `accessible.type_text` refuses every terminal at every setting of this
//! choice, because what is typed into a terminal runs — that is the separate
//! choice ADR 0064's levels are about, and this does not make it.

use crate::a_terminal_for_the_agent::ATerminalForTheAgent;
use crate::grants::Grants;

impl Grants {
    /// This list, as the person's terminal choice is now — for one moment.
    ///
    /// A copy, so the list a service keeps never carries it. A copy made with
    /// [`ATerminalForTheAgent::NotAtAll`] is the list as it always was.
    #[must_use]
    pub fn with_the_terminal_as(&self, terminal: ATerminalForTheAgent) -> Self {
        let mut now = self.clone();
        now.terminal = terminal;
        now
    }

    /// What this list says the person allowed their agent to do with a
    /// terminal: [`ATerminalForTheAgent::NotAtAll`] unless it is a copy made by
    /// [`Self::with_the_terminal_as`].
    #[must_use]
    pub const fn the_terminal(&self) -> ATerminalForTheAgent {
        self.terminal
    }
}

#[cfg(test)]
#[path = "a_terminal_on_the_list_tests.rs"]
mod tests;
