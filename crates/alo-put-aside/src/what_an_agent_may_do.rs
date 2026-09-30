//! What an agent may do with a window because it is put aside. **Nothing.**
//!
//! Task 8's acceptance, from `docs/autonomy/putting-a-window-aside.md`:
//!
//! > A minimised state is never permission for an agent to inspect, share or act on the
//! > window — **tested as a refusal, not assumed from the absence of a call.**
//!
//! # Why this function cannot succeed
//!
//! It returns [`NotPermitted`] rather than `Result<_, NotPermitted>`. **There is no path
//! through it to a yes**, and that is the whole design: a caller cannot obtain permission
//! here, whatever it passes, because this is not where permission comes from.
//!
//! That shape is what the acceptance's last clause asks for. *Assumed from the absence of a
//! call* would be a test asserting that nothing reached the window — which passes on a
//! machine where the agent is merely not running, and goes on passing after somebody adds the
//! call. A refusal has to be **asked for and declined**, so there has to be a door, and the
//! door has to say no.
//!
//! # The two reasons, and why the second is also refused
//!
//! [`TheReasonGiven::ItIsPutAside`] is the reason this file exists to refuse. *Nobody is
//! looking at it* is an argument about attention, not about permission, and it is the one a
//! well-meaning caller reaches for: the window is out of the way, so surely reading it harms
//! nobody. A person put a window aside to stop dealing with it, which is not the same as
//! handing it over.
//!
//! [`TheReasonGiven::ThePersonGranted`] is refused too, and by a different name. This crate
//! **cannot check a grant** — grants are enumerated, visible, revocable and expiring, and
//! they live where that is true. A second place that decided them would be a second answer to
//! *may this happen*, and the weaker one would be the one an agent found first. So the answer
//! is *not here*, pointing at the surface that can.
//!
//! The result is that every call is a refusal and the type says which kind. A reader who
//! wants a yes has to go and find the thing that can give one.

/// What an agent might want to do with a window that is put aside.
///
/// The three the acceptance names. Enumerated rather than a string, because a capability this
/// crate cannot enumerate is one it cannot refuse by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Doing {
    /// Read what is in it — its contents, its document, its messages.
    Inspecting,
    /// Send it, or anything about it, anywhere.
    Sharing,
    /// Do something in it: type, click, save, send.
    ActingOn,
}

/// What a caller offers as its justification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheReasonGiven {
    /// *It is minimised, so nobody is looking at it.*
    ///
    /// **The reason this file exists.** Putting a window aside is a person deciding not to
    /// deal with it, and that is not the same as handing it over.
    ItIsPutAside,
    /// *The person granted this.*
    ///
    /// Refused here as well — not because it is a bad reason, but because **this is not where
    /// grants are checked**, and a crate that said yes to a grant it never saw would be
    /// guessing.
    ThePersonGranted {
        /// Which grant, by the name a person would recognise in the place grants are listed.
        named: String,
    },
}

/// Why not.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotPermitted {
    /// Being put aside is not permission.
    #[error(
        "{0:?} was asked for because the window is put aside, and a window put aside is a \
         person choosing not to deal with it rather than handing it over"
    )]
    BeingPutAsideIsNotPermission(Doing),
    /// A grant was offered, and this is not where grants are checked.
    #[error(
        "{0:?} was asked for under the grant {1:?}, and this crate cannot check a grant — ask \
         the surface that holds them, where a grant is visible, revocable and expires"
    )]
    GrantsAreNotCheckedHere(Doing, String),
}

/// What an agent may do with a put-aside window, which is nothing this crate can allow.
///
/// **The return type has no success case.** See this module's header: a caller cannot obtain
/// permission here whatever it passes, and the acceptance requires that to be tested as a
/// refusal rather than inferred from nothing happening.
#[must_use]
pub fn what_an_agent_may_do(doing: Doing, reason: &TheReasonGiven) -> NotPermitted {
    match reason {
        TheReasonGiven::ItIsPutAside => NotPermitted::BeingPutAsideIsNotPermission(doing),
        TheReasonGiven::ThePersonGranted { named } => {
            NotPermitted::GrantsAreNotCheckedHere(doing, named.clone())
        }
    }
}
