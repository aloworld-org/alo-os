//! What becomes of a notification that arrives: shown, or held.
//!
//! **Held while locked, with no variant a setting could turn into a preview.**
//! A notification is the thing on a screen most likely to put a private
//! sentence in front of somebody else — the first line of a message, the name
//! of a sender, a calendar entry — and the lock screen is the screen most
//! likely to have somebody else in front of it. So there are two answers and
//! the second one carries nothing back to draw:
//!
//! ```compile_fail
//! let preview = alo_locking::Arrived::<String>::Preview;
//! ```

/// What one arriving notification became.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arrived<N> {
    /// The session is unlocked: here it is, for whatever shows notifications.
    ToShow(N),
    /// The machine is locked: it is held for the person and nothing is shown.
    /// It is handed to them when they unlock, in the order it arrived.
    Held,
}
