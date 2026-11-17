//! **The file chooser as a decision**: a person standing in a folder, choosing
//! a file to open or a name to save under — the one chooser alo OS shows for
//! every application, decided by the owner on 2026-11-17 (*"Yes, build it"*).
//!
//! An application asks through the file chooser portal; the portal asks the
//! desktop through its door's `choose-file` act
//! (`docs/contracts/application-door.md`); the desktop shows the sheet and
//! answers with what the person chose. This crate is what the sheet **knows and
//! decides**: what is shown, what each key does, when a name may be saved, and
//! what the answer is. It draws nothing and opens nothing — the shell draws it
//! (`alo-shell`'s `the_file_chooser`), and what the asking application is then
//! given is the portal's.
//!
//! ```
//! use std::path::{Path, PathBuf};
//!
//! use alo_file_chooser::{Chooser, Ending, Folders, Key, Kind, Listing, NotListed};
//!
//! /// A home folder with one folder and one file in it.
//! struct AHome;
//! impl Folders for AHome {
//!     fn listed(&self, at: &Path) -> Result<Listing, NotListed> {
//!         match at.to_str() {
//!             Some("/home/anna") => Ok(Listing::of(vec!["Letters".into()], vec!["march.txt".into()])),
//!             Some("/home/anna/Letters") => Ok(Listing::of(vec![], vec!["to Rui.txt".into()])),
//!             _ => Err(NotListed::NotThere),
//!         }
//!     }
//!     fn holds(&self, at: &Path, name: &str) -> Option<Kind> {
//!         let listing = self.listed(at).ok()?;
//!         if listing.folders().iter().any(|one| one == name) {
//!             Some(Kind::Folder)
//!         } else {
//!             listing.files().iter().any(|one| one == name).then_some(Kind::File)
//!         }
//!     }
//! }
//!
//! // The person opens Letters and chooses the file in it.
//! let mut chooser = Chooser::to_open(Path::new("/home/anna"), &AHome);
//! assert_eq!(chooser.key(Key::Enter, &AHome), None);
//! assert_eq!(chooser.at(), Path::new("/home/anna/Letters"));
//! assert_eq!(
//!     chooser.key(Key::Enter, &AHome),
//!     Some(Ending::Chosen(PathBuf::from("/home/anna/Letters/to Rui.txt")))
//! );
//! ```
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`chooser`] | Where the person stands, the rows, the name, and every key |
//! | [`listing`] | What is inside a folder, as the chooser shows it |
//! | [`on_this_disk`] | The one implementation that reads a real disk |
//!
//! # Nothing here limits the person
//!
//! The person may go anywhere their own account can read. What alo may do on
//! this sheet — choose only inside the folders the person granted it — is
//! decided where alo's verbs are judged, and never here: a rule written here
//! would narrow the person's own use.
//!
//! # Nothing here says anything
//!
//! A [`Note`] is a sentence named, not written; the shell says it in the
//! person's language.
#![forbid(unsafe_code)]

pub mod chooser;
pub mod listing;
pub mod on_this_disk;

#[cfg(test)]
mod testing;

pub use chooser::{Chooser, Ending, How, Key, Note, Row, Where};
pub use listing::{Folders, Kind, Listing, MOST_SHOWN, NotListed};
pub use on_this_disk::OnThisDisk;
