//! **What `alo-app` asks the desktop**, through the compositor's door
//! (`docs/contracts/application-door.md`).
//!
//! Five things, and every word of them is `alo-acting`'s, never spelt here:
//! which window was in front before this terminal (`front`), where alo OS's
//! own tree is (`alo-tree`), the three acts on a window (`open`, `focus`,
//! `close`), and a count of what an application sends while one act runs
//! (`count`, ADR 0148).
//!
//! A trait, so that every refusal can be tested without a desktop; the one
//! that reaches a real socket is [`crate::at_the_door`].

use alo_acting::{Answer, Counted, InFront, Request, ToCount, WhereAlosTreeIs};

/// Why the desktop gave no answer that can be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unanswered {
    /// Nothing was sent: no door to knock at, or it would not open.
    NotReached,
    /// It was sent, and what came back was nothing, or nothing this command's
    /// vocabulary knows — which is never read as success.
    NotUnderstood,
}

/// A count of what one application sends, held open while one act runs.
pub trait HeldCount {
    /// Say the act is done, and read what was counted.
    ///
    /// [`None`] when nothing usable came back.
    fn done(self: Box<Self>) -> Option<Counted>;
}

/// Everything `alo-app` asks the desktop.
pub trait TheDoor {
    /// Which window the person was working in before this terminal.
    ///
    /// # Errors
    /// [`Unanswered`].
    fn front(&mut self) -> Result<InFront, Unanswered>;

    /// Where alo OS's own surfaces are on the accessibility bus.
    ///
    /// # Errors
    /// [`Unanswered`].
    fn alos_tree(&mut self) -> Result<WhereAlosTreeIs, Unanswered>;

    /// One act on an application's window.
    ///
    /// # Errors
    /// [`Unanswered`]: [`Unanswered::NotUnderstood`] means the act was sent.
    fn act(&mut self, request: &Request) -> Result<Answer, Unanswered>;

    /// Begin counting what this application sends.
    ///
    /// [`None`] unless the desktop answered `counting`: a caller never reads
    /// anything else as a count having begun.
    fn begin_counting(&mut self, asked: &ToCount) -> Option<Box<dyn HeldCount>>;
}
