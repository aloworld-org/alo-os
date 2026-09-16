//! Handing a picture to the clipboard.
//!
//! `alo-clipboard` holds one selection and the way back to whoever owns it, and
//! **never holds anybody's data**: the bytes do not exist there until the owner
//! is asked. A picture of the screen has no application behind it, so this file
//! is the owner — the smallest possible one, holding the picture it was given
//! and handing it over when somebody pastes.
//!
//! # Why a picture is offered in one form and never converted
//!
//! The offer is [`alo_clipboard::Kind::image_png`] and nothing else, because
//! that is what the rented mechanism handed over ([`crate::Picture::form`]). A
//! document that cannot take an image is told so in words by the clipboard
//! itself, which is a better answer than being handed something converted:
//! `alo-clipboard`'s own rule is that nothing anywhere converts, and an owner
//! that quietly re-encoded would be the first thing to break it.
//!
//! # Taking the selection retires whatever had it
//!
//! `alo_clipboard::Clipboard::taken` drops the previous owner, so a picture
//! copied here replaces whatever was on the clipboard exactly as any other copy
//! does. That is the ordinary meaning of copying and is not softened here: a
//! person who copies a picture of their screen expects their next paste to be
//! the picture.

use alo_clipboard::{Clipboard, CouldNotGive, Gives, Kind, Offer, Offered};

use crate::picture::Picture;
use crate::refusing::NotTaken;

/// A picture of the screen, holding itself for whoever pastes it.
///
/// Not public: nothing outside this crate has any reason to own a clipboard
/// selection on a picture's behalf, and a second owner of the same picture
/// would be a second answer to *what is pasted*.
#[derive(Debug)]
struct APictureOfTheScreen {
    /// What was taken.
    picture: Picture,
}

impl Gives for APictureOfTheScreen {
    fn give(&mut self, form: &Kind) -> Result<Vec<u8>, CouldNotGive> {
        if form != &self.picture.form() {
            // `Clipboard::paste` refuses a form that was not offered before it
            // ever reaches here, so this is unreachable through the clipboard.
            // It is an answer rather than a panic because a `give` that could
            // take the shell down would make every paste a risk, and because
            // converting — the other tempting answer — is the one thing
            // `alo-clipboard` says an owner must never do.
            return Err(CouldNotGive::TheApplicationDidNot);
        }
        Ok(self.picture.bytes().to_vec())
    }
}

/// Put this picture on the clipboard, ready to paste.
///
/// # Errors
/// [`NotTaken::NotCopied`] if the one-form offer this file makes is somehow not
/// an offer. It cannot be: the test below is what says so. It is a refusal
/// rather than an unwrap because a library that panicked over its own list
/// would take the shell down with it.
pub(crate) fn copy(clipboard: &mut Clipboard, picture: Picture) -> Result<Offered, NotTaken> {
    let offer = Offer::copied(vec![picture.form()]).map_err(|why| NotTaken::NotCopied {
        said: format!("the picture could not be offered: {why}"),
    })?;
    Ok(clipboard.taken(offer, Box::new(APictureOfTheScreen { picture })))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::a_picture;

    /// **What is pasted is the picture that was taken**, byte for byte, with
    /// nothing in between converting it.
    #[test]
    fn what_is_pasted_is_the_picture_that_was_taken() {
        let picture = a_picture();
        let mut clipboard = Clipboard::nothing_copied_yet();
        let offered = copy(&mut clipboard, picture.clone()).unwrap();

        assert!(offered.offers(&Kind::image_png()));
        let pasted = clipboard.paste(&offered, &Kind::image_png()).unwrap();
        assert_eq!(pasted.bytes(), picture.bytes());
    }

    /// **A picture is offered as a picture and as nothing else.** Somewhere
    /// that takes only text is told so in words rather than handed an image's
    /// bytes to make the best of.
    #[test]
    fn a_picture_is_offered_in_one_form_and_never_converted() {
        let mut clipboard = Clipboard::nothing_copied_yet();
        let offered = copy(&mut clipboard, a_picture()).unwrap();
        assert_eq!(offered.forms(), [Kind::image_png()]);
        assert!(clipboard.paste(&offered, &Kind::text()).is_err());
    }

    /// **The offer this file makes is an offer**, which is what makes
    /// [`NotTaken::NotCopied`] unreachable on a machine that shipped — and why
    /// it is written as a refusal rather than left to an unwrap.
    #[test]
    fn the_offer_this_file_makes_is_one() {
        assert!(Offer::copied(vec![a_picture().form()]).is_ok());
    }

    /// **The owner hands over the form it offered, and nothing else.** Reached
    /// directly, because the clipboard refuses another form before an owner is
    /// ever asked — and an owner that answered anyway would be one that
    /// converts.
    #[test]
    fn the_owner_hands_over_what_it_offered_and_nothing_else() {
        let mut owner = APictureOfTheScreen {
            picture: a_picture(),
        };
        assert_eq!(owner.give(&Kind::image_png()), Ok(a_picture().into_bytes()));
        assert_eq!(
            owner.give(&Kind::text()),
            Err(CouldNotGive::TheApplicationDidNot)
        );
    }

    /// **Copying a picture retires whatever had the clipboard**, which is what
    /// copying means everywhere else on the machine.
    #[test]
    fn copying_a_picture_retires_whatever_had_the_clipboard() {
        let mut clipboard = Clipboard::nothing_copied_yet();
        let first = copy(&mut clipboard, a_picture()).unwrap();
        let second = copy(&mut clipboard, a_picture()).unwrap();
        assert!(!clipboard.still_offering(&first));
        assert!(clipboard.still_offering(&second));
    }
}
