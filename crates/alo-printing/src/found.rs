//! Finding the printers on the local network and on this machine's cables.
//!
//! The printing service already looks, through its own backends: DNS-SD for
//! printers announcing themselves on the local network, and USB and
//! IPP-over-USB for the ones on a cable. [`find`] asks it once and turns what
//! comes back into [`Found`] printers, each with the name it gave itself, where
//! it is, and whether it can be set up without a driver.
//!
//! # Finding never adds anything
//!
//! A list of what is nearby is an answer, and it changes nothing: [`find`]
//! sends one request, and that request is the one that lists devices. A
//! machine that acquired a printer by finding one would have made a place a
//! document can go without anybody choosing it, so adding is a separate call,
//! [`crate::set_up`], made for one printer a person chose.
//! `tests/finding_printers.rs` watches the service's side to hold that.
//!
//! # The local network's rules
//!
//! A printer is found the way `alo-nearby` finds a machine, and trusted as
//! little: finding one grants it nothing, its name is data that is checked
//! before it can be shown, and finding nothing is an answer rather than a
//! failure.

use alo_strings::{Filling, Said, Strings};

use crate::ipp::{Group, Message, Value};
use crate::printer::Called;
use crate::reached::{Reached, Speaks};
use crate::service::{PrintingService, Unanswered, next_request};
use crate::words;

/// The operation that lists the devices the printing service's backends can
/// find.
const GET_DEVICES: u16 = 0x400b;

/// How many seconds the printing service is asked to spend looking.
const LOOKING_FOR: i32 = 10;

/// The backends asked to look: the local network's, and the cable's.
const SCHEMES: [&str; 7] = ["dnssd", "ipp", "ipps", "ippusb", "usb", "socket", "lpd"];

/// A printer that was found, and has not been set up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// What it is called.
    called: Called,
    /// Where it is.
    reached: Reached,
    /// How it is spoken to.
    speaks: Speaks,
    /// The address the printing service found it at.
    device: String,
}

impl Found {
    /// What it is called.
    #[must_use]
    pub fn called(&self) -> &Called {
        &self.called
    }

    /// Where it is.
    #[must_use]
    pub fn reached(&self) -> &Reached {
        &self.reached
    }

    /// How it is spoken to, which decides whether it sets up by itself.
    #[must_use]
    pub fn speaks(&self) -> Speaks {
        self.speaks
    }

    /// The address it was found at.
    pub(crate) fn device(&self) -> &str {
        &self.device
    }

    /// The bytes the printing service reported this printer under — the
    /// address it was found at — for **digesting into an identity and nothing
    /// else**.
    ///
    /// A change to the whole machine names a found printer by the SHA-256 of
    /// these bytes (`alo_broker::Identity`), so what crosses into the
    /// privileged broker is never an address somebody could have written. Bytes
    /// rather than text, and no `Display` anywhere near them: the address is
    /// machinery, and a person never reads it.
    #[must_use]
    pub fn as_reported(&self) -> &[u8] {
        self.device.as_bytes()
    }

    /// Its line in the list of what was found.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let word = match self.reached {
            Reached::OnThisMachine => words::FOUND_ON_THIS_MACHINE,
            Reached::OnTheNetwork { .. } => words::FOUND_ON_THE_NETWORK,
        };
        strings.say(
            &word.key(),
            &self
                .called
                .fills(words::PRINTER, Filling::nothing(), strings),
        )
    }

    /// What a person chooses before it is set up.
    #[must_use]
    pub fn proposal(&self, strings: &Strings) -> Said {
        strings.say(
            &words::SET_UP_PROPOSED.key(),
            &self
                .called
                .fills(words::PRINTER, Filling::nothing(), strings),
        )
    }
}

/// Every printer this machine's printing service can find, on the local
/// network and on its cables.
///
/// Takes as long as the service spends looking — about ten seconds — and is
/// meant for a person who opened the list of printers, not for a loop.
///
/// # Errors
/// [`Unanswered`] when the service could not be asked. An empty list is not an
/// error: it is a machine with no printer near it.
pub fn find(service: &PrintingService) -> Result<Vec<Found>, Unanswered> {
    let request = Message::request(GET_DEVICES, next_request())
        .with(
            Group::Operation,
            "timeout",
            vec![Value::Integer(LOOKING_FOR)],
        )
        .with(
            Group::Operation,
            "include-schemes",
            SCHEMES.into_iter().map(Value::name).collect(),
        );
    let answer = service.exchange("/", &request)?;
    if !answer.succeeded() {
        return Err(Unanswered::NotUnderstood);
    }
    Ok(found_in(&answer))
}

/// The printers in an answer listing devices, each once, in the order the
/// service listed them.
fn found_in(answer: &Message) -> Vec<Found> {
    let mut found: Vec<Found> = Vec::new();
    for (group, attributes) in answer.groups() {
        if group != Group::Printer {
            continue;
        }
        let text = |name: &str| {
            attributes
                .iter()
                .find(|attribute| attribute.name() == name)
                .and_then(|attribute| attribute.text())
        };
        let Some(device) = text("device-uri") else {
            continue;
        };
        let Some((reached, speaks)) = Reached::of_device(device) else {
            continue;
        };
        if found.iter().any(|already| already.device == device) {
            continue;
        }
        let called = match Called::announced(text("device-make-and-model")) {
            Called::Unshowable => Called::announced(text("device-info")),
            named => named,
        };
        found.push(Found {
            called,
            reached,
            speaks,
            device: device.to_owned(),
        });
    }
    found
}
