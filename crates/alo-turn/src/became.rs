//! What became of a change a turn put to the person.
//!
//! A change waits for one approval (ADR 0001 §5), and whoever asked for it
//! is told the number it waits under and nothing more at that moment. This
//! is what they may ask afterwards: whether the person approved it, declined
//! it, or let it lapse — or whether nothing of that number was ever put to
//! them. It is remembered by the turn that put the change, because the
//! change is that turn's: a change nobody answered goes away with its turn,
//! and so does this.
//!
//! Additive, and it decides nothing: asking what became of a change is a
//! read of the turn's own memory, asks the grants nothing, and runs nothing.
//! The plan for the local network names it so that an agent on a paired
//! machine is not left guessing whether a file moved on a machine down the
//! corridor; a local agent may ask the same of its own turn.

/// What became of one change a turn put to the person.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Became {
    /// It is still waiting for the person.
    Waiting,
    /// The person approved it. Whether the machine then could is in the
    /// record, which is where what ran is always answered from.
    Approved,
    /// The person declined it.
    Declined,
    /// Nobody answered it in the time it stood, and it can no longer be.
    Lapsed,
    /// No change of that number was put to the person by this turn.
    NothingWaiting,
}
