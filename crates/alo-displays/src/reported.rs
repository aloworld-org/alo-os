//! What the machine reports about one screen that is plugged in now.
//!
//! Everything here is read off the compositor and never asked of it: the kernel
//! and the compositor library are rented (ADR 0011), and no line in this crate
//! sets a mode, drives a socket or talks to a driver. What this crate does with
//! these numbers is decide — which screen it is, where it goes and how large —
//! and hand the decision back.
//!
//! # Why the refusals here keep their English
//!
//! Every one of them means the compositor handed us something that is not a
//! screen: no pixels, no size, a name with nothing in it. There is nothing to
//! ask a person and nothing they could do about it, so — exactly as
//! `alo_desktops::NotADisplay` and `alo_dock::ScreenError` do, for the same
//! reason — [`NotAScreen`] is English with a `Display`, read by whoever is
//! fixing alo OS. The refusals a person reads are [`crate::IdentityError`] and
//! [`crate::NotArranged`], and they come out of a settings file rather than out
//! of the machine.

use alo_appearance::DisplayId;

use crate::identity::{Identity, Panel, Socket, whoever_is_attached};

/// Why something the machine reported is not a screen.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotAScreen {
    /// A width or a height of no pixels at all: what was reported, both
    /// numbers.
    #[error("a screen has a width and a height in pixels: {0} by {1} is not one")]
    NoPixels(u32, u32),
    /// A physical size with nothing in one direction: what was reported, both
    /// numbers. A screen that reports no size at all is not this — it is a
    /// screen alo OS cannot work a size out for, and it is allowed.
    #[error("a screen that gives its size gives both sides: {0} by {1} millimetres is not a size")]
    NoSize(u32, u32),
    /// The name the shell would know this screen by is not one `alo-appearance`
    /// accepts, so its background could never be asked for.
    #[error("{0} is not a name the shell can hold a background against")]
    NotANameTheShellCanUse(String),
}

/// How large a screen is, in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolution {
    /// How many across.
    width: u32,
    /// How many down.
    height: u32,
}

impl Resolution {
    /// How many across.
    #[must_use]
    pub const fn width(self) -> u32 {
        self.width
    }

    /// How many down.
    #[must_use]
    pub const fn height(self) -> u32 {
        self.height
    }
}

/// How large a screen is, in millimetres of glass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Millimetres {
    /// How wide.
    width: u32,
    /// How tall.
    height: u32,
}

impl Millimetres {
    /// How wide.
    #[must_use]
    pub const fn width(self) -> u32 {
        self.width
    }

    /// How tall.
    #[must_use]
    pub const fn height(self) -> u32 {
        self.height
    }
}

/// One screen, as the machine reports it while it is plugged in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reported {
    /// Which socket it is in.
    socket: Socket,
    /// What it says about itself, where it says anything.
    says: Option<Panel>,
    /// How many pixels it has.
    pixels: Resolution,
    /// How large the glass is, where it says.
    millimetres: Option<Millimetres>,
    /// The name the shell holds this screen's background against.
    named_for_the_shell: DisplayId,
}

impl Reported {
    /// One screen as the machine reports it.
    ///
    /// # Errors
    /// [`NotAScreen`] for a resolution with no pixels in one direction, a
    /// physical size with nothing in one direction, or a name `alo-appearance`
    /// would not hold a background against. A screen that reports **no**
    /// physical size is not an error: it is a screen whose size cannot be
    /// worked out, and [`crate::Scale::worked_out_for`] says what happens then.
    pub fn of(
        socket: Socket,
        says: Option<Panel>,
        pixels: (u32, u32),
        millimetres: Option<(u32, u32)>,
    ) -> Result<Self, NotAScreen> {
        let (width, height) = pixels;
        if width == 0 || height == 0 {
            return Err(NotAScreen::NoPixels(width, height));
        }
        let glass = match millimetres {
            Some((across, down)) if across == 0 || down == 0 => {
                return Err(NotAScreen::NoSize(across, down));
            }
            Some((across, down)) => Some(Millimetres {
                width: across,
                height: down,
            }),
            None => None,
        };
        let named = match &says {
            Some(panel) => format!("{} {} {}", socket.name(), panel.make(), panel.model()),
            None => socket.name().to_owned(),
        };
        let named_for_the_shell = DisplayId::named(&named)
            .map_err(|_| NotAScreen::NotANameTheShellCanUse(named.clone()))?;
        Ok(Self {
            socket,
            says,
            pixels: Resolution { width, height },
            millimetres: glass,
            named_for_the_shell,
        })
    }

    /// Which socket it is in.
    #[must_use]
    pub const fn socket(&self) -> &Socket {
        &self.socket
    }

    /// What it says about itself, where it says anything.
    #[must_use]
    pub fn says(&self) -> Option<&Panel> {
        self.says.as_ref()
    }

    /// How many pixels it has.
    #[must_use]
    pub const fn pixels(&self) -> Resolution {
        self.pixels
    }

    /// How large the glass is, where it says.
    #[must_use]
    pub const fn millimetres(&self) -> Option<Millimetres> {
        self.millimetres
    }

    /// The name the shell holds this screen's background and its dock against.
    ///
    /// **The socket and the description together**, which is what
    /// `alo_appearance::DisplayId`'s own documentation describes: *a connector
    /// and the monitor's own description*. Two identical screens are two names
    /// because their sockets differ, so a background chosen for one of them
    /// does not appear on the other; and a screen that says nothing is its
    /// socket alone.
    #[must_use]
    pub const fn named_for_the_shell(&self) -> &DisplayId {
        &self.named_for_the_shell
    }

    /// Which screen this is **on its own**, ignoring every other screen
    /// attached.
    ///
    /// [`crate::whoever_is_attached`] is the one a caller wants: two screens
    /// giving the same description are told apart by their sockets, and that
    /// cannot be decided one screen at a time.
    #[must_use]
    pub fn identity_on_its_own(&self) -> Identity {
        match &self.says {
            Some(panel) => Identity::Panel(panel.clone()),
            None => Identity::Socket(self.socket.clone()),
        }
    }
}

/// Which screen each of these is, decided over the whole set — the one a
/// caller wants.
///
/// In the same order as what was handed in.
/// [`crate::identity::whoever_is_attached`] is the decision; this is it, asked
/// of what the machine reported.
#[must_use]
pub fn which_screens_these_are(attached: &[Reported]) -> Vec<Identity> {
    let says: Vec<(Socket, Option<Panel>)> = attached
        .iter()
        .map(|screen| (screen.socket.clone(), screen.says.clone()))
        .collect();
    whoever_is_attached(&says)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A screen that says what it is keeps every number it reported, and is
    /// known to the shell by its socket and its description together.
    #[test]
    fn a_reported_screen_keeps_what_it_reported() {
        let screen = Reported::of(
            Socket::named("DP-1").unwrap(),
            Some(Panel::of("Dell", "U2720Q", Some("CN-0ABC")).unwrap()),
            (3840, 2160),
            Some((596, 336)),
        )
        .unwrap();
        assert_eq!(screen.socket().name(), "DP-1");
        assert_eq!(screen.pixels().width(), 3840);
        assert_eq!(screen.pixels().height(), 2160);
        assert_eq!(screen.millimetres().unwrap().width(), 596);
        assert_eq!(screen.named_for_the_shell().name(), "DP-1 Dell U2720Q");
    }

    /// A screen that says nothing is known to the shell by its socket alone,
    /// and reporting no size is allowed rather than refused.
    #[test]
    fn a_screen_that_says_nothing_and_gives_no_size_is_still_a_screen() {
        let screen =
            Reported::of(Socket::named("eDP-1").unwrap(), None, (1920, 1080), None).unwrap();
        assert_eq!(screen.named_for_the_shell().name(), "eDP-1");
        assert_eq!(screen.millimetres(), None);
        assert_eq!(
            screen.identity_on_its_own(),
            Identity::Socket(Socket::named("eDP-1").unwrap())
        );
    }

    /// **A screen with no pixels, or half a size, is refused** — and the
    /// refusal names both numbers, because whoever reads it needs to see which
    /// of the two was nothing.
    #[test]
    fn a_screen_with_no_pixels_or_half_a_size_is_refused() {
        let socket = Socket::named("DP-1").unwrap();
        assert_eq!(
            Reported::of(socket.clone(), None, (0, 1080), None),
            Err(NotAScreen::NoPixels(0, 1080))
        );
        assert_eq!(
            Reported::of(socket.clone(), None, (1920, 0), None),
            Err(NotAScreen::NoPixels(1920, 0))
        );
        assert_eq!(
            Reported::of(socket, None, (1920, 1080), Some((0, 336))),
            Err(NotAScreen::NoSize(0, 336))
        );
        assert_eq!(
            NotAScreen::NoPixels(0, 1080).to_string(),
            "a screen has a width and a height in pixels: 0 by 1080 is not one"
        );
    }

    /// **Which screen each of these is, is asked of the whole set**, so two
    /// screens with one description between them are told apart by their
    /// sockets here too.
    #[test]
    fn which_screens_these_are_is_asked_of_the_whole_set() {
        let twin = Panel::of("Acme", "P24", None).unwrap();
        let attached = [
            Reported::of(
                Socket::named("DP-1").unwrap(),
                Some(twin.clone()),
                (1920, 1080),
                None,
            )
            .unwrap(),
            Reported::of(
                Socket::named("DP-2").unwrap(),
                Some(twin),
                (1920, 1080),
                None,
            )
            .unwrap(),
        ];
        assert_eq!(
            which_screens_these_are(&attached),
            vec![
                Identity::Socket(Socket::named("DP-1").unwrap()),
                Identity::Socket(Socket::named("DP-2").unwrap()),
            ]
        );
        assert_ne!(
            attached.first().unwrap().identity_on_its_own(),
            Identity::Socket(Socket::named("DP-1").unwrap()),
            "on its own, a screen is what it says it is"
        );
    }
}
