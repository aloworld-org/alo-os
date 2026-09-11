//! What an owner says it can give, and whether it is copying or cutting.
//!
//! An offer is the whole of what a copier puts on the clipboard: **the forms it
//! can produce, and nothing else.** Not the text, not the image, not the file —
//! those stay in the application that owns them until somebody actually asks,
//! which is how every desktop clipboard works and is why copying a hundred
//! megabytes of image costs nothing until it is pasted.
//!
//! So the broker holds an offer and a way back to the owner, and never a copy
//! of anybody's data. [`crate::Clipboard`] is where that is enforced by there
//! being nowhere to put it.
//!
//! # Copying and cutting are one offer with a flag, not two clipboards
//!
//! [`Taking::Cut`] says *the owner still holds this, and will remove it when
//! the transfer completes* — the deferred cut a file manager makes, where the
//! files stay where they are until somebody pastes. [`crate::Clipboard`] retires
//! a cut offer as soon as one transfer completes, because a cut pasted twice
//! would be a move that duplicated.
//!
//! **An editor's cut is a copy here**, deliberately: a text editor deletes the
//! selection at the moment the person presses the key and puts a plain copy on
//! the clipboard, so by the time the clipboard is involved there is nothing left
//! anywhere to move. Saying otherwise would give this crate a rule about text
//! that text does not obey, and would make cut text unpastable a second time —
//! which every person who has ever pasted the same line into two files would
//! experience as the machine losing their work.
//!
//! # An offer arrives from a client, so it is bounded
//!
//! A client may offer no form at all (that is not a copy, and is refused), the
//! same form twice (which would make *which forms are available* an ambiguous
//! question, and is refused by name), or more forms than any real application
//! has (bounded by [`THE_MOST_FORMS`]). None of these is something a person
//! reads: what has gone wrong is in an application, and a compositor handed one
//! has one thing to do with it, which is not to take the selection.

use crate::kind::Kind;

/// The most forms one offer may carry.
///
/// A real client offers a handful: a text editor offers plain text, HTML and
/// maybe RTF; a drawing program offers three or four image types. Sixty-four is
/// far past any of them and is a bound rather than a budget — what it stops is a
/// client handing a compositor an offer whose *list* is the thing that costs the
/// memory.
pub const THE_MOST_FORMS: usize = 64;

/// Whether what was taken is being copied or moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Taking {
    /// The owner keeps what it copied. Pasting it again pastes it again.
    Copied,
    /// The owner still holds it and removes it when the transfer completes.
    /// One transfer, and then the offer is retired — a move that happened
    /// twice is not a move.
    Cut,
}

/// What one owner put on the clipboard: the forms it can give, and how.
///
/// ```
/// use alo_clipboard::{Kind, Offer, Taking};
///
/// let offer = Offer::copied(vec![Kind::text(), Kind::named("text/html").expect("a media type")])
///     .expect("two forms is an offer");
///
/// assert_eq!(offer.taking(), Taking::Copied);
/// assert!(offer.offers(&Kind::text()));
/// assert!(!offer.offers(&Kind::image_png()));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    /// Copying, or moving.
    taking: Taking,
    /// Every form the owner says it can give, in the order it offered them.
    forms: Vec<Kind>,
}

impl Offer {
    /// A copy: these forms, and the owner keeps what it has.
    ///
    /// # Errors
    /// [`OfferError`], for an offer that is not one.
    pub fn copied(forms: Vec<Kind>) -> Result<Self, OfferError> {
        Self::taken(Taking::Copied, forms)
    }

    /// A cut: these forms, and the owner removes its own when one transfer
    /// completes.
    ///
    /// # Errors
    /// [`OfferError`], for an offer that is not one.
    pub fn cut(forms: Vec<Kind>) -> Result<Self, OfferError> {
        Self::taken(Taking::Cut, forms)
    }

    /// The one constructor both of the above are.
    fn taken(taking: Taking, forms: Vec<Kind>) -> Result<Self, OfferError> {
        if forms.is_empty() {
            return Err(OfferError::NothingOffered);
        }
        if forms.len() > THE_MOST_FORMS {
            return Err(OfferError::TooManyForms {
                how_many: forms.len(),
            });
        }
        for (which, form) in forms.iter().enumerate() {
            if forms.iter().take(which).any(|earlier| earlier == form) {
                return Err(OfferError::TheSameFormTwice {
                    media: form.media().to_owned(),
                });
            }
        }
        Ok(Self { taking, forms })
    }

    /// Whether this was copied or cut.
    #[must_use]
    pub fn taking(&self) -> Taking {
        self.taking
    }

    /// Every form the owner said it can give, in the order it offered them.
    ///
    /// The order is the owner's preference — a text editor lists HTML before
    /// plain text when it would rather be pasted as HTML — and it is kept
    /// rather than sorted, because reordering it would be this crate making
    /// somebody else's choice on their behalf.
    #[must_use]
    pub fn forms(&self) -> &[Kind] {
        &self.forms
    }

    /// Whether the owner said it can give this form.
    ///
    /// The whole of what a transfer is allowed to ask for. There is no
    /// near-enough, no family match and no conversion: a form that is not in
    /// this list is refused in words, because converting would hand somebody
    /// bytes that the application which copied them never said it could produce.
    #[must_use]
    pub fn offers(&self, form: &Kind) -> bool {
        self.forms.contains(form)
    }
}

/// Why what a client put up is not an offer.
///
/// Not a refusal anybody reads in their own language, for [`crate::KindError`]'s
/// reason: what has gone wrong is in an application, and the person is not part
/// of this conversation. Nothing here is a partial repair — an offer with a
/// duplicate is refused whole rather than deduplicated, because a compositor
/// that quietly edited a client's offer would be answering *which forms are
/// available* with a list the client did not write.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OfferError {
    /// A client took the selection and named no form at all.
    #[error(
        "an offer with no form in it is not a copy: there would be nothing anybody could ask for"
    )]
    NothingOffered,

    /// One media type, twice.
    #[error(
        "{media} is offered twice, and a form offered twice makes `which forms are available` a question with two answers"
    )]
    TheSameFormTwice {
        /// Which one.
        media: String,
    },

    /// More forms than [`THE_MOST_FORMS`].
    #[error(
        "{how_many} forms is more than the {THE_MOST_FORMS} a real application offers, and a clipboard does not hold what a client says without a bound on it"
    )]
    TooManyForms {
        /// How many arrived.
        how_many: usize,
    },
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **Text, images and files go through one door.** The same constructor,
    /// the same question afterwards, and nothing anywhere switching on which of
    /// the three it is — which is the acceptance's *three offered types rather
    /// than three mechanisms*, in the one file where a second mechanism would
    /// have to appear.
    #[test]
    fn text_images_and_files_are_one_offer() {
        let offer = Offer::copied(vec![Kind::text(), Kind::image_png(), Kind::files()]).unwrap();
        for form in [Kind::text(), Kind::image_png(), Kind::files()] {
            assert!(offer.offers(&form), "{}", form.media());
        }
        assert_eq!(offer.forms().len(), 3);
    }

    /// An owner's order is the owner's, and this crate does not improve it.
    #[test]
    fn the_owners_order_is_kept() {
        let html = Kind::named("text/html").unwrap();
        let offer = Offer::copied(vec![html.clone(), Kind::text()]).unwrap();
        assert_eq!(offer.forms(), [html, Kind::text()]);
    }

    /// Copying and cutting are one shape with a flag, and the flag is read off
    /// the offer rather than guessed at from what was copied.
    #[test]
    fn copying_and_cutting_are_one_shape() {
        assert_eq!(
            Offer::copied(vec![Kind::files()]).unwrap().taking(),
            Taking::Copied
        );
        assert_eq!(
            Offer::cut(vec![Kind::files()]).unwrap().taking(),
            Taking::Cut
        );
    }

    /// **A form nobody offered is not offered**, whatever family it is in. This
    /// is the one question a transfer is allowed to ask, so it is the one that
    /// must have no near-enough in it.
    #[test]
    fn a_form_nobody_offered_is_not_offered() {
        let offer = Offer::copied(vec![Kind::named("text/html").unwrap()]).unwrap();
        assert!(!offer.offers(&Kind::text()));
        assert!(!offer.offers(&Kind::image_png()));
        assert!(!offer.offers(&Kind::named("text/html;charset=utf-8").unwrap()));
    }

    /// **An offer of nothing is not a copy.** A client that takes the selection
    /// and names no form has left a clipboard that refuses everything, which is
    /// indistinguishable to a person from one that is broken.
    #[test]
    fn an_offer_of_nothing_is_refused() {
        assert_eq!(Offer::copied(Vec::new()), Err(OfferError::NothingOffered));
        assert_eq!(Offer::cut(Vec::new()), Err(OfferError::NothingOffered));
    }

    /// **One form twice is refused whole**, rather than deduplicated: a
    /// compositor that edited a client's offer would be answering *which forms
    /// are available* with a list nobody wrote.
    #[test]
    fn the_same_form_twice_is_refused() {
        let refused = Offer::copied(vec![Kind::text(), Kind::image_png(), Kind::text()]);
        assert_eq!(
            refused,
            Err(OfferError::TheSameFormTwice {
                media: "text/plain;charset=utf-8".to_owned()
            })
        );
        // And case is not what makes two forms two, for `Kind`'s reason.
        assert!(matches!(
            Offer::copied(vec![Kind::named("IMAGE/PNG").unwrap(), Kind::image_png()]),
            Err(OfferError::TheSameFormTwice { .. })
        ));
    }

    /// **More forms than any real application offers is refused by name.** A
    /// bound on what a client can make a compositor hold, and the refusal says
    /// how many arrived rather than silently keeping the first sixty-four.
    #[test]
    fn more_forms_than_any_real_application_offers_is_refused() {
        let many: Vec<Kind> = (0..THE_MOST_FORMS)
            .map(|which| Kind::named(&format!("application/x-{which}")).unwrap())
            .collect();
        assert_eq!(
            Offer::copied(many.clone()).unwrap().forms().len(),
            THE_MOST_FORMS
        );

        let mut over = many;
        over.push(Kind::text());
        assert_eq!(
            Offer::copied(over),
            Err(OfferError::TooManyForms {
                how_many: THE_MOST_FORMS + 1
            })
        );
    }
}
