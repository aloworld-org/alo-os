//! The agent overlay's summoning: one key, from anywhere.
//!
//! `docs/features.md` promises at v0.01: **★ The agent overlay: one key, from
//! anywhere** — the thing that makes alo OS an AI-native operating system
//! rather than a desktop with an assistant in a window. The key exists:
//! `alo-shortcuts` declares `alo_shortcuts::Action::TheAgent`, ships it on
//! `Super+A`, and refuses any other binding that would take its chord. What
//! did not exist is anything that happens when it is pressed.
//!
//! This crate is that seam, and deliberately nothing more:
//!
//! - [`Summoning`] — the state the chord drives, holding the two promises
//!   that make the key trustworthy: one press asks the compositor for the
//!   surface **exactly once**, and a second press while the overlay is open
//!   asks **nothing**;
//! - [`SurfaceRequest`] — the request the compositor can honour or refuse,
//!   with no rendering in it: no size, no place, no pixels;
//! - [`NotSummoned`] — the refusal in words when there is nowhere to show
//!   it, because a key that silently does nothing is the worst outcome
//!   available.
//!
//! And what the overlay **shows** once it is open and before anybody has
//! asked it anything:
//!
//! - [`AtRest`] — the whole of it, derived from what the machine holds: what
//!   would answer, what the agent may reach, and what is leaving;
//! - [`Standing`] — which of the three states that adds up to, with a
//!   sentence for each and an instruction in the two a person can act on.
//!
//! What the overlay *looks like* is the compositor's and is not modelled here
//! at all.
//!
//! ```
//! use alo_overlay::{Compositor, Pressed, SurfaceRefused, SurfaceRequest, Summoning};
//!
//! /// A compositor with room for the overlay.
//! struct Room;
//! impl Compositor for Room {
//!     fn honour(&mut self, _asked: SurfaceRequest) -> Result<(), SurfaceRefused> {
//!         Ok(())
//!     }
//! }
//!
//! let mut summoning = Summoning::closed();
//! let mut room = Room;
//!
//! // One press asks and opens; a second asks nothing at all.
//! assert!(matches!(summoning.press(Some(&mut room)), Pressed::Summoned));
//! assert!(matches!(summoning.press(Some(&mut room)), Pressed::AlreadySummoned));
//!
//! // Dismissed, the key works again.
//! summoning.dismissed()?;
//! assert!(!summoning.is_open());
//!
//! // And with no compositor, the answer is a sentence rather than silence.
//! assert!(matches!(summoning.press(None), Pressed::Refused(_)));
//! # Ok::<(), alo_overlay::NotOpen>(())
//! ```
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`summoning`] | The key's half: one press, once only, and the answer |
//! | [`surface`] | The compositor's half: the request, its refusal, the port |
//! | [`refusing`] | What the person reads when the agent cannot appear |
//! | [`resting`] | What it shows once it is up and nothing has been asked |
//! | [`standing`] | Which of the three states that is, and the sentence for it |
//! | [`answering`] | What would answer, if a question were asked now |
//! | [`granted`] | How much the agent may reach at this moment |
//! | [`quiet`] | Whether anything is leaving this machine at this moment |
//! | [`words`] | Every string this crate can say, and the English beside each |
//!
//! # Three things this crate is deliberately not
//!
//! **It is not the overlay.** Nothing here draws, sizes, places or focuses a
//! surface. Honouring a [`SurfaceRequest`] is the compositor taking
//! responsibility for the overlay appearing; every decision about how is the
//! compositor's, which is `alo-shell` and is another worker's chain. This
//! crate is what lets the key and the compositor agree on *whether* without
//! either owning the other.
//!
//! **It is not a context reader.** Summoning the agent offers nothing to it:
//! the focused window, the selection and the open document reach an agent
//! only at the moment of invocation (`CLAUDE.md`), and the summoning is not
//! an invocation — it is a surface appearing. What the overlay offers when a
//! question is actually asked is `alo-context`'s law, not this crate's.
//!
//! [`AtRest`] does not change that. What it reads is what the machine says
//! about **itself** — what it would answer with, what it may reach, what is
//! leaving it — which is nobody's content, and it reads it at the moment the
//! key is pressed rather than continuously. A background reader keeping it
//! fresh would be a bug in this product, not a feature request.
//!
//! **It is not a verb.** A person pressing a key on their own machine needs
//! no grant and is never proposed for approval; there is no connection
//! between this crate and `alo-capability`, and that is not an omission —
//! `alo-shortcuts` says the same of the chord itself.
//!
//! # Nothing here says anything in English by itself
//!
//! Every sentence a person can be shown is declared in [`words`] and answered
//! through a `said` in the language they read — the two refusals, the three
//! states, and the three readings under them. The one exception is
//! [`NotOpen`], which reports the shell wiring disagreeing with this model —
//! alo OS's own bug, read by whoever is fixing it, in the same deliberate
//! English as `alo_shortcuts::DefaultsError`.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod answering;
pub mod granted;
pub mod quiet;
pub mod refusing;
pub mod resting;
pub mod standing;
pub mod summoning;
pub mod surface;
pub mod words;

#[cfg(test)]
mod testing;

pub use answering::WouldAnswer;
pub use granted::Granted;
pub use quiet::Quiet;
pub use refusing::NotSummoned;
pub use resting::AtRest;
pub use standing::Standing;
pub use summoning::{NotOpen, Pressed, Summoning};
pub use surface::{Compositor, SurfaceRefused, SurfaceRequest};
pub use words::{Counted, Word, WordsError, declare_into, overlay_words};
