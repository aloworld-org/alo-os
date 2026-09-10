//! The egress indicator, on a screen.
//!
//! Law 1: *nothing leaves silently.* `alo-egress` is that law as working code —
//! it decides what may leave, and it puts every departure on a list **before**
//! it hands back the permission to open a connection, so there is no order of
//! operations in which something is permitted and unshown. What did not exist
//! is anywhere a person can actually see that list. `ROADMAP.md`'s v0.01 exit
//! gate requires the indicator to have **stayed dark** through a working day,
//! which until now was a claim about a surface nobody could look at.
//!
//! This crate is that surface's model, and deliberately nothing more:
//!
//! - [`Lamp`] — dark or lit, made from an `alo_egress::Indicator` and from
//!   nothing else, with no constructor anywhere that takes a count;
//! - [`Drawn`] — the light and the readable lines under it, from one moment, so
//!   the two cannot disagree;
//! - [`Compositor`] — the port a compositor implements to be handed one, with
//!   no rendering in it;
//! - [`Indicating`] — what is on the screen, kept in step with what is leaving:
//!   one change, one redraw, and nothing shown twice;
//! - [`NotShown`] — the refusal in words when there is nowhere to show it,
//!   because a machine that cannot show what is leaving and says nothing about
//!   it looks exactly like a machine on which nothing is leaving.
//!
//! ```
//! use alo_capability::Grantee;
//! use alo_egress::{Destination, EgressPolicy, Indicator, Leaving, Why};
//! use alo_indicator::{Compositor, Drawn, Drew, Indicating, NotShown, SurfaceRefused};
//! use std::time::{Duration, SystemTime};
//!
//! /// A compositor with a screen, keeping whatever it was last given.
//! #[derive(Default)]
//! struct Screen {
//!     showing: Option<Drawn>,
//! }
//! impl Compositor for Screen {
//!     fn show(&mut self, drawn: Drawn) -> Result<(), SurfaceRefused> {
//!         self.showing = Some(drawn);
//!         Ok(())
//!     }
//! }
//!
//! let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
//! let mut screen = Screen::default();
//! let mut indicating = Indicating::nowhere();
//! let mut indicator = Indicator::default();
//!
//! // A day answered on this machine: the indicator is up, and it is dark.
//! assert_eq!(indicating.show(Some(&mut screen), &indicator), Drew::Shown);
//! assert!(screen.showing.as_ref().is_some_and(|drawn| drawn.lamp().is_dark()));
//!
//! // One question goes elsewhere, and the light is on while it is in flight.
//! let departing = indicator.beginning(
//!     &EgressPolicy::Anywhere,
//!     Leaving::because(
//!         &Grantee::named("@files"),
//!         Why::Fetching,
//!         Destination::at("alo.example").expect("a host that can be shown"),
//!     ),
//!     now,
//! )?;
//! assert_eq!(indicating.show(Some(&mut screen), &indicator), Drew::Shown);
//! assert!(screen.showing.as_ref().is_some_and(|drawn| drawn.lamp().is_lit()));
//!
//! // It finishes, and the machine is dark again.
//! indicator.ended(departing);
//! assert_eq!(indicating.show(Some(&mut screen), &indicator), Drew::Shown);
//! assert!(screen.showing.as_ref().is_some_and(|drawn| drawn.lamp().is_dark()));
//!
//! // With nowhere to show it, the answer is a sentence rather than silence.
//! assert_eq!(
//!     indicating.show(None, &indicator),
//!     Drew::Refused(NotShown::NoCompositor),
//! );
//! # Ok::<(), alo_egress::NotPermitted>(())
//! ```
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`lamp`] | The light: dark, or lit for this many things |
//! | [`drawn`] | The light and the lines under it, from one moment |
//! | [`surface`] | The compositor's half: what it is given, and how it refuses |
//! | [`indicating`] | What is on the screen, kept in step with what is leaving |
//! | [`refusing`] | What a person reads when there is nowhere to show it |
//! | [`words`] | Every string this crate can say, and the English beside each |
//!
//! # There is one door, and it is the machine's own indicator
//!
//! The plan's acceptance for this task is a sentence about provenance — *the
//! indicator is drawn from `alo_egress::Indicator` and nothing else*, and *it
//! cannot be drawn from a value that was not a departure* — so provenance is
//! what the types here are shaped around rather than what their documentation
//! promises.
//!
//! [`Lamp::of`] and [`Drawn::of`] take `&alo_egress::Indicator`. Neither type
//! has a public field, a `From`, a deserialiser or a constructor from a count,
//! and [`Indicating::show`] makes the picture itself rather than accepting one,
//! so a compositor can only ever be handed something that came off the
//! machine's own list. That list is only ever added to by the two calls in
//! `alo-egress` that hand back permission to open a connection, and an egress a
//! policy refused never reaches it — because nothing left, and a light that lit
//! for the connections a rule stopped would teach people to ignore the one
//! thing on the screen that matters.
//!
//! The compile-fail examples on [`Lamp`] and [`Drawn`] are that argument as
//! tests: a second door added later stops being a design discussion and starts
//! being a failing build.
//!
//! # Three things this crate is deliberately not
//!
//! **It is not the indicator's decision.** Nothing here permits, refuses,
//! counts or records an egress. `alo-egress` does all of it, and this crate
//! cannot even see a destination — only the `alo_egress::Shown` that carries
//! one, worded by the crate that decided it.
//!
//! **It does not draw anything.** No size, no place, no colour, no icon, no
//! z-order, no animation. *Dark* and *lit* are states; which pixels say so is
//! the compositor's, and `docs/features.md` puts the indicator's eventual home
//! in the dock's status area at v0.5 — furniture this value does not need to
//! know about.
//!
//! **It is not a capability.** A person looking at their own machine to find
//! out what is leaving it is not an agent doing something, so there is no verb,
//! no grant and no approval here, and no connection to `alo-capability` at all.
//! An indicator an agent could ask to be turned off would not be an indicator.
//!
//! # Nothing here says anything in English by itself
//!
//! Every sentence a person can be shown is declared in [`words`] and answered
//! through a `said` in the language they read: the two refusals, and the two
//! readings of the light itself. The lines are `alo-egress`'s sentences,
//! answered through its own vocabulary.
//!
//! There is no diagnostic English anywhere in this crate — no `NotOpen` of the
//! kind the agent overlay's summoning needed — because there is no way for a
//! compositor and this model to disagree about whether the indicator exists:
//! nothing here is told that a surface went away. It finds out by being refused
//! the next time it asks, which is a fact rather than a report.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod drawn;
pub mod indicating;
pub mod lamp;
pub mod refusing;
pub mod surface;
pub mod words;

#[cfg(test)]
mod testing;

pub use drawn::Drawn;
pub use indicating::{Drew, Indicating};
pub use lamp::Lamp;
pub use refusing::NotShown;
pub use surface::{Compositor, SurfaceRefused};
pub use words::{Counted, Word, WordsError, declare_into, indicator_words};
