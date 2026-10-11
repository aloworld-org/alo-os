//! Which graphics card this machine's login seat holds.
//!
//! **One answer, asked by both processes that need a screen.** The compositor
//! that draws the sign-in screen and the desktop a person signs in to both
//! have to open a card, and until this file existed only one of them knew how
//! to find the right one.
//!
//! # A card number is not a fact about a machine
//!
//! Both units used to carry `ALO_DISPLAY=/dev/dri/card0`, named rather than
//! guessed, on the reasoning that a machine says what it is. But firmware that
//! hands over a framebuffer gives `simpledrm` minor 0, so the real GPU lands
//! on minor 1 — **measured on a written disk on 2026-10-05**, where `logind`
//! had put `card1` on `seat0`, the compositor asked for `card0`, and the seat
//! answered *No such file or directory*. Most UEFI machines hand over a
//! framebuffer, so most machines fail that way.
//!
//! The compositor was fixed then and the desktop was not, because the desktop
//! had no unit at all yet. When it got one it was given the same constant with
//! a comment saying the constant is wrong on most machines — which is a
//! default chosen knowing it is usually wrong, and that is a bug rather than
//! a quirk. `docs/quirks.md` is for reality disagreeing with a specification.
//!
//! # Why `primary_gpu` is the right question
//!
//! It prefers the device whose PCI parent is marked `boot_vga`, which is the
//! firmware's own answer to *which screen is this machine's*. **That is what
//! excludes `simpledrm`**: a handed-over framebuffer is a platform device with
//! no PCI parent and so no such mark, and cannot win despite holding the lower
//! minor number.
//!
//! The seat is `seat0` because that is the seat a machine boots with and the
//! one `logind` puts a card on. A second seat is somebody plugging in a second
//! keyboard and screen, which is ADR 0088's question and not this one.

use std::path::PathBuf;

/// The card this machine's login seat holds.
///
/// # Errors
///
/// A sentence for whoever is standing the machine up, in English: a seat
/// holding no card at all, or a seat that could not be asked. Neither is
/// something the person in front of the screen can act on, and both are
/// written for a service log rather than a surface.
pub fn the_card_the_seat_holds() -> Result<PathBuf, String> {
    match smithay::backend::udev::primary_gpu("seat0") {
        Ok(Some(card)) => Ok(card),
        Ok(None) => Err(
            "this machine's login seat holds no graphics card, so there is no \
                         screen to draw on"
                .to_owned(),
        ),
        Err(why) => Err(format!(
            "this machine's login seat could not be asked what card it holds: {why}"
        )),
    }
}

/// The card to open: the one somebody named, or the one the seat holds.
///
/// **`ALO_DISPLAY` is still honoured where it is set**, because it is a public
/// configuration key and a person with two cards may have a reason. Nothing in
/// this repository's image sets it any more, and a machine that does not set
/// it is the ordinary case rather than the exception.
///
/// # Errors
///
/// [`the_card_the_seat_holds`]'s, when nothing named one.
pub fn the_card_to_open() -> Result<PathBuf, String> {
    the_card_named_or_the_seats(std::env::var("ALO_DISPLAY").ok())
}

/// The same decision, with the reading of the environment taken out.
///
/// **Split so it can be tested at all.** `unsafe_code = "forbid"` is this
/// workspace's constitution and `std::env::set_var` is unsafe in this edition,
/// so a test cannot put a variable in place to see what happens — and a
/// decision that cannot be tested is the half most worth testing. The reading
/// is one line above; this is the choosing, and it is the part that can be
/// wrong.
///
/// An empty value falls through to the seat rather than opening a path of no
/// name: `Environment=ALO_DISPLAY=` in a unit is somebody clearing the
/// variable, not naming a card.
///
/// # Errors
///
/// [`the_card_the_seat_holds`]'s, when nothing named one.
fn the_card_named_or_the_seats(named: Option<String>) -> Result<PathBuf, String> {
    match named {
        Some(named) if !named.trim().is_empty() => Ok(PathBuf::from(named)),
        _ => the_card_the_seat_holds(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A named card is taken as named**, because the variable is a public
    /// key and somebody who sets it has said something about their machine.
    ///
    /// The seat is not asked, which this test also shows: it passes on a host
    /// with no seat at all, where asking would answer an error.
    #[test]
    fn a_card_somebody_named_is_the_card() {
        let chosen = the_card_named_or_the_seats(Some("/dev/dri/card7".to_owned()));

        assert_eq!(chosen, Ok(PathBuf::from("/dev/dri/card7")));
    }

    /// **An empty value is nobody naming a card**, and falls through to the
    /// seat rather than opening a path with no name.
    ///
    /// `Environment=ALO_DISPLAY=` in a unit is somebody clearing the variable.
    /// Taken literally it would be an `open` on `""`, and the refusal a person
    /// standing the machine up reads would name no file at all.
    #[test]
    fn an_empty_value_names_no_card() {
        for said in [Some(String::new()), Some("   ".to_owned()), None] {
            let chosen = the_card_named_or_the_seats(said.clone());

            assert_ne!(
                chosen,
                Ok(PathBuf::new()),
                "{said:?} was taken as a card called nothing"
            );
        }
    }

    /// **And the refusals say what to do about them**, which is the half a
    /// service log is read for. Neither sentence names a card number, because
    /// a number is what was wrong in the first place.
    #[test]
    fn neither_refusal_names_a_card_number() {
        for said in [
            "this machine's login seat holds no graphics card, so there is no screen to draw on",
            "this machine's login seat could not be asked what card it holds: x",
        ] {
            assert!(
                !said.contains("card0") && !said.contains("card1"),
                "a refusal names a card number: {said}"
            );
            assert!(
                said.contains("seat"),
                "a refusal does not say where it looked: {said}"
            );
        }
    }
}
