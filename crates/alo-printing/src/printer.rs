//! A printer this machine prints on, and the name a person knows it by.
//!
//! **A person knows a printer by the name it gave itself** — its maker and
//! model, as it announced them — and by nothing else. The queue the printing
//! service keeps it under is derived here, never chosen by anybody and never
//! shown: [`Queue`] has no road to a sentence.

use alo_strings::{Filling, Strings};

use crate::reached::Reached;
use crate::words;

/// The most characters a printer's own name may be and still be shown.
const LONGEST_NAME: usize = 127;

/// What a printer is called.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Called {
    /// The name it gave itself.
    Named(String),
    /// It gave none, or one that could not be shown on a line.
    Unshowable,
}

impl Called {
    /// What a printer is called, from the name it announced.
    ///
    /// A name with a character in it that cannot be shown on one line is not
    /// shown at all rather than shown cleaned: a printer's name is whatever
    /// the device on the network sent, and a line in a list that could be made
    /// to say something else is the indicator's problem in another place.
    #[must_use]
    pub fn announced(name: Option<&str>) -> Self {
        let Some(name) = name.map(str::trim) else {
            return Self::Unshowable;
        };
        if name.is_empty()
            || name.eq_ignore_ascii_case("unknown")
            || name.chars().count() > LONGEST_NAME
            || name.chars().any(char::is_control)
        {
            return Self::Unshowable;
        }
        Self::Named(name.to_owned())
    }

    /// This name, put into the gap named `gap` of another sentence.
    ///
    /// A name a printer gave is data and is put in as it is; the words for a
    /// printer with no name are this crate's, and carry whether anybody
    /// translated them.
    #[must_use]
    pub fn fills(&self, gap: &str, filling: Filling, strings: &Strings) -> Filling {
        match self {
            Self::Named(name) => filling.and(gap, name.clone()),
            Self::Unshowable => filling.and_said(
                gap,
                &strings.say(&words::A_PRINTER_WITHOUT_A_NAME.key(), &Filling::nothing()),
            ),
        }
    }
}

/// The name the printing service keeps a printer under.
///
/// Derived from what the printer is and where it is, so the same printer found
/// twice is the same queue, and two printers of one model are two. Never
/// shown: nothing here turns one into words.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Queue(String);

impl Queue {
    /// The queue for the printer at this address, called this.
    #[must_use]
    pub(crate) fn for_device(called: &Called, device: &str) -> Self {
        let mut slug = String::from("alo-");
        if let Called::Named(name) = called {
            let mut last_was_dash = true;
            for letter in name.chars() {
                if slug.len() >= 40 {
                    break;
                }
                if letter.is_ascii_alphanumeric() {
                    slug.push(letter.to_ascii_lowercase());
                    last_was_dash = false;
                } else if !last_was_dash {
                    slug.push('-');
                    last_was_dash = true;
                }
            }
        } else {
            slug.push_str("printer-");
        }
        if !slug.ends_with('-') {
            slug.push('-');
        }
        slug.push_str(&format!("{:016x}", fnv(device.as_bytes())));
        Self(slug)
    }

    /// A queue by the name the printing service answered with, if it is one
    /// this crate could have made.
    #[must_use]
    pub(crate) fn answered(name: &str) -> Option<Self> {
        let plain = !name.is_empty()
            && name.len() <= LONGEST_NAME
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        plain.then(|| Self(name.to_owned()))
    }

    /// The queue's name, for the path and the address a request is sent to.
    #[must_use]
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    /// The address the printing service knows this queue by.
    #[must_use]
    pub(crate) fn address(&self) -> String {
        format!("ipp://localhost/printers/{}", self.as_str())
    }

    /// The path a request about this queue is sent to.
    #[must_use]
    pub(crate) fn path(&self) -> String {
        format!("/printers/{}", self.as_str())
    }
}

/// A 64-bit FNV-1a digest: stable across builds and machines, which is all a
/// queue's name needs from it. Not a security property — two devices whose
/// addresses collide would share a name, and the printing service would say
/// the second one replaced the first.
fn fnv(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// A printer set up on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Printer {
    /// The queue the service keeps it under.
    queue: Queue,
    /// What a person knows it as.
    called: Called,
    /// Where it is.
    reached: Reached,
}

impl Printer {
    /// A printer set up under this queue.
    pub(crate) fn of(queue: Queue, called: Called, reached: Reached) -> Self {
        Self {
            queue,
            called,
            reached,
        }
    }

    /// What a person knows it as.
    #[must_use]
    pub fn called(&self) -> &Called {
        &self.called
    }

    /// Where it is.
    #[must_use]
    pub fn reached(&self) -> &Reached {
        &self.reached
    }

    /// The queue it is kept under.
    pub(crate) fn queue(&self) -> &Queue {
        &self.queue
    }

    /// The bytes the printing service reports this printer under — the name
    /// of the queue it keeps it in — for **digesting into an identity and
    /// nothing else**.
    ///
    /// The queue is derived and never shown ([`Queue`]); what leaves this crate
    /// is these bytes, which the privileged broker's callers turn into an
    /// `alo_broker::Identity`, and which the broker compares with what the
    /// printing service reports at the moment it carries a change out.
    #[must_use]
    pub fn as_reported(&self) -> &[u8] {
        self.queue.as_str().as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A name a printer sent that could make a line say something else is not
    /// shown, and nothing is not a name.
    #[test]
    fn a_name_that_cannot_be_shown_on_a_line_is_not_shown() {
        assert_eq!(
            Called::announced(Some(" Brother HL-L2350DW series ")),
            Called::Named("Brother HL-L2350DW series".to_owned())
        );
        for name in [
            None,
            Some(""),
            Some("Unknown"),
            Some("Printer\nYour files are being uploaded"),
            Some("\u{1b}[2J"),
        ] {
            assert_eq!(Called::announced(name), Called::Unshowable, "{name:?}");
        }
        assert_eq!(
            Called::announced(Some(&"x".repeat(128))),
            Called::Unshowable
        );
    }

    /// **A queue is derived, stable, and different for two printers of one
    /// model** — so setting up a second one never quietly repoints the first.
    #[test]
    fn a_queue_is_derived_stable_and_one_per_printer() {
        let called = Called::Named("HP LaserJet Pro M404".to_owned());
        let first = Queue::for_device(&called, "ipp://192.168.1.20/ipp/print");
        assert_eq!(
            first,
            Queue::for_device(&called, "ipp://192.168.1.20/ipp/print")
        );
        assert_ne!(
            first,
            Queue::for_device(&called, "ipp://192.168.1.21/ipp/print")
        );
        assert!(
            first.as_str().starts_with("alo-hp-laserjet-pro-m404-"),
            "{first:?}"
        );
        assert_eq!(Queue::answered(first.as_str()), Some(first.clone()));
        let nameless = Queue::for_device(&Called::Unshowable, "usb://x");
        assert!(
            nameless.as_str().starts_with("alo-printer-"),
            "{nameless:?}"
        );
        assert_eq!(Queue::answered("../../etc"), None);
        assert_eq!(Queue::answered(""), None);
    }
}
