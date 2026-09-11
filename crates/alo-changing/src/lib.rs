//! A person's change to the grants reaches the file the daemon re-reads.
//!
//! The daemon's half of *a change reaches the running daemon* exists and is
//! tested: `alo-agentd`'s `rereading.rs` answers a knock
//! (`alo_protocol::FromAPerson::Granted`, which deliberately carries nothing)
//! by re-reading `/var/lib/alo/grants.toml` whole, so a grant missing from the
//! file is a grant revoked. Until this crate the **person's half had no
//! owner**: a grant made through `alo-picking` or a revocation made through
//! `alo-granted` changed a `Grants` in one process's memory and reached no
//! daemon and no next sign-in, unless a surface hand-wrote the composition —
//! write the file, then knock — which is order-sensitive glue of exactly the
//! kind this repository turns into tested values. A knock sent before the
//! write re-reads the old list, and nothing anywhere would say so.
//!
//! # The order is held by shape
//!
//! [`Changing`] is that composition as one value. Both of its doors —
//! [`Changing::granted`] and [`Changing::revoked`] — end in the same private
//! function, and that function is the only caller of [`Knocking::knock`] in
//! this crate: the knock happens after `alo_remembering::kept` has answered,
//! or it does not happen at all. There is no method that knocks without
//! writing, no method that writes without offering to knock, and no order for
//! a caller to get wrong.
//!
//! The change is applied to a **copy** of the list, the copy is written, and
//! only then does it replace the list the caller holds. So a write that fails
//! leaves everything as it was — the file (which `kept` replaces whole or not
//! at all), the list in memory, and the daemon, which is never knocked for a
//! change that did not land ([`NotChanged`]).
//!
//! # A machine with no daemon is not an error
//!
//! The knock is a courtesy to a daemon that happens to be running, not a
//! condition of the change. With nobody at the door the change already stands
//! on the disk, and the daemon that starts at the next sign-in reads it the
//! way it reads everything — which is the state this repository shipped with
//! before the knock existed. [`Stood`] says which of the three things
//! happened, because a person revoking something worrying is owed the
//! difference between *the running agent has already been told* and *it takes
//! effect at the next sign-in*.
//!
//! # What this crate cannot do, by construction
//!
//! **It cannot grant anything a person did not pick.** A grant arrives as an
//! `alo_picking::Chosen`, whose folder is sealed in that crate, so the only
//! grant this crate can carry to the disk is one a person's own pick made.
//!
//! **It cannot revoke by number.** A revocation arrives as an
//! `alo_granted::Seen` — a row derived from the machine's own list, with no
//! constructor from text — so a revocation carried here lands on a grant the
//! machine really holds, or on nothing at all.
//!
//! **It cannot be reached from an agent's door.** `alo-agentd` does not
//! depend on this crate, and this crate depends on no daemon, no turn and no
//! record; the daemon's own `rereading.rs` shows that nothing arriving on the
//! socket writes a byte of the grants file, and an agent sending the knock is
//! refused there in words. The manifest test in this crate is what keeps
//! tomorrow's build honest about it, the same way `alo-clipboard` keeps a
//! turn out of the clipboard.
//!
//! # Map
//!
//! | | |
//! |---|---|
//! | [`changing`] | The composition: applied to a copy, kept whole, then the knock |
//! | [`outcome`] | What a grant and a revocation each come to |
//! | [`stood`] | Where the change stands with the running daemon |
//! | [`knocking`] | The daemon's door as the person's side sees it |
//! | [`door`] | The real socket client, and its patience |
//! | [`refusing`] | The two ways a change does not happen, in words |
//! | [`words`] | Every string this crate can say, and the English beside each |

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

#[cfg(unix)]
pub mod changing;
#[cfg(unix)]
pub mod door;
pub mod knocking;
pub mod outcome;
pub mod refusing;
pub mod stood;
pub mod words;

#[cfg(unix)]
pub use changing::Changing;
#[cfg(unix)]
pub use door::TheDaemonsDoor;
pub use knocking::Knocking;
pub use outcome::{Gone, Made};
pub use refusing::NotChanged;
pub use stood::Stood;
pub use words::{Word, WordsError, changing_words, declare_into};
