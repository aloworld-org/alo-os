//! Why the network did not come up, in the words a person can act on.
//!
//! The environment waits for a connection with `nm-online` and, when that
//! fails, had exactly one sentence: *alo OS could not be downloaded, so nothing
//! was changed. Check that this computer is connected to the internet by a
//! network cable, then restart to try again.*
//!
//! **That sentence is right for one of the three cases it covered and
//! impossible for another.** A computer whose network card is not there at all
//! — no driver, or a driver whose firmware is missing — is told to check a
//! cable it has nowhere to plug in. A computer with the cable in and no address
//! is told to check the cable, which is already right. Only the middle case,
//! nothing plugged in, was the sentence's own.
//!
//! So on a failed wait, this asks the kernel what cards there are and what
//! state they are in, and the refusal names which.
//!
//! # Why the kernel's own answer and not the network manager's
//!
//! `nmcli` prints a device's state as a **translated string**: its gettext
//! domain is `NetworkManager`, the image ships the catalogues under
//! `/usr/share/locale`, and a person chooses their language at first boot. A
//! parser matching `unavailable` would work on an English machine and silently
//! match nothing on a German one — falling through to the same single sentence
//! while looking like progress, which is the fault this file exists to remove.
//! *Found by the desktop lane on 2026-10-20, reading the image's own binaries.*
//!
//! `ip -json link` prints `flags`, `operstate` and `link_type`, which are
//! iproute2's own constants and are not translated. Measured on 2026-10-20,
//! with a veth pair in a container's own network namespace, which is a cable
//! being pulled out:
//!
//! | | `flags` | `operstate` |
//! |---|---|---|
//! | both ends up | `BROADCAST MULTICAST UP LOWER_UP` | `UP` |
//! | peer taken down | **`NO-CARRIER`** `BROADCAST MULTICAST UP M-DOWN` | `LOWERLAYERDOWN` |
//! | freshly made | `BROADCAST MULTICAST M-DOWN` | `DOWN` |
//!
//! So **no cable is a flag**, and it is the one field that answers the question
//! directly.

use serde::Deserialize;

/// What the kernel calls an ordinary network card, wired or wireless.
///
/// Both are `ether`; the loopback is `loopback`. The environment cannot choose
/// a wireless network anyway (ADR 0128 is the decision that would change that),
/// so a wireless card with no association reads the same as a cable that is not
/// plugged in — which is the honest answer, because in both cases nothing is
/// connected and the person must do something about it.
const A_CARD: &str = "ether";

/// The flag the kernel sets when the wire is not live.
const NOTHING_ON_THE_WIRE: &str = "NO-CARRIER";

/// The flag the kernel sets when it is.
const SOMETHING_ON_THE_WIRE: &str = "LOWER_UP";

/// Why there is no network, as far as the kernel can say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhyThereIsNoNetwork {
    /// No network card at all — only the loopback, or nothing.
    ///
    /// On the certified machine this is a driver that did not load or a driver
    /// whose firmware is not there, and in both cases *check the cable* is
    /// advice a person cannot take.
    NoCard,
    /// A card is there and nothing is on the wire.
    ///
    /// The one case the old single sentence was already right about.
    NothingConnected,
    /// A card is there and the wire is live, so what failed is further out than
    /// anything this can see: no address, or the registry.
    ConnectedAndStillNoNetwork,
    /// The kernel was not asked, or did not answer in a shape this can read.
    ///
    /// **Never read as any of the three above.** A reading that did not happen
    /// must not become a sentence about a cable.
    NotRead,
}

/// One interface, from what `ip -json link` prints.
#[derive(Deserialize)]
struct Interface {
    /// `ether` for a card, `loopback` for the loopback.
    link_type: String,
    /// `NO-CARRIER`, `UP`, `LOWER_UP` and the rest, as iproute2 spells them.
    #[serde(default)]
    flags: Vec<String>,
}

impl WhyThereIsNoNetwork {
    /// Read it from what `ip -json link` printed.
    #[must_use]
    pub fn read(printed: Option<&str>) -> Self {
        let Some(printed) = printed else {
            return Self::NotRead;
        };
        let Ok(interfaces) = serde_json::from_str::<Vec<Interface>>(printed) else {
            return Self::NotRead;
        };
        let cards: Vec<&Interface> = interfaces
            .iter()
            .filter(|interface| interface.link_type == A_CARD)
            .collect();
        if cards.is_empty() {
            // An empty list is `NotRead` and not `NoCard`: `ip` printing `[]`
            // is a tool that found nothing, and this program is reached from an
            // environment the kernel booted, which has a loopback at least.
            // Nothing found at all is an instrument, not an answer.
            if interfaces.is_empty() {
                return Self::NotRead;
            }
            return Self::NoCard;
        }
        let live = |card: &&Interface| {
            card.flags.iter().any(|flag| flag == SOMETHING_ON_THE_WIRE)
                && !card.flags.iter().any(|flag| flag == NOTHING_ON_THE_WIRE)
        };
        if cards.iter().any(live) {
            Self::ConnectedAndStillNoNetwork
        } else {
            Self::NothingConnected
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The loopback, as the kernel printed it on 2026-10-20.
    const THE_LOOPBACK: &str = r#"{"ifindex":1,"ifname":"lo","flags":["LOOPBACK","UP","LOWER_UP"],"operstate":"UNKNOWN","link_type":"loopback"}"#;

    /// A card with the wire live, as the kernel printed it.
    const A_LIVE_CARD: &str = r#"{"ifindex":2,"ifname":"eth0","flags":["BROADCAST","MULTICAST","UP","LOWER_UP"],"operstate":"UP","link_type":"ether"}"#;

    /// The same card with the other end taken down — a cable pulled out.
    const A_CARD_WITH_NOTHING_ON_IT: &str = r#"{"ifindex":2,"ifname":"eth0","flags":["NO-CARRIER","BROADCAST","MULTICAST","UP","M-DOWN"],"operstate":"LOWERLAYERDOWN","link_type":"ether"}"#;

    /// A card that was never brought up.
    const A_CARD_NEVER_BROUGHT_UP: &str = r#"{"ifindex":2,"ifname":"eth0","flags":["BROADCAST","MULTICAST","M-DOWN"],"operstate":"DOWN","link_type":"ether"}"#;

    /// What `ip -json link` prints for these interfaces.
    fn printed(interfaces: &[&str]) -> String {
        format!("[{}]", interfaces.join(","))
    }

    /// **A machine with nothing but a loopback has no card**, which is what a
    /// driver that did not load, or one whose firmware is not there, looks like
    /// from here. It is the case *check the cable* cannot be acted on.
    #[test]
    fn only_a_loopback_is_no_card() {
        assert_eq!(
            WhyThereIsNoNetwork::read(Some(&printed(&[THE_LOOPBACK]))),
            WhyThereIsNoNetwork::NoCard
        );
    }

    /// **A card with `NO-CARRIER` is nothing connected**, which is the one case
    /// the old single sentence was right about.
    #[test]
    fn no_carrier_is_nothing_connected() {
        assert_eq!(
            WhyThereIsNoNetwork::read(Some(&printed(&[THE_LOOPBACK, A_CARD_WITH_NOTHING_ON_IT]))),
            WhyThereIsNoNetwork::NothingConnected
        );
    }

    /// **A card that was never brought up is nothing connected too.** It has no
    /// `NO-CARRIER` flag and no `LOWER_UP` either, and the absence of the
    /// second is what decides it — which is why the test is here and not left
    /// to the reader to assume.
    #[test]
    fn a_card_never_brought_up_is_nothing_connected() {
        assert_eq!(
            WhyThereIsNoNetwork::read(Some(&printed(&[THE_LOOPBACK, A_CARD_NEVER_BROUGHT_UP]))),
            WhyThereIsNoNetwork::NothingConnected
        );
    }

    /// **A live card means the failure is further out than this can see.**
    #[test]
    fn a_live_card_sends_the_question_further_out() {
        assert_eq!(
            WhyThereIsNoNetwork::read(Some(&printed(&[THE_LOOPBACK, A_LIVE_CARD]))),
            WhyThereIsNoNetwork::ConnectedAndStillNoNetwork
        );
    }

    /// **One live card among several dead ones is still a live card**, because
    /// the question is whether *any* road out is up.
    #[test]
    fn one_live_card_among_dead_ones_counts() {
        assert_eq!(
            WhyThereIsNoNetwork::read(Some(&printed(&[
                THE_LOOPBACK,
                A_CARD_WITH_NOTHING_ON_IT,
                A_LIVE_CARD,
            ]))),
            WhyThereIsNoNetwork::ConnectedAndStillNoNetwork
        );
    }

    /// **`NO-CARRIER` beats `LOWER_UP` on the same card.**
    ///
    /// The kernel prints both in the pulled-cable reading above — `UP` is
    /// there, `M-DOWN` is there — so a check that only looked for `LOWER_UP`
    /// would have to be sure `NO-CARRIER` never appears beside it. It does not
    /// in anything measured, and this holds the rule rather than the
    /// observation.
    #[test]
    fn nothing_on_the_wire_beats_something_on_it() {
        let both = r#"{"flags":["NO-CARRIER","UP","LOWER_UP"],"link_type":"ether"}"#;
        assert_eq!(
            WhyThereIsNoNetwork::read(Some(&printed(&[both]))),
            WhyThereIsNoNetwork::NothingConnected
        );
    }

    /// **Nothing that is not an answer becomes one**, and in particular an
    /// empty list is the instrument and not a machine without a card: this runs
    /// in an environment the kernel booted, which has a loopback at least.
    #[test]
    fn nothing_that_is_not_an_answer_becomes_one() {
        for not_an_answer in [
            None,
            Some(""),
            Some("[]"),
            Some("null"),
            Some("not json"),
            // `ip` is not there, or refused: its own words, not JSON.
            Some("Command \"link\" is unknown, try \"ip help\"."),
            // JSON, and not this JSON.
            Some(r#"{"ifname":"eth0"}"#),
        ] {
            assert_eq!(
                WhyThereIsNoNetwork::read(not_an_answer),
                WhyThereIsNoNetwork::NotRead,
                "{not_an_answer:?}"
            );
        }
    }

    /// **An interface with no `flags` at all is not a live one.**
    ///
    /// `flags` is defaulted rather than required, so a future `ip` that omits
    /// it does not make every reading `NotRead` — but a card with no flags must
    /// not read as connected either.
    #[test]
    fn a_card_with_no_flags_is_not_connected() {
        assert_eq!(
            WhyThereIsNoNetwork::read(Some(r#"[{"link_type":"ether"}]"#)),
            WhyThereIsNoNetwork::NothingConnected
        );
    }
}
