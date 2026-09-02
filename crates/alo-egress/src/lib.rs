//! What an agent causes to leave this machine, and what it may not.
//!
//! This crate is **law 1** as working code: *nothing leaves silently.* Every
//! network egress an agent causes is decided against a policy and visible at the
//! moment it happens, and the two are one act rather than two —
//! [`Indicator::beginning`] asks the policy and shows the result, and there is
//! no other way to obtain the [`Departing`] a caller needs before it opens a
//! connection.
//!
//! ```
//! use alo_capability::Grantee;
//! use alo_egress::{Destination, EgressPolicy, Indicator, Leaving};
//! use alo_models::{InferenceSource, Region};
//! use std::time::{Duration, SystemTime};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
//! let mut indicator = Indicator::default();
//!
//! // A question answered on this machine is not a departure at all, so there
//! // is nothing to decide about and nothing to show.
//! let mail = Grantee::named("@mail");
//! assert!(Leaving::asking(&mail, &InferenceSource::ThisMachine).is_err());
//! assert!(indicator.is_quiet());
//!
//! // One answered elsewhere is, and the person is told while it happens.
//! let elsewhere = InferenceSource::Hosted {
//!     provider: "alo".to_owned(),
//!     region: Region::Declared("the EU".to_owned()),
//! };
//! let departing = indicator.beginning(
//!     &EgressPolicy::Anywhere,
//!     Leaving::asking(&mail, &elsewhere)?,
//!     now,
//! )?;
//! assert_eq!(
//!     indicator.showing()[0].describe(),
//!     "@mail is asking a question of alo, in the EU",
//! );
//!
//! indicator.ended(departing);
//! assert!(indicator.is_quiet());
//! # Ok(())
//! # }
//! ```
//!
//! # What is here, and what is not
//!
//! **The decision and the indicator.** [`EgressPolicy`] is what an organisation
//! permits, [`Leaving`] is one egress about to happen, [`Indicator`] is what a
//! person sees while it does, and [`Departing`] is the only thing that means
//! *this may leave*.
//!
//! **Not the enforcement.** Making this true at the network boundary — so that
//! code which never asked cannot open a socket either — is Linux, and it is a
//! later item. What this crate guarantees is that the ordinary path cannot
//! permit an egress without showing it, which is the mistake that actually gets
//! made.
//!
//! **Not the record.** What left this machine *earlier* is a question for
//! `alo-record`, and writing egress down is its own decision: an answer from a
//! provider is both a departure and where an answer came from, and a record
//! that kept it as two entries would count one departure twice. A
//! [`Departing`] carries everything such an entry needs — who, where, why and
//! when — decided once at the moment it was allowed.
//!
//! # Why this is wider than `alo-models`
//!
//! [`InferenceSource`](alo_models::InferenceSource) and
//! [`SourcePolicy`](alo_models::SourcePolicy) already answer where a *question*
//! may be answered, and inference is the largest egress this product will ever
//! cause. It is not the only one: a verb that fetches something, an adapter that
//! sends something, and a paired machine reached for anything at all are egress
//! too, and an indicator with a gap in it for the case nobody thought of is not
//! an indicator.
//!
//! So this crate widens the boundary and keeps the rule single. An
//! [`EgressPolicy`] is made from a
//! [`SourcePolicy`](alo_models::SourcePolicy) rather than stated a second time,
//! and that the two agree about every source there is, is a test in
//! [`policy`] rather than an intention.
//!
//! # Telling the time
//!
//! Nothing here reads the clock, as in `alo-capability` and `alo-record` and for
//! the same reason: the moment is passed in, so what a person saw on the
//! indicator and what a record says about it cannot disagree about when it
//! happened.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod departing;
pub mod destination;
pub mod indicator;
pub mod leaving;
pub mod policy;

pub use departing::Departing;
pub use destination::{Destination, DestinationError};
pub use indicator::{Indicator, Shown, ShownId};
pub use leaving::{Leaving, Why};
pub use policy::{EgressPolicy, NotPermitted};
