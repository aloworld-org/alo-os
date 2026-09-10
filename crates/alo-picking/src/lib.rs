//! Native folder selection: the one act on this machine that makes a grant.
//!
//! ADR 0001 §3: *an agent sees no path a person has not granted it. A grant
//! comes from a deliberate act: a folder chosen in a picker.* `alo-capability`
//! has held grants since the beginning and `alo-agentd` has enforced them, and
//! until this crate **nothing on the machine could make one** — which
//! `crates/alo-agentd/src/starting.rs` says in as many words: every verb is
//! refused, correctly, because nothing has been granted and nothing can be.
//!
//! This crate is the picker as a decision, and deliberately nothing more:
//!
//! - [`Picker`] — where a person is standing, the folders in front of them,
//!   and the three moves they can make: in, up, and pick;
//! - [`Folders`] — the one road to a disk, so that every rule above is tested
//!   on a filesystem written down in a test as well as on a real one;
//! - [`Picked`] and [`Chosen`] — a folder somebody chose, sealed so nothing
//!   can invent one, and the picker's other ending: nothing;
//! - [`Granting`] — the pick, become a grant `alo_capability::Grants` holds
//!   and `alo-agentd` honours;
//! - [`NotPicked`] — every way it can fail, each with a sentence saying what
//!   to do next.
//!
//! ```
//! use std::path::Path;
//! use std::time::{Duration, SystemTime};
//!
//! use alo_capability::{Ask, Grantee, Grants};
//! use alo_picking::{Granting, NotPicked, OnThisDisk, Picker};
//!
//! # fn pick(home: &Path, folder: &str) -> Result<(), NotPicked> {
//! // A person opens the picker in their home folder and walks into one of
//! // the folders it shows.
//! let mut picker = Picker::standing_in(home, &OnThisDisk)?;
//! picker.go_into(folder, &OnThisDisk)?;
//!
//! // They pick it, and the machine has a grant it did not have before.
//! let chosen = picker.pick()?;
//! let mut grants = Grants::default();
//! let now = SystemTime::now();
//! let granting = Granting::to("@files", Duration::from_secs(60 * 60));
//! let made = granting.of(&chosen, &mut grants, now);
//! assert!(matches!(made, Ok(Some(_))));
//!
//! // The daemon's own question about the folder that was picked.
//! assert!(grants.permits(&Grantee::named("@files"), &Ask::path(home.join(folder)), now));
//!
//! // And closing it without picking grants nothing at all.
//! let nothing = granting.of(&picker.closed_without_picking(), &mut grants, now);
//! assert_eq!(nothing, Ok(None));
//! # Ok(())
//! # }
//! ```
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`browsing`] | Where a person is standing, and everything they may do |
//! | [`folders`] | What is inside a folder, as the picker may learn it |
//! | [`on_this_disk`] | The one implementation that reads a real filesystem |
//! | [`picked`] | A folder somebody chose, and closing without choosing |
//! | [`granting`] | The pick, become a grant |
//! | [`refusing`] | Every way it fails, and the sentence for each |
//! | [`words`] | Every string this crate can say, and the English beside each |
//!
//! # Four things this crate is deliberately not
//!
//! **It is not a portal, and it is not a file dialog.** `docs/features.md`
//! puts portals — including the file chooser an ordinary Linux application
//! asks for — at v0.5. This is alo OS's own surface for granting the agent a
//! folder, which is a v0.01 promise (*grants: pick a folder, see what is
//! granted, revoke it, and it expires*) and a different thing: what comes out
//! of it is a grant, not a file handed to an application.
//!
//! **It is not reachable by an agent.** There is no verb here and there could
//! not be one. A folder chooser an agent could open, drive or answer would be
//! an agent granting itself a folder, which is every other control in ADR 0001
//! defeated in one call. Every entry point here is a person's, driven by
//! whatever holds the keyboard, and `alo-protocol` has nothing that reaches
//! it.
//!
//! **It does not draw.** Nothing here has a size, a position, a row height or
//! a colour. What the picker looks like is the compositor's, exactly as
//! `alo_overlay::SurfaceRequest` leaves the overlay's appearance to whatever
//! owns the screen. What is fixed here is what a shell must not be free to
//! decide differently: what may be shown, what may be opened, what may be
//! picked, and what a person is told when any of it is refused.
//!
//! **It does not keep the grants.** [`Granting::of`] adds to an
//! `alo_capability::Grants` it is handed. Where a machine's list of grants
//! lives between one sign-in and the next is a question this crate does not
//! answer and must not answer on its own — it is the same list `alo-agentd`
//! holds, and whoever wires the two together decides where it is written.
//!
//! # Nothing here says anything in English by itself
//!
//! Every sentence a person can be shown is declared in [`words`] and answered
//! through a `said` in the language they read: the heading, the sentence
//! saying what a pick will cover, the one about a folder with more in it than
//! can be shown, and all seven refusals. There is no `Display` on
//! [`NotPicked`] that could put English on a screen by accident.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod browsing;
pub mod folders;
pub mod granting;
pub mod on_this_disk;
pub mod picked;
pub mod refusing;
pub mod words;

#[cfg(test)]
mod testing;

pub use browsing::Picker;
pub use folders::{Folders, Inside, MOST_SHOWN, NotShown};
pub use granting::Granting;
pub use on_this_disk::OnThisDisk;
pub use picked::{Chosen, Picked};
pub use refusing::NotPicked;
pub use words::{EVERY_WORD, Word, WordsError, declare_into, picking_words};
