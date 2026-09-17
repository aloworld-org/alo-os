//! Dragging something out of one window and letting it go in another.
//!
//! `docs/features.md` promises at v0.5: **drag and drop between applications**,
//! and `docs/autonomy/v0-5-hands-on-the-desktop-plan.md` task 4 is what it has
//! to mean. Two sentences of that acceptance are the whole of this crate:
//!
//! - **A drop carries what copy and paste carries** — text, images, files —
//!   through `alo-clipboard`'s payload types rather than a second set, and a
//!   sandboxed window receives it the way a sandboxed window expects.
//! - **Dropping a file onto the agent's surface offers it for that question and
//!   is not a grant.** No path becomes reachable, nothing is written into
//!   anybody's grant list, and there is no `alo-capability` dependency here to
//!   write one with. ADR 0001 §3: a grant is made in `alo-picking`, and nowhere
//!   else.
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`target`] | What is under the pointer: a window, the agent's surface, or nothing |
//! | [`dragging`] | One drag, from picked up to let go — and it happens once |
//! | [`over`] | What letting go here would do, while it is still held |
//! | [`delivery`] | Bytes on the spot, or files exported through the documents portal |
//! | [`handed`] | A drop that happened: who received it, in which form, and how |
//! | [`for_one_turn`] | A drop on the agent's surface: read once, and never a grant |
//! | [`uri_list`] | Reading `text/uri-list`, which is how a desktop drags files |
//! | [`refusing`] | The six ways nothing arrives, each with a sentence |
//! | [`words`] | Every string this crate can say, and the English beside each |
//!
//! ```
//! use alo_clipboard::{CouldNotGive, Gives, Kind, Offer, Taking};
//! use alo_handing::{Application, Drag, LetGo, Over, Target};
//!
//! /// The window a person is dragging a paragraph out of.
//! struct AnEditor;
//! impl Gives for AnEditor {
//!     fn give(&mut self, _form: &Kind) -> Result<Vec<u8>, CouldNotGive> {
//!         Ok(b"the second paragraph".to_vec())
//!     }
//! }
//!
//! let drag = Drag::begun(Offer::copied(vec![Kind::text()])?, Box::new(AnEditor));
//! let notes = Target::AWindow(Application::sandboxed("org.alo.Notes", vec![Kind::text()]));
//!
//! // While it is still held, the pointer says what letting go would do, and
//! // the editor is not asked for a byte.
//! assert_eq!(
//!     drag.over(&notes),
//!     Over::WouldHandItOver { form: Kind::text(), taking: Taking::Copied },
//! );
//!
//! // Letting go hands it over, in a form both sides named.
//! // `NotHanded` has no `Display` — the only road to words is `said`, in the
//! // language the person reads — so a doctest names it the way a shell would
//! // not: by printing the value.
//! let let_go = drag.let_go_on(&notes).map_err(|refused| format!("{refused:?}"))?;
//! let LetGo::Handed(handed) = let_go else { unreachable!() };
//! assert_eq!(handed.to(), "org.alo.Notes");
//! assert_eq!(handed.delivery().bytes(), Some(b"the second paragraph".as_slice()));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Nothing here draws
//!
//! No pointer, no cursor, no surface and no protocol. What a person sees while
//! they drag is [`Over`], which is a sentence and a form; the compositor draws
//! it and the Wayland data device carries it. This crate decides.
//!
//! # Nothing here can be read back
//!
//! **No serde dependency at all**, which is `alo-clipboard`'s guarantee one step
//! further on: a drag that could be deserialised would be a drop that nobody's
//! hand made, and a file offered to the agent that no person ever dragged.
//!
//! # Nothing here reaches the record, and nothing here is egress
//!
//! A drag is a person moving their own things between their own windows. No verb
//! reaches it, `alo-record` is for what an agent did, and there is no network in
//! this crate or in anything it depends on.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod delivery;
pub mod dragging;
pub mod for_one_turn;
pub mod handed;
pub mod over;
pub mod refusing;
pub mod target;
pub mod uri_list;
pub mod words;

#[cfg(test)]
mod testing;

pub use delivery::Delivery;
pub use dragging::{Drag, LetGo};
pub use for_one_turn::ForOneTurn;
pub use handed::Handed;
pub use over::Over;
pub use refusing::NotHanded;
pub use target::{Application, Target, the_agents_surface};
pub use uri_list::{NotAFileList, THE_LONGEST_LIST, files_in};
pub use words::{Word, WordsError, declare_into, handing_words};
