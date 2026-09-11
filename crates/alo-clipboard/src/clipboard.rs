//! The broker: one selection at a time, and the way back to whoever owns it.
//!
//! Everything else in this crate is a value. This is the one thing that holds
//! state, and it holds exactly two: **which offer is current, and how to reach
//! the owner that made it.** No copy of anybody's data, no history, no
//! previous owner, no queue.
//!
//! # Taking the selection retires the previous owner, by ownership
//!
//! [`Clipboard::taken`] replaces what is held, which **drops the previous
//! owner's [`crate::Gives`]**. That is the whole of the acceptance's *taking the
//! selection retires the previous owner's offer at once, so a transfer against
//! a stale one moves nothing*: not a flag that is checked, not a rule somebody
//! remembers, but the way back to the previous owner ceasing to exist anywhere
//! in the program. There is nothing left to ask, so nothing can be served.
//!
//! A clipboard that matched a stale handle against the current offer *by what
//! it looks like* would hand the new owner's data to somebody who asked the old
//! one — which is a leak between two applications, and which reads to everybody
//! involved exactly like a paste. `offered.rs` is why the serial is a number
//! rather than a resemblance.
//!
//! # A cut moves once
//!
//! A completed transfer of a [`crate::Taking::Cut`] offer retires it, for the
//! same reason in the other direction: a move that happened twice is not a
//! move, and the second paste would duplicate files the owner has already given
//! up. An editor's cut is a copy here and is unaffected — `offer.rs` says why.
//!
//! # Nothing here writes anything down
//!
//! No record, no log, no history. `docs/features.md` puts *clipboard history,
//! on the machine and never synced anywhere* at v1, and until somebody builds
//! that deliberately the absence is a feature: what a person copies is the
//! password they moved out of a manager, and a clipboard that kept the last ten
//! things would be a file of them. Nothing an agent did happened here either —
//! copy and paste is a person moving their own text between their own windows,
//! no verb reaches it, and `alo-record` is for what an agent did.
//!
//! # And nothing here leaves the machine
//!
//! There is no network in this crate and no dependency that has one. A
//! clipboard shared between machines would be an egress on every copy; ADR 0003
//! is why that would be a deliberate pairing rather than a convenience, and
//! nothing in v0.01 promises it.

use std::fmt;

use crate::giving::Gives;
use crate::kind::Kind;
use crate::offer::{Offer, Taking};
use crate::offered::Offered;
use crate::pasted::Pasted;
use crate::refusing::NotPasted;

/// What one owner put up, and how to reach it.
struct Taken {
    /// Which selection this is.
    serial: u64,
    /// What the owner said it can give.
    offer: Offer,
    /// The way back to the owner. **Dropped when this is replaced**, which is
    /// what retires an offer rather than marking it retired.
    source: Box<dyn Gives>,
}

/// The machine's clipboard: one selection, its forms, and its owner.
///
/// ```
/// use alo_clipboard::{Clipboard, CouldNotGive, Gives, Kind, NotPasted, Offer};
///
/// /// An application that copied a line of text.
/// struct AnEditor;
/// impl Gives for AnEditor {
///     fn give(&mut self, _form: &Kind) -> Result<Vec<u8>, CouldNotGive> {
///         Ok(b"the second paragraph".to_vec())
///     }
/// }
///
/// let mut clipboard = Clipboard::nothing_copied_yet();
///
/// // Pasting before anything was copied says so, rather than saying nothing.
/// assert_eq!(clipboard.to_paste(), Err(NotPasted::NothingCopied));
///
/// // Somebody copies.
/// let offer = Offer::copied(vec![Kind::text()]).expect("one form is an offer");
/// clipboard.taken(offer, Box::new(AnEditor));
///
/// // Somebody else pastes, in a form the copier offered.
/// let offered = clipboard.to_paste().expect("something was copied");
/// let pasted = clipboard.paste(&offered, &Kind::text()).expect("a form that was offered");
/// assert_eq!(pasted.bytes(), b"the second paragraph");
///
/// // A form nobody offered is refused rather than converted.
/// assert_eq!(
///     clipboard.paste(&offered, &Kind::image_png()),
///     Err(NotPasted::NotThatForm { form: Kind::image_png() }),
/// );
/// ```
#[derive(Default)]
pub struct Clipboard {
    /// The number the next selection gets. Never reused, so no handle can ever
    /// name a selection other than the one it was taken against.
    next: u64,
    /// What is on the clipboard, and nothing when nothing is.
    taken: Option<Taken>,
}

impl fmt::Debug for Clipboard {
    /// What is held, without the owner — which has no `Debug` and is somebody
    /// else's object anyway — and **without anything that was copied**, which
    /// is not here to print.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut shown = formatter.debug_struct("Clipboard");
        match &self.taken {
            None => shown.field("taken", &"nothing").finish(),
            Some(taken) => shown
                .field("serial", &taken.serial)
                .field("offer", &taken.offer)
                .finish_non_exhaustive(),
        }
    }
}

impl Clipboard {
    /// Where every machine starts: nothing has been copied.
    #[must_use]
    pub fn nothing_copied_yet() -> Self {
        Self {
            next: 0,
            taken: None,
        }
    }

    /// An owner takes the selection: this is what it can give, and this is how
    /// to reach it.
    ///
    /// **The previous owner is retired here**, by its way back being dropped.
    /// Anything holding the previous handle is refused from this moment, and
    /// nothing it asks for reaches anybody.
    ///
    /// Answers the handle for the new selection, which is what the owner's own
    /// side holds if it wants to know later whether it still owns the
    /// clipboard.
    pub fn taken(&mut self, offer: Offer, source: Box<dyn Gives>) -> Offered {
        self.next = self.next.saturating_add(1);
        let offered = Offered::of(self.next, &offer);
        self.taken = Some(Taken {
            serial: self.next,
            offer,
            source,
        });
        offered
    }

    /// The owner gives the selection up — it is closing, or the person undid
    /// the copy.
    ///
    /// Answers whether there was anything to give up. Takes no handle, because
    /// the only caller is the thing that owns the current selection and a
    /// compositor calling it holds the client that made the offer.
    pub fn given_up(&mut self) -> bool {
        self.taken.take().is_some()
    }

    /// What is on the clipboard, as somebody about to paste sees it.
    ///
    /// [`None`] when nothing has been copied. A caller that is about to paste
    /// wants [`Clipboard::to_paste`] instead, which says why in words.
    #[must_use]
    pub fn offered(&self) -> Option<Offered> {
        self.taken
            .as_ref()
            .map(|taken| Offered::of(taken.serial, &taken.offer))
    }

    /// What is on the clipboard, for somebody who is pasting **now**.
    ///
    /// # Errors
    /// [`NotPasted::NothingCopied`] — *there is nothing to paste: nothing has
    /// been copied yet* — which is the acceptance's *pasting when nothing has
    /// been copied is nothing to copy from rather than the thing before*. It is
    /// the only place that refusal is produced, because it is the only moment
    /// at which it is true: a caller that holds a handle holds proof that
    /// something was copied.
    pub fn to_paste(&self) -> Result<Offered, NotPasted> {
        self.offered().ok_or(NotPasted::NothingCopied)
    }

    /// Whether this handle still names the selection that is on the clipboard.
    ///
    /// For a paster deciding what to show before anybody asks for anything.
    /// Nothing is decided by it: [`Clipboard::paste`] asks the same question
    /// again at the moment of the transfer, because a selection can change
    /// between the two.
    #[must_use]
    pub fn still_offering(&self, offered: &Offered) -> bool {
        self.taken
            .as_ref()
            .is_some_and(|taken| taken.serial == offered.serial())
    }

    /// Paste what is on the clipboard, in this form.
    ///
    /// The owner is asked **only** for a form it offered, and only while the
    /// handle still names the selection it is holding. A cut that completes is
    /// retired here, so the same move cannot happen twice.
    ///
    /// # Errors
    /// [`NotPasted`], one of five, each with a sentence a person reads:
    /// something else has been copied since, the owner has given it up, that
    /// form was never offered, or the owner did not hand it over.
    pub fn paste(&mut self, offered: &Offered, form: &Kind) -> Result<Pasted, NotPasted> {
        let taken = self.taken.as_mut().ok_or(NotPasted::NoLongerOffered)?;
        if taken.serial != offered.serial() {
            return Err(NotPasted::CopiedOver);
        }
        if !taken.offer.offers(form) {
            // The owner is not asked at all. A compositor that asked and then
            // converted would be handing somebody bytes the application which
            // copied them never said it could produce.
            return Err(NotPasted::NotThatForm { form: form.clone() });
        }
        let bytes = taken.source.give(form).map_err(NotPasted::CouldNotGive)?;
        let moving = taken.offer.taking() == Taking::Cut;
        if moving {
            // A move that happened twice is not a move. The owner has given
            // this up, so there is nothing left for it to give.
            self.taken = None;
        }
        Ok(Pasted::of(form.clone(), bytes, moving))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::giving::CouldNotGive;
    use crate::testing::{AnOwner, Asked, an_owner, nothing_to_give};

    /// **A fresh machine has nothing to paste, and says so.** Not an empty
    /// value, not a silence: the sentence that sends a person to copy
    /// something.
    #[test]
    fn a_fresh_machine_has_nothing_to_paste() {
        let clipboard = Clipboard::nothing_copied_yet();
        assert_eq!(clipboard.offered(), None);
        assert_eq!(clipboard.to_paste(), Err(NotPasted::NothingCopied));
    }

    /// The default is the same machine, so a `Clipboard` made either way is one
    /// thing.
    #[test]
    fn the_default_is_a_machine_where_nothing_has_been_copied() {
        assert_eq!(
            Clipboard::default().to_paste(),
            Err(NotPasted::NothingCopied)
        );
    }

    /// **What is pasted is what was copied**, in the form that was asked for,
    /// and the owner was asked exactly once for exactly that form.
    #[test]
    fn what_is_pasted_is_what_was_copied() {
        let mut clipboard = Clipboard::nothing_copied_yet();
        let asked = Asked::nothing_yet();
        clipboard.taken(
            Offer::copied(vec![Kind::text(), Kind::named("text/html").unwrap()]).unwrap(),
            an_owner(&asked),
        );

        let offered = clipboard.to_paste().unwrap();
        let pasted = clipboard.paste(&offered, &Kind::text()).unwrap();
        assert_eq!(pasted.bytes(), AnOwner::gives(&Kind::text()).as_slice());
        assert_eq!(pasted.form(), &Kind::text());
        assert!(!pasted.the_owner_gave_it_up());
        assert_eq!(asked.forms(), [Kind::text()]);
    }

    /// **A copy stays**, so a person pasting the same thing into three windows
    /// pastes it three times.
    #[test]
    fn a_copy_can_be_pasted_again() {
        let mut clipboard = Clipboard::nothing_copied_yet();
        let asked = Asked::nothing_yet();
        clipboard.taken(Offer::copied(vec![Kind::text()]).unwrap(), an_owner(&asked));

        let offered = clipboard.to_paste().unwrap();
        for _ in 0..3 {
            assert!(clipboard.paste(&offered, &Kind::text()).is_ok());
        }
        assert_eq!(asked.how_many(), 3);
    }

    /// **A form nobody offered is refused, and the owner is never asked.** The
    /// second half is what makes it a refusal rather than a conversion: nothing
    /// reached the application that copied this, so nothing could have been
    /// turned into something it never said it could give.
    #[test]
    fn a_form_that_was_never_offered_is_refused_without_asking_anybody() {
        let mut clipboard = Clipboard::nothing_copied_yet();
        let asked = Asked::nothing_yet();
        clipboard.taken(Offer::copied(vec![Kind::text()]).unwrap(), an_owner(&asked));

        let offered = clipboard.to_paste().unwrap();
        assert_eq!(
            clipboard.paste(&offered, &Kind::image_png()),
            Err(NotPasted::NotThatForm {
                form: Kind::image_png()
            })
        );
        assert_eq!(asked.how_many(), 0);
    }

    /// **Taking the selection retires the previous owner at once.** The stale
    /// handle is refused, the previous owner is never asked again, and what the
    /// new owner has is what a paste now produces.
    #[test]
    fn taking_the_selection_retires_the_previous_owner() {
        let mut clipboard = Clipboard::nothing_copied_yet();
        let first = Asked::nothing_yet();
        clipboard.taken(Offer::copied(vec![Kind::text()]).unwrap(), an_owner(&first));
        let stale = clipboard.to_paste().unwrap();

        let second = Asked::nothing_yet();
        clipboard.taken(
            Offer::copied(vec![Kind::text()]).unwrap(),
            an_owner(&second),
        );

        assert!(!clipboard.still_offering(&stale));
        assert_eq!(
            clipboard.paste(&stale, &Kind::text()),
            Err(NotPasted::CopiedOver)
        );
        assert_eq!(
            first.how_many(),
            0,
            "the previous owner was asked for something"
        );

        // And the current owner answers a handle taken against it.
        let now = clipboard.to_paste().unwrap();
        assert!(clipboard.paste(&now, &Kind::text()).is_ok());
        assert_eq!(second.how_many(), 1);
    }

    /// **Two identical offers are two selections.** A handle taken against the
    /// first is refused against the second even though they look the same,
    /// which is what stops one application's data being served to somebody who
    /// asked another for it.
    #[test]
    fn a_handle_cannot_be_spent_against_an_identical_later_offer() {
        let mut clipboard = Clipboard::nothing_copied_yet();
        let first = Asked::nothing_yet();
        clipboard.taken(Offer::copied(vec![Kind::text()]).unwrap(), an_owner(&first));
        let stale = clipboard.to_paste().unwrap();
        let second = Asked::nothing_yet();
        let now = clipboard.taken(
            Offer::copied(vec![Kind::text()]).unwrap(),
            an_owner(&second),
        );

        assert_eq!(stale.forms(), now.forms());
        assert_ne!(stale, now);
        assert_eq!(
            clipboard.paste(&stale, &Kind::text()),
            Err(NotPasted::CopiedOver)
        );
        assert_eq!(first.how_many(), 0);
        assert_eq!(second.how_many(), 0);
    }

    /// **An owner that gives the selection up is gone, and says so in its own
    /// words** — which is not the sentence for a machine where nothing was ever
    /// copied, because the two send a person to two different places.
    #[test]
    fn an_owner_that_gives_up_the_selection_is_gone() {
        let mut clipboard = Clipboard::nothing_copied_yet();
        let asked = Asked::nothing_yet();
        clipboard.taken(Offer::copied(vec![Kind::text()]).unwrap(), an_owner(&asked));
        let offered = clipboard.to_paste().unwrap();

        assert!(clipboard.given_up());
        assert!(!clipboard.given_up(), "there was nothing left to give up");

        assert_eq!(clipboard.offered(), None);
        assert_eq!(
            clipboard.paste(&offered, &Kind::text()),
            Err(NotPasted::NoLongerOffered)
        );
        assert_eq!(asked.how_many(), 0);

        // And somebody starting a paste from nothing is told the other thing.
        assert_eq!(clipboard.to_paste(), Err(NotPasted::NothingCopied));
    }

    /// **A cut moves once.** The transfer says the owner has given it up, and
    /// the offer is retired at that moment, so nothing can paste the same move
    /// a second time.
    #[test]
    fn a_cut_moves_once() {
        let mut clipboard = Clipboard::nothing_copied_yet();
        let asked = Asked::nothing_yet();
        clipboard.taken(Offer::cut(vec![Kind::files()]).unwrap(), an_owner(&asked));
        let offered = clipboard.to_paste().unwrap();

        let pasted = clipboard.paste(&offered, &Kind::files()).unwrap();
        assert!(pasted.the_owner_gave_it_up());
        assert_eq!(pasted.bytes(), AnOwner::gives(&Kind::files()).as_slice());

        assert_eq!(
            clipboard.paste(&offered, &Kind::files()),
            Err(NotPasted::NoLongerOffered)
        );
        assert_eq!(asked.how_many(), 1);
        assert_eq!(clipboard.to_paste(), Err(NotPasted::NothingCopied));
    }

    /// **A cut that was refused is not a cut that happened.** The offer stands,
    /// because nothing moved — a clipboard that retired the selection on a
    /// failed transfer would lose somebody's files between two windows.
    #[test]
    fn a_cut_that_did_not_happen_does_not_retire_the_offer() {
        let mut clipboard = Clipboard::nothing_copied_yet();
        let asked = Asked::nothing_yet();
        clipboard.taken(Offer::cut(vec![Kind::files()]).unwrap(), an_owner(&asked));
        let offered = clipboard.to_paste().unwrap();

        // A form nobody offered, and then an owner that could not produce one.
        assert!(clipboard.paste(&offered, &Kind::text()).is_err());
        assert!(clipboard.still_offering(&offered));

        let mut clipboard = Clipboard::nothing_copied_yet();
        clipboard.taken(Offer::cut(vec![Kind::files()]).unwrap(), nothing_to_give());
        let offered = clipboard.to_paste().unwrap();
        assert_eq!(
            clipboard.paste(&offered, &Kind::files()),
            Err(NotPasted::CouldNotGive(CouldNotGive::TheApplicationDidNot))
        );
        assert!(clipboard.still_offering(&offered));
        assert_eq!(asked.how_many(), 0);
    }

    /// **An owner that cannot hand it over is said so**, rather than producing
    /// an empty paste — which a person would read as the thing they copied
    /// having been empty.
    #[test]
    fn an_owner_that_cannot_hand_it_over_is_a_refusal_rather_than_nothing() {
        let mut clipboard = Clipboard::nothing_copied_yet();
        clipboard.taken(
            Offer::copied(vec![Kind::text()]).unwrap(),
            nothing_to_give(),
        );
        let offered = clipboard.to_paste().unwrap();

        assert_eq!(
            clipboard.paste(&offered, &Kind::text()),
            Err(NotPasted::CouldNotGive(CouldNotGive::TheApplicationDidNot))
        );
        // And it is still there to try again, because nothing about the
        // selection changed.
        assert!(clipboard.still_offering(&offered));
    }

    /// **What is held is the offer and the way back, and nothing else** —
    /// measured where it would show first, which is what the clipboard says
    /// about itself when somebody prints it in a log.
    #[test]
    fn what_is_held_is_an_offer_and_never_what_was_copied() {
        let mut clipboard = Clipboard::nothing_copied_yet();
        clipboard.taken(
            Offer::copied(vec![Kind::text()]).unwrap(),
            an_owner_saying(b"hunter2"),
        );

        let shown = format!("{clipboard:?}");
        assert!(!shown.contains("hunter2"), "{shown}");
        assert!(shown.contains("text/plain"), "{shown}");
    }

    /// An owner with something worth not printing.
    fn an_owner_saying(bytes: &'static [u8]) -> Box<dyn Gives> {
        struct Saying(&'static [u8]);
        impl Gives for Saying {
            fn give(&mut self, _form: &Kind) -> Result<Vec<u8>, CouldNotGive> {
                Ok(self.0.to_vec())
            }
        }
        Box::new(Saying(bytes))
    }
}
