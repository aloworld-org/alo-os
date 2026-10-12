//! Which graphics card this machine's login seat holds.
//!
//! # Why this is a file rather than a line in each binary
//!
//! Two binaries need it and they are the same machine's two halves: the greeter
//! stands in front of somebody who has not signed in, and the desktop is what
//! they get afterwards. **They must not be able to disagree about which screen
//! that is**, and until 2026-10-11 they did.
//!
//! `alo-compositor` asks the seat. `alo-desktop` did not ask anything — it
//! required `ALO_DISPLAY` and refused to start without it — so
//! `image/usr/lib/systemd/system/alo-desktop.service` had to name a card, and
//! the card it names is `/dev/dri/card0` under a comment that begins **"this
//! line is wrong on most machines"**. That comment is honest and thorough and
//! it names the fix as belonging to `crates/alo-desktop`; this is that fix, put
//! where both binaries can read it instead of in one of them.
//!
//! # The fault it exists to prevent, which is not a guess
//!
//! Measured on 2026-10-05 against a written disk, by the lane that owns the
//! image: firmware that hands over a framebuffer gives `simpledrm` **minor 0**,
//! so the real GPU is **minor 1**, and `logind` had put `card1` on `seat0`
//! while the unit asked for `card0`. The answer was *No such file or
//! directory*. **Most UEFI machines hand over a framebuffer**, so most machines
//! fail that way — and a virtual machine with one `virtio-gpu`, which is where
//! this is developed, does not.
//!
//! So the fault is invisible exactly where it is worked on and present almost
//! everywhere else, which is the kind that reaches a release.
//!
//! # Why `primary_gpu` is the right question
//!
//! It prefers the device whose PCI parent is marked `boot_vga` — the firmware's
//! own answer to *which screen is this machine's*. **That is what excludes
//! `simpledrm`**: a handed-over framebuffer is a platform device with no PCI
//! parent and no such mark, so it cannot win even though it holds the lower
//! minor number. Picking the lowest minor, or the first entry in
//! `/dev/dri`, would pick the framebuffer on exactly the machines this is for.
//!
//! `seat0` because that is the seat a machine boots with and the one `logind`
//! puts a card on. A second seat is somebody plugging in a second keyboard and
//! screen, which is ADR 0088's question and not this one.

use std::path::PathBuf;

/// The card this machine's login seat holds, honouring an override.
///
/// `named` is what the unit file said, where it said anything.
/// [`which_card_the_seat_holds()`] is what callers use: it reads `ALO_DISPLAY`
/// and hands the answer here.
///
/// Private, because one caller and this module's tests are all of it. A
/// function exported only so a test can reach it is a public surface nobody
/// outside asked for.
///
/// **The override comes first**, because a machine with two cards is a machine
/// whose owner may mean the other one, and a unit file is where they say so. A
/// default that cannot be overridden is a wall, and a machine that ignores what
/// its owner wrote is worse than one that cannot find a card.
///
/// **Empty is not a name.** `ALO_DISPLAY=` with nothing after it is a unit file
/// that meant to say something and did not; taken as a path it becomes *No such
/// file or directory* about a filename of nothing, which is the least useful
/// thing a machine can say about this. It asks the seat instead, which is what
/// whoever wrote that line was reaching for.
///
/// # Errors
/// A sentence for a seat that holds no card, and one for a seat that could not
/// be asked. Both name the seat, because *there is no card* and *udev would not
/// answer* are different machines to be standing in front of.
fn which_card(named: Option<&str>) -> Result<PathBuf, String> {
    if let Some(named) = named
        && !named.trim().is_empty()
    {
        return Ok(PathBuf::from(named));
    }
    match smithay::backend::udev::primary_gpu("seat0") {
        Ok(Some(card)) => Ok(card),
        Ok(None) => Err(
            "this machine's login seat, seat0, holds no graphics card, so there is \
                         no screen to draw on"
                .to_owned(),
        ),
        Err(why) => Err(format!(
            "this machine's login seat, seat0, could not be asked what card it holds: {why}"
        )),
    }
}

/// The same, reading the override from `ALO_DISPLAY`.
///
/// **Separate from `which_card` above so the decision can be tested.** This crate
/// sets `unsafe_code = "forbid"`, and `std::env::set_var` is `unsafe` — so no
/// test anywhere in this workspace can put a variable in front of a function
/// that reads one. Splitting the environment off the decision is what makes
/// *empty is not a name* and *a named card is used as given* checkable at all,
/// and both are rules somebody could plausibly get wrong later.
///
/// What stays unmeasured is the udev call, and it is `smithay`'s: `primary_gpu`
/// preferring the `boot_vga` device over `simpledrm` is verified by booting a
/// machine, which `docs/booting.md` says a virtual machine cannot show.
///
/// # Errors
/// Whatever `which_card` refuses: a seat holding no card, or one that could
/// not be asked.
pub fn which_card_the_seat_holds() -> Result<PathBuf, String> {
    which_card(std::env::var("ALO_DISPLAY").ok().as_deref())
}

#[cfg(test)]
#[path = "which_card_the_seat_holds_tests.rs"]
mod tests;
