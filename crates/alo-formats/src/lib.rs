//! **How a date, a time and a number are written, per language, from CLDR.**
//!
//! `ROADMAP.md` v0.5's *Language*, the half that is not `alo-strings`, and task
//! 6 of `docs/autonomy/v0-5-access-and-language-plan.md`.
//!
//! # Rented, never written down here
//!
//! How 17 September 2026 is written in Maltese is Unicode's answer, kept
//! current by people who do that work. A table of our own would be a second
//! answer that drifts, and it would drift first in the languages nobody in this
//! repository reads — which is the failure this whole plan exists to prevent. So
//! the formats come from `icu`, the CLDR implementation, rented and never
//! edited (ADR 0011).
//!
//! **What that crate does not expose is written down rather than typed out**:
//! [`what_cldr_has_that_this_cannot_reach`].
//!
//! # A language is not a country
//!
//! A Portuguese speaker in Belgium reads Portuguese and writes dates the
//! Belgian way, so [`Regionally`] carries the language and the region apart, and
//! a person changes either without the other.
//!
//! # A timezone is not looked up behind somebody's back
//!
//! The timezone is what the person chose. Following the network's idea of where
//! this machine is costs a network call, so it is off until they turn it on
//! (law 1: nothing leaves silently).

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod cannot_reach;
pub mod keeping;
pub mod regionally;
pub mod timezone;

pub use cannot_reach::{Missing, what_cldr_has_that_this_cannot_reach};
pub use regionally::{FirstDay, NotARegion, Regionally};
pub use timezone::{FollowingTheNetwork, Timezone};
