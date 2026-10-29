//! **How a date, a time and a number are written, per language, from CLDR.**
//!
//! `ROADMAP.md` v0.5's *Language*, the half that is not `alo-strings`, and task
//! 6 of `docs/autonomy/access-and-language-plan.md`.
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
//! a person changes either without the other. Since 2026-10-28 they change the
//! region in Settings, from [`the_regions_named_in`], and it is kept in their
//! own folder by [`keeping`] — **the region alone**: the words are always in the
//! language they read now, so a person who changes language gets its month
//! names in the same region's order.
//!
//! # One answer for every surface
//!
//! `alo_shell::TheirFormats` is what a clock, the record and the lock screen
//! ask: what the person chose, the language they read, the zone they are in,
//! and the one rule for where they chose nothing. It lives in `alo-shell`
//! rather than here because it turns a moment into a day with `jiff`, and
//! this crate is reached by every daemon through `alo-saying`: `jiff`'s graph
//! reaches `defmt`, which links a system library a musl image does not have
//! (`crates/alo-image/tests/no_daemon_links_a_system_library.rs`).
//!
//! # A timezone is not looked up behind somebody's back
//!
//! The timezone is what the person chose. Following the network is off until
//! they turn it on, because it is a request that leaves the machine (law 1).

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod cannot_reach;
pub mod keeping;
pub mod region_code;
pub mod region_names;
pub mod regionally;
pub mod regions;
pub mod set_up_in;
pub mod the_machines_zone;
pub mod timezone;
mod unkept;
pub mod where_the_machine_is;
pub mod words;
pub mod zones;

pub use cannot_reach::{Missing, what_cldr_has_that_this_cannot_reach};
pub use keeping::Chosen;
pub use region_code::RegionCode;
pub use region_names::{NamedRegion, the_regions_named_in};
pub use regionally::{FirstDay, NotARegion, Regionally, THE_SOURCE_LANGUAGE};
pub use timezone::{FollowingTheNetwork, Timezone};
pub use unkept::{FileNotRead, FileNotWritten};
pub use words::{WordsError, declare_into, formats_words};
