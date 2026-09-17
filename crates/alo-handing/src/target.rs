//! What is under the pointer when a person lets go.
//!
//! Three things, and the third is the one this crate is careful about:
//! something that takes a drop, the agent's own surface, and nothing at all.
//!
//! # A window says what it takes, and is believed about that and nothing else
//!
//! An application declares the forms it accepts, exactly as it declares the
//! forms it offers, and a drop it has no form in common with is refused rather
//! than converted — [`alo_clipboard::Offered::best_of`]'s rule, in the other
//! direction. What it does **not** get to say is who it is or whether it is
//! sandboxed: [`Application::sandboxed`] is the compositor's answer, read from
//! the sandbox the way `alo_portals::Sandboxed` reads it, because an
//! application that could call itself unsandboxed would be an application that
//! could ask for a person's paths in the clear.

use std::time::SystemTime;

use alo_clipboard::Kind;

/// Where a person let go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// A window belonging to an application.
    AWindow(Application),
    /// The agent's surface — its overlay, during one question.
    ///
    /// What lands here is offered for that question and **nothing is granted**:
    /// see [`crate::ForOneTurn`], and ADR 0001 §3, which makes a grant out of a
    /// folder chosen in a picker and a document offered at invocation, and out
    /// of nothing else.
    TheAgentsSurface {
        /// When the question the surface is open for is over. Carried at the
        /// drop rather than read from a clock later, so that nothing has to
        /// remember to throw the offer away.
        the_question_ends: SystemTime,
    },
    /// Nothing that takes a drop: the desktop, a panel, a window that accepts
    /// nothing at all.
    Nothing,
}

/// A window that takes drops, as the compositor knows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Application {
    /// Which application it is, by the identifier its sandbox names.
    id: String,
    /// Whether it runs in a sandbox. **The compositor's answer, never the
    /// application's** — it decides whether a file crosses a boundary as a path
    /// or through the documents portal.
    sandboxed: bool,
    /// Every form it says it can take, in the order it listed them.
    accepts: Vec<Kind>,
}

impl Application {
    /// A sandboxed application — every application `docs/features.md` promises
    /// to install (ADR 0005) — and the forms it takes.
    #[must_use]
    pub fn sandboxed(id: &str, accepts: Vec<Kind>) -> Self {
        Self {
            id: id.trim().to_owned(),
            sandboxed: true,
            accepts,
        }
    }

    /// An application running outside a sandbox: the person's own terminal, or
    /// a program they built themselves. It reaches their files already, so a
    /// file dropped on it is handed over as the path it is.
    #[must_use]
    pub fn outside_a_sandbox(id: &str, accepts: Vec<Kind>) -> Self {
        Self {
            id: id.trim().to_owned(),
            sandboxed: false,
            accepts,
        }
    }

    /// Which application this is.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Whether it runs in a sandbox.
    #[must_use]
    pub fn is_sandboxed(&self) -> bool {
        self.sandboxed
    }

    /// Every form it said it can take.
    #[must_use]
    pub fn accepts(&self) -> &[Kind] {
        &self.accepts
    }

    /// Whether it takes this form.
    #[must_use]
    pub fn takes(&self, form: &Kind) -> bool {
        self.accepts.contains(form)
    }
}

/// The agent's surface, during a question that ends at this moment.
///
/// The way a compositor names the overlay as a place to let go: a surface with
/// no end to its question would be an offer that outlived the turn, which is
/// the one thing a drop on the agent may never be.
#[must_use]
pub fn the_agents_surface(the_question_ends: SystemTime) -> Target {
    Target::TheAgentsSurface { the_question_ends }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A window takes the forms it listed and no others — there is no
    /// near-enough and no family match, because converting would hand an
    /// application bytes nobody said they could produce.
    #[test]
    fn a_window_takes_the_forms_it_listed_and_no_others() {
        let notes = Application::sandboxed("org.alo.Notes", vec![Kind::text()]);
        assert!(notes.takes(&Kind::text()));
        assert!(!notes.takes(&Kind::image_png()));
        assert_eq!(notes.id(), "org.alo.Notes");
        assert!(notes.is_sandboxed());
        assert_eq!(notes.accepts(), [Kind::text()]);
    }

    /// Being outside a sandbox is something the compositor says, and it is the
    /// whole difference between the two constructors.
    #[test]
    fn whether_it_is_sandboxed_is_not_something_it_names_itself() {
        let terminal = Application::outside_a_sandbox("  org.alo.Terminal  ", vec![Kind::files()]);
        assert!(!terminal.is_sandboxed());
        assert_eq!(terminal.id(), "org.alo.Terminal");
    }
}
