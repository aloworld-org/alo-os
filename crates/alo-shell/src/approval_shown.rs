//! The compositor's half of `alo-approving`: what is on the screen.
//!
//! `alo_approving::Compositor` is the port whatever owns the screen implements
//! to be handed a question. This is that implementation, and it is
//! deliberately only a place to keep what it was handed: the `Asked` —
//! sentence, agent and number, made by `alo-approving` from a change that is
//! really waiting — and nothing else. There is no way to put a question here
//! except to be handed one through that port, so there is no way to draw a
//! sentence that was not the proposal's own.

use alo_approving::{Asked, Compositor, SurfaceRefused};

/// The question on the screen, as the compositor was last handed it.
#[derive(Debug)]
pub(crate) struct ApprovalShown {
    /// Whether this compositor has an output to put a question on.
    output: bool,
    /// The question it was handed, until it is taken down.
    asked: Option<Asked>,
}

impl ApprovalShown {
    /// A compositor drawing on an output.
    pub(crate) fn on_an_output() -> Self {
        Self {
            output: true,
            asked: None,
        }
    }

    /// A compositor with nothing to put a question on: a headless session.
    pub(crate) fn with_no_output() -> Self {
        Self {
            output: false,
            asked: None,
        }
    }

    /// The question on the screen, if one is.
    pub(crate) fn asked(&self) -> Option<&Asked> {
        self.asked.as_ref()
    }

    /// Take the question down, because it was answered.
    pub(crate) fn taken_down(&mut self) {
        self.asked = None;
    }
}

impl Compositor for ApprovalShown {
    fn ask(&mut self, asked: Asked) -> Result<(), SurfaceRefused> {
        if !self.output {
            self.asked = None;
            return Err(SurfaceRefused::NothingToShowOn);
        }
        self.asked = Some(asked);
        Ok(())
    }
}
