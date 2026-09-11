//! Copy, cut and paste across applications — the selection, decided before
//! there is anything to draw.
//!
//! `docs/features.md` promises at v0.01: **copy, cut and paste — text, images
//! and files, across applications.** It is the promise that had no crate, no
//! test and no line anywhere, and it spent four readings of
//! `docs/autonomy/v0-01-evidence.md` sorted into *work that needs a machine*
//! beside *the GPU works on first boot*. That sorting was wrong, and this crate
//! is why: **a clipboard is a protocol before it is a surface.**
//!
//! One application owns the selection and says which forms it can give another;
//! somebody asks for one of those forms; the broker holds the offer and the way
//! back to the owner, and holds nothing else. Every value in that sentence is
//! decidable with no pixels — exactly as `alo-overlay`, `alo-approving`,
//! `alo-indicator` and `alo-recounting` decided their surfaces without drawing
//! one.
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`kind`] | One form something can be given in, and the promise's three |
//! | [`offer`] | What an owner put up: the forms, and copied or cut |
//! | [`offered`] | What somebody about to paste holds, and *which* selection it is |
//! | [`giving`] | The owner's half: the only place bytes ever come from |
//! | [`clipboard`] | The broker: one selection, and the way back to its owner |
//! | [`pasted`] | A completed transfer |
//! | [`refusing`] | The five ways a paste does not happen, each with a sentence |
//! | [`words`] | Every string this crate can say, and the English beside each |
//!
//! ```
//! use alo_clipboard::{Clipboard, CouldNotGive, Gives, Kind, NotPasted, Offer};
//!
//! /// A drawing program, which can hand over what it has as a PNG.
//! struct ADrawing;
//! impl Gives for ADrawing {
//!     fn give(&mut self, _form: &Kind) -> Result<Vec<u8>, CouldNotGive> {
//!         Ok(b"\x89PNG\r\n\x1a\n".to_vec())
//!     }
//! }
//!
//! let mut clipboard = Clipboard::nothing_copied_yet();
//! clipboard.taken(
//!     Offer::copied(vec![Kind::image_png()]).expect("one form is an offer"),
//!     Box::new(ADrawing),
//! );
//!
//! // A document that accepts images pastes the image.
//! let offered = clipboard.to_paste().expect("something was copied");
//! let pasted = clipboard.paste(&offered, &Kind::image_png()).expect("a form on offer");
//! assert!(pasted.bytes().starts_with(b"\x89PNG"));
//!
//! // A text field is told why it cannot, in a sentence, rather than being
//! // handed an image's bytes to make the best of.
//! assert_eq!(
//!     clipboard.paste(&offered, &Kind::text()),
//!     Err(NotPasted::NotThatForm { form: Kind::text() }),
//! );
//! ```
//!
//! # The three promises, and what each is in the code
//!
//! - **What is pasted is what was copied, in a form the copier offered.** There
//!   is nowhere in [`Clipboard`] to hold anybody's data: the bytes do not exist
//!   until an owner is asked through [`Gives`], and a form the owner did not
//!   offer is refused before that call is made. Nothing anywhere converts.
//! - **Taking the selection retires the previous owner at once.**
//!   [`Clipboard::taken`] drops the previous owner's [`Gives`], so the way back
//!   to it stops existing. A transfer against a stale handle does not move
//!   nothing because a rule says so; it moves nothing because there is nobody
//!   left to ask.
//! - **Pasting when nothing has been copied says so.** [`Clipboard::to_paste`]
//!   answers [`NotPasted::NothingCopied`] — *there is nothing to paste: nothing
//!   has been copied yet* — which is a different sentence from the one for a
//!   selection whose owner has gone, because the two send a person to two
//!   different places.
//!
//! # Text, images and files are three forms and not three mechanisms
//!
//! The promise names three things and there is exactly one path through this
//! crate for all of them: a [`Kind`] is a media type, an [`Offer`] is a list of
//! them, and nothing anywhere switches on which of the three is being moved.
//! `kind.rs` says why that is the load-bearing decision rather than a tidy one —
//! three paths would be three sets of refusals to keep honest and a fourth
//! thing, a font or a spreadsheet range, arriving as a fourth path nobody wrote.
//!
//! # This is not `alo-context`'s selection, and that is a guarantee
//!
//! ADR 0001 §4 offers an agent the focused window, the highlighted text and the
//! open document **at the moment of invocation and for that turn**. What a
//! person has copied is none of the three. A turn that could read it would be
//! the background reader `CLAUDE.md` calls a bug in this product — and worse
//! than the general case, because a clipboard is where a password manager puts
//! a password.
//!
//! So the two crates do not know about each other: neither names the other in
//! its dependencies, and `tests/the_clipboard_is_not_a_turns_context.rs` reads
//! both manifests and a whole turn to say so. **No agent verb reaches here
//! either.** The ten verbs in `docs/contracts/agent-verbs.md` are six about
//! files and four about applications; copy and paste is a person moving their
//! own text between their own windows, and law 2's enumerated verbs are not
//! where it belongs.
//!
//! # What this crate deliberately does not do
//!
//! **It draws nothing.** No surface, no Wayland, no `smithay`. The compositor
//! half — `wl_data_device` taken, offered and read through a pipe — belongs in
//! `crates/alo-shell`, which is the desktop lane's; what is settled here is
//! every value and every refusal that lane would otherwise decide while wiring
//! it, which is how a refusal gets decided by accident.
//!
//! **It remembers nothing.** `docs/features.md` puts *clipboard history, on the
//! machine and never synced anywhere* at v1. Until somebody builds that
//! deliberately, holding the last ten things a person copied would be a file of
//! their passwords.
//!
//! **It sends nothing anywhere.** There is no network here and no dependency
//! that has one, so a working day's copying produces no egress at all.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod clipboard;
pub mod giving;
pub mod kind;
pub mod offer;
pub mod offered;
pub mod pasted;
pub mod refusing;
pub mod words;

#[cfg(test)]
mod testing;

pub use clipboard::Clipboard;
pub use giving::{CouldNotGive, Gives};
pub use kind::{Form, Kind, KindError, THE_LONGEST};
pub use offer::{Offer, OfferError, THE_MOST_FORMS, Taking};
pub use offered::Offered;
pub use pasted::Pasted;
pub use refusing::NotPasted;
pub use words::{EVERY_WORD, Word, WordsError, clipboard_words, declare_into};
