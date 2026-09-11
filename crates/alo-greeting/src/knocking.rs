//! The opener's door, as the greeter sees it.
//!
//! One method, and what it carries is `alo_sessiond::Knock` — a `u32` and
//! nothing else, with no field for a name, a path or a password. That is the
//! whole of what keeps a password off the wire, and it is that crate's shape
//! rather than a rule kept here: this trait cannot widen it, because there is
//! nothing on the type to widen.
//!
//! A trait for the reason `alo_changing::Knocking` is one: the composition in
//! [`crate::Greeting`] has to be testable against a door with a counter behind
//! it — *how many knocks, and for whom* — without a test having to stand a
//! privileged service up. `TheOpenersDoor` (`src/door.rs`) is the only one
//! that ships, and `tests/the_greeter_knocks_at_a_real_door.rs` runs the
//! composition against a real socket with a real `alo-sessiond` door on the
//! other side of it.

use alo_sessiond::{Answered, Knock};

use crate::refusing::NotAnswered;

/// Somewhere a session can be asked for.
pub trait Knocking: std::fmt::Debug {
    /// Knock, and hear what the door said.
    ///
    /// # Errors
    ///
    /// [`NotAnswered`] for every way the conversation fails to *happen* — a
    /// door nobody is behind, a connection that drops, an answer that never
    /// comes, a line that is not one. A door that heard the knock and refused
    /// it is `Ok(Answered::Refused)` and not an error: that is the machine
    /// answering, and the sentence a person reads is the one it chose.
    fn knock(&self, knock: Knock) -> Result<Answered, NotAnswered>;
}
