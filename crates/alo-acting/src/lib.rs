//! **The acting half of the application verbs**: the door the compositor
//! serves and `alo-agentd` asks.
//!
//! `alo-applications` decides all four — `open`, `focus`, `arrange`, `close` —
//! and stops one step short, in its own words: *it opens nothing, focuses
//! nothing and closes nothing: that is the acting half, it is Wayland and
//! D-Bus, and it is not this crate's.* This is that half's door.
//!
//! [ADR 0108](../../../docs/decisions/0108-the-compositor-carries-an-application-verb-out-and-needs-no-token-to.md)
//! is the shape and the argument.
//!
//! # Why there is a door at all
//!
//! The capability model is in `alo-agentd` and the windows are in the
//! compositor. Nothing else is in the way: the compositor already starts an
//! application over the bus and already activates, switches and closes windows.
//!
//! # And why no token crosses it
//!
//! `alo-broker` demands five words and an HMAC because it runs as **root**: a
//! root service must not act because a person's process said so. **The
//! compositor runs as the person**, and asking it to close a window is
//! something any of the person's processes can already do, so a door admitting
//! the person's own uid hands out nothing they did not have.
//!
//! **And the agent cannot reach it.** The person and the agent are two Unix
//! logins (`docs/contracts/daemon-protocol.md`), and the daemon refuses to
//! start where they are one. A door admitting only the person therefore admits
//! the daemon and refuses the agent — decided by the kernel, with nothing on
//! the wire taking part.
//!
//! So the authority an application verb runs under is the **turn's**, checked
//! where every other verb's is. This door is a hand, never a second judge.
//!
//! # What is here and what is not
//!
//! | | |
//! |---|---|
//! | [`Acting`] | The closed list: what may be asked of a window |
//! | [`Request`], [`Answer`] | One line in, one line out |
//! | [`NotRead`], [`NotActed`] | Why a line is not a request, and why a request did nothing |
//!
//! Not here: the socket, the compositor's windows, and the authority. The
//! socket is `alo-shell`'s to listen on and `alo-agentd`'s to connect to —
//! this crate is portable and names no path. The windows are the compositor's.
//! The authority is the turn's.
//!
//! # `arrange` is not here, and that is deliberate
//!
//! `open`, `focus` and `close` are a window named and a thing done. `arrange`
//! takes a **where** — a half, a side, a whole screen — and where a window goes
//! is `alo-arranging`'s and the design's rather than this door's. Putting it in
//! the first pass would tangle proving the road with deciding a layout.
#![forbid(unsafe_code)]

pub mod acting;
pub mod answer;
pub mod request;

pub use acting::Acting;
pub use answer::{Answer, NotActed};
pub use request::{NotRead, Request};
