//! What a portal request from an application is, and how it is judged.
//!
//! `docs/features.md`, v0.5: *applications install sandboxed and reach the
//! system through the XDG Desktop Portal interfaces … each portal request is a
//! grant in the sense of ADR 0001.* This crate is that sentence as a type. An
//! application asking to use the camera, print a document or send a
//! notification is asking for what an agent asks for when it calls a verb: a
//! bounded thing, decided against what the person granted, refused outside it,
//! and revocable.
//!
//! [ADR 0040](../../../docs/decisions/0040-what-an-applications-grant-is-over.md)
//! decided what that takes of `alo-capability`, and that crate now has it: a
//! closed list of facilities, a grantee that is an agent or an application, and
//! application grants that outlive declining the agent. This crate adds the
//! portal side and **decides nothing itself**.
//!
//! ```
//! use alo_capability::{Applicant, Facility, Grant, Grants, Reach};
//! use alo_portals::{Portal, Refused, Request};
//! use std::time::{Duration, SystemTime};
//!
//! let noon = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
//! let mut grants = Grants::default();
//! grants.grant(Grant::checked_for(
//!     &Applicant::named("org.gnome.Cheese").grantee(),
//!     Reach::Facility(Facility::Camera),
//!     noon,
//!     Duration::from_secs(60 * 60),
//! )?);
//!
//! // The camera was granted, so the camera portal is allowed…
//! let camera = Request::of("org.gnome.Cheese", Portal::Camera).expect("a request");
//! assert!(camera.judged(&grants, noon).is_ok());
//!
//! // …and the microphone was not.
//! let microphone = Request::of("org.gnome.Cheese", Portal::Microphone).expect("a request");
//! assert!(matches!(
//!     microphone.judged(&grants, noon),
//!     Err(Refused::NotAllowed { .. })
//! ));
//!
//! // An application nobody granted anything is refused before any dialog.
//! let stranger = Request::of("org.example.Stranger", Portal::Camera).expect("a request");
//! assert!(matches!(
//!     stranger.judged(&grants, noon),
//!     Err(Refused::NothingGranted { .. })
//! ));
//! # Ok::<(), alo_capability::GrantError>(())
//! ```
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`portal`] | The sixteen v0.5 portals, closed, and what each is over |
//! | [`request`] | A request: which application, which portal, what for |
//! | [`not_a_request`] | Why what arrived was never a request |
//! | [`judging`] | A request judged against the grants, and what allowed it |
//! | [`refused`] | Why a request was refused, and what a person is told |
//! | [`words`] | Every string this crate can say |
//!
//! # What is not here
//!
//! **No D-Bus, no socket and no dialog.** Task 1 of the applications plan
//! decides; task 5 serves `org.freedesktop.portal.*` from these decisions, and
//! the file chooser a portal opens is the desktop lane's. **No grant is made
//! here**: making one is a person's act, and nothing in this crate holds a
//! `&mut Grants`. **No v1 portal** — USB, global shortcuts, launchers, remote
//! desktop — is listed, refused or otherwise: a portal this machine does not
//! offer is not a portal it lists.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod judging;
pub mod not_a_request;
pub mod portal;
pub mod refused;
pub mod request;
pub mod words;

pub use judging::Allowed;
pub use not_a_request::NotARequest;
pub use portal::{Over, Portal};
pub use refused::Refused;
pub use request::{LONGEST_IDENTIFIER, Request};
pub use words::{EVERY_WORD, WordsError, declare_into, portal_words};
