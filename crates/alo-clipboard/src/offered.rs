//! What somebody who wants to paste holds: the forms available, and *which*
//! offer they are the forms of.
//!
//! A paster asks the clipboard what is there, is handed one of these, and comes
//! back with it when the person chooses a form. Between those two moments the
//! selection can change — somebody copies something else in another window, or
//! the application that copied it quits — and the whole of this type's job is
//! that the second call can tell.
//!
//! # The serial is the whole point
//!
//! It is not a name, a hash of the forms or a moment: it is a number this
//! machine's clipboard hands out, one per selection ever taken, and it never
//! repeats. Two identical offers made by two applications are two selections
//! with two serials, so a handle taken against the first cannot be spent
//! against the second.
//!
//! That matters more than it looks. A clipboard that matched a stale handle by
//! *what it looked like* would serve the new owner's data to somebody who asked
//! the old one — which reads to everybody involved exactly like a paste, and is
//! a leak between two applications.
//!
//! # There is nothing in it to spend
//!
//! A handle is a fact, not a permission: it carries what was offered so a
//! paster can show a person their choices, and holding one grants nothing. The
//! authority is the clipboard's own current offer, asked again at the moment of
//! the transfer, which is where [`crate::Clipboard::paste`] looks.

use crate::kind::Kind;
use crate::offer::{Offer, Taking};

/// What is on the clipboard right now, as somebody about to paste sees it.
///
/// Made by [`crate::Clipboard`] and by nothing else — there is no public field,
/// no constructor and no `From`, because a handle anybody could write would be
/// a handle naming a selection nobody took.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offered {
    /// Which selection this is: the number the clipboard gave it.
    serial: u64,
    /// Whether it was copied or cut.
    taking: Taking,
    /// Every form the owner said it can give, in the owner's own order.
    forms: Vec<Kind>,
}

impl Offered {
    /// The handle for a selection the clipboard has just taken.
    pub(crate) fn of(serial: u64, offer: &Offer) -> Self {
        Self {
            serial,
            taking: offer.taking(),
            forms: offer.forms().to_vec(),
        }
    }

    /// Which selection this is.
    pub(crate) fn serial(&self) -> u64 {
        self.serial
    }

    /// Whether what is on the clipboard was copied or cut.
    ///
    /// A paster reads it to word what it is about to do — *move* rather than
    /// *copy* — and never to decide whether it may.
    #[must_use]
    pub fn taking(&self) -> Taking {
        self.taking
    }

    /// Every form it can be pasted in, in the order the owner preferred them.
    #[must_use]
    pub fn forms(&self) -> &[Kind] {
        &self.forms
    }

    /// Whether one of the forms on offer is this one.
    #[must_use]
    pub fn offers(&self, form: &Kind) -> bool {
        self.forms.contains(form)
    }

    /// The form to paste in, given what the paster can accept, in the owner's
    /// order of preference.
    ///
    /// The owner's order and not the paster's, because the owner is the one who
    /// knows what is lost: an editor offering HTML before plain text is saying
    /// that plain text drops its formatting. A paster that wants the other
    /// order walks [`Offered::forms`] itself — nothing here is compulsory, and
    /// nothing here converts.
    #[must_use]
    pub fn best_of(&self, acceptable: &[Kind]) -> Option<&Kind> {
        self.forms.iter().find(|form| acceptable.contains(form))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::kind::Kind;

    /// The forms a paster sees are the forms the owner offered, in the owner's
    /// order.
    #[test]
    fn a_handle_carries_what_was_offered() {
        let html = Kind::named("text/html").unwrap();
        let offer = Offer::copied(vec![html.clone(), Kind::text()]).unwrap();
        let offered = Offered::of(7, &offer);

        assert_eq!(offered.forms(), [html, Kind::text()]);
        assert_eq!(offered.taking(), Taking::Copied);
        assert!(offered.offers(&Kind::text()));
        assert!(!offered.offers(&Kind::image_png()));
    }

    /// **Two identical offers are two selections.** What tells them apart is
    /// the serial rather than what they look like, which is what stops a handle
    /// taken against one being spent against the other.
    #[test]
    fn two_identical_offers_are_two_selections() {
        let offer = Offer::copied(vec![Kind::text()]).unwrap();
        assert_ne!(Offered::of(1, &offer), Offered::of(2, &offer));
        assert_eq!(Offered::of(1, &offer), Offered::of(1, &offer));
    }

    /// Choosing a form takes the owner's order, because the owner is the one
    /// who knows what a form loses.
    #[test]
    fn the_form_chosen_is_the_owners_preference() {
        let html = Kind::named("text/html").unwrap();
        let offer = Offer::copied(vec![html.clone(), Kind::text()]).unwrap();
        let offered = Offered::of(1, &offer);

        // The paster accepts both, in the other order. The owner's wins.
        assert_eq!(offered.best_of(&[Kind::text(), html.clone()]), Some(&html));
        // A paster that accepts only one gets that one.
        assert_eq!(offered.best_of(&[Kind::text()]), Some(&Kind::text()));
        // And one that accepts neither is told nothing fits, rather than being
        // handed something to convert.
        assert_eq!(offered.best_of(&[Kind::image_png()]), None);
        assert_eq!(offered.best_of(&[]), None);
    }
}
