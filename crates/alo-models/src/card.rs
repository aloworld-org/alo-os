//! What this machine has to draw with, read from what the kernel says about it.
//!
//! [ADR 0007](../../../docs/decisions/0007-the-cpu-is-the-default.md) settled
//! the principle — *the CPU is the default; a GPU is acceleration*, and *a GPU
//! changes speed, not capability* — and nothing implemented it. Every
//! `on_the_gpu_bytes` in this repository was [`None`], because nothing ever
//! looked at a card, so a machine with one ran exactly as slowly as a machine
//! without. This file is the looking.
//!
//! Everything here comes out of `/sys/class/drm`, which is the kernel's own
//! list of what draws on this machine, read the way `alo_power`'s battery is
//! read: files, not a daemon, and not a program of ours run as a subprocess.
//! A card is hardware, the kernel already knows about it, and a reading that
//! went through something else would be a reading that stops when that thing
//! does.
//!
//! # *No driver* and *no card* are different answers, and the difference is
//! the whole file
//!
//! A machine with an NVIDIA card and no NVIDIA driver must read as **no
//! driver**, never as *no card*. The base is rented and unmodified
//! ([ADR 0011](../../../docs/decisions/0011-the-base-is-rented-and-the-image-is-a-container.md))
//! and ships no such driver, so that is the likely state of every NVIDIA
//! machine alo OS meets today rather than an edge case. A person told *no card*
//! goes looking for hardware they already own; a person told *there is a card
//! here and nothing on this machine can reach it* knows what is true. The two
//! sentences send them to different shops.
//!
//! So [`ACard`] carries the vendor the bus reports **and** whether a driver is
//! bound to it, separately, and [`crate::road`] refuses to collapse them.
//!
//! # A card that will not say how much memory it has
//!
//! `amdgpu` publishes `mem_info_vram_total`; the proprietary NVIDIA driver
//! publishes nothing of the kind, and an integrated processor's memory is the
//! machine's. So [`ACard::memory_bytes`] is an [`Option`], and [`None`] means
//! *this machine did not say* rather than *none*. Nothing here guesses one from
//! a model name or from the catalogue, which is the assumption ADR 0007
//! rejected: what a card really held is what the runtime reports having put
//! there, and that arrives as [`crate::Loaded`].

use std::path::{Path, PathBuf};

/// Where the kernel lists what draws on this machine.
pub const WHERE_THE_CARDS_ARE: &str = "/sys/class/drm";

/// **Who made a graphics device**, from the identifier the bus carries.
///
/// Three are named because the pinned runtime's answer differs between them
/// and a person reads the name; everything else is [`Vendor::Someone`] with its
/// number, which is honest about a device nobody here has seen rather than
/// silent about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Vendor {
    /// NVIDIA.
    Nvidia,
    /// AMD.
    Amd,
    /// Intel.
    Intel,
    /// Somebody else, by the identifier on the bus.
    Someone(u16),
}

impl Vendor {
    /// NVIDIA's identifier on the PCI bus.
    pub const NVIDIA: u16 = 0x10de;
    /// AMD's.
    pub const AMD: u16 = 0x1002;
    /// Intel's.
    pub const INTEL: u16 = 0x8086;

    /// Who this identifier belongs to.
    #[must_use]
    pub fn on_the_bus(identifier: u16) -> Self {
        match identifier {
            Self::NVIDIA => Self::Nvidia,
            Self::AMD => Self::Amd,
            Self::INTEL => Self::Intel,
            other => Self::Someone(other),
        }
    }

    /// The identifier the bus carries for this vendor.
    #[must_use]
    pub fn identifier(self) -> u16 {
        match self {
            Self::Nvidia => Self::NVIDIA,
            Self::Amd => Self::AMD,
            Self::Intel => Self::INTEL,
            Self::Someone(other) => other,
        }
    }

    /// **The name a person reads**, which is a proper noun and is never
    /// translated.
    ///
    /// A vendor nobody here has seen is named by its identifier, in the
    /// spelling the kernel writes — `0x1af4` — because that is the string
    /// somebody can look up.
    #[must_use]
    pub fn named(self) -> String {
        match self {
            Self::Nvidia => "NVIDIA".to_owned(),
            Self::Amd => "AMD".to_owned(),
            Self::Intel => "Intel".to_owned(),
            Self::Someone(other) => format!("0x{other:04x}"),
        }
    }
}

/// **One graphics device this machine has**, as the kernel describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ACard {
    /// Where the kernel keeps it.
    at: PathBuf,
    /// Who made it.
    made_by: Vendor,
    /// The driver bound to it, or [`None`] where nothing is.
    driver: Option<String>,
    /// The memory it reported, or [`None`] where this machine did not say.
    memory_bytes: Option<u64>,
}

impl ACard {
    /// Where the kernel keeps it.
    #[must_use]
    pub fn where_it_is(&self) -> &Path {
        &self.at
    }

    /// Who made it.
    #[must_use]
    pub fn made_by(&self) -> Vendor {
        self.made_by
    }

    /// **The driver bound to it**, or [`None`] where nothing on this machine
    /// can reach it.
    ///
    /// [`None`] is the answer that matters: a card with no driver is a card,
    /// and this crate never reports it as an absent one.
    #[must_use]
    pub fn driver(&self) -> Option<&str> {
        self.driver.as_deref()
    }

    /// **How much memory the card reported**, or [`None`] where this machine
    /// did not say — which is most machines, and is not zero.
    #[must_use]
    pub fn memory_bytes(&self) -> Option<u64> {
        self.memory_bytes
    }

    /// Whether anything on this machine can reach it.
    #[must_use]
    pub fn is_reachable(&self) -> bool {
        self.driver.is_some()
    }

    /// One device, read from the folder the kernel keeps it in, or [`None`]
    /// where that folder describes no device this crate can name.
    fn at(folder: &Path) -> Option<Self> {
        let device = folder.join("device");
        let made_by = Vendor::on_the_bus(an_identifier(&said(&device, "vendor")?)?);
        Some(Self {
            at: folder.to_path_buf(),
            made_by,
            driver: the_driver(&device),
            memory_bytes: said(&device, "mem_info_vram_total").and_then(|said| said.parse().ok()),
        })
    }
}

/// **What this machine has to draw with.**
///
/// Three answers rather than a list that might be empty, because an empty list
/// would make *this machine has no card* and *this machine would not say* the
/// same sentence, and they are not the same fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatDrawsHere {
    /// The kernel's own list was not there to read. Not an error and not a
    /// card: alo OS is not the only place this code compiles, and a machine
    /// whose kernel keeps no such list has not said it has nothing.
    TheMachineWouldNotSay,
    /// The list is there, and nothing on this machine's bus draws.
    NoCard,
    /// These, in the order the kernel lists them.
    These(Vec<ACard>),
}

impl WhatDrawsHere {
    /// **What this machine has**, read now.
    ///
    /// Read each time rather than once: a card can be passed to a machine after
    /// it booted, and an answer cached at start-up is an answer about a
    /// different machine.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self::among(Path::new(WHERE_THE_CARDS_ARE))
    }

    /// The same question, asked of a list a test wrote.
    ///
    /// Public for the reason `alo_power`'s is: the only way to test every shape
    /// of a machine on one machine is to hand the reader a folder, and a reader
    /// that could only ever look at `/sys` would be a reader nobody could check.
    #[must_use]
    pub fn among(drm: &Path) -> Self {
        let Ok(listing) = std::fs::read_dir(drm) else {
            return Self::TheMachineWouldNotSay;
        };
        let mut folders: Vec<PathBuf> = listing
            .filter_map(|entry| Some(entry.ok()?.path()))
            .filter(|at| is_a_card(at))
            .collect();
        folders.sort();
        let cards: Vec<ACard> = folders.iter().filter_map(|at| ACard::at(at)).collect();
        if cards.is_empty() {
            Self::NoCard
        } else {
            Self::These(cards)
        }
    }

    /// Every card, which is empty on the two answers that carry none.
    #[must_use]
    pub fn cards(&self) -> &[ACard] {
        match self {
            Self::TheMachineWouldNotSay | Self::NoCard => &[],
            Self::These(cards) => cards,
        }
    }
}

/// Whether a folder in the kernel's list is a device rather than a connector.
///
/// The list carries `card0`, `card0-eDP-1`, `renderD128` and a `version` file.
/// Only the first shape is a device; `card0-eDP-1` is a screen plugged into it,
/// and counting one would report a laptop with one card as having two.
fn is_a_card(at: &Path) -> bool {
    let Some(name) = at.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let Some(number) = name.strip_prefix("card") else {
        return false;
    };
    !number.is_empty() && number.chars().all(|letter| letter.is_ascii_digit())
}

/// The driver bound to a device, or [`None`] where nothing is bound.
///
/// Read from `uevent`'s `DRIVER=` line rather than from the `driver` symlink
/// beside it. The two say the same thing, and the kernel writes the line only
/// when a driver is actually bound — which is the question this file exists to
/// answer. It is also a plain file, so a test can write a machine that does not
/// exist without needing the privilege a symlink costs on some hosts.
///
/// The link is read as well, for a kernel whose `uevent` is silent: a device
/// with a driver must never read as one without.
fn the_driver(device: &Path) -> Option<String> {
    let named = said(device, "uevent")
        .and_then(|uevent| {
            uevent
                .lines()
                .find_map(|line| line.trim().strip_prefix("DRIVER=").map(str::to_owned))
        })
        .or_else(|| {
            let bound = std::fs::read_link(device.join("driver")).ok()?;
            Some(bound.file_name()?.to_str()?.to_owned())
        })?;
    let named = named.trim().to_owned();
    (!named.is_empty()).then_some(named)
}

/// One of the kernel's files, trimmed, or [`None`] where there is none.
fn said(at: &Path, file: &str) -> Option<String> {
    Some(
        std::fs::read_to_string(at.join(file))
            .ok()?
            .trim()
            .to_owned(),
    )
}

/// A vendor identifier as the kernel writes one — `0x10de`.
fn an_identifier(written: &str) -> Option<u16> {
    u16::from_str_radix(written.trim().strip_prefix("0x")?, 16).ok()
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::a_list_of_what_draws;

    /// **A machine with nothing on its bus says so, and it is not an error.**
    #[test]
    fn a_list_with_nothing_in_it_is_no_card_rather_than_no_answer() {
        let drm = a_list_of_what_draws();
        assert_eq!(WhatDrawsHere::among(drm.at()), WhatDrawsHere::NoCard);
    }

    /// **A machine with no list at all has not said it has nothing.**
    #[test]
    fn a_list_that_is_not_there_is_a_machine_that_would_not_say() {
        let drm = a_list_of_what_draws();
        assert_eq!(
            WhatDrawsHere::among(&drm.at().join("no-such-folder")),
            WhatDrawsHere::TheMachineWouldNotSay,
        );
    }

    /// **A connector is not a card**, and a laptop with one card and two
    /// screens has one card.
    #[test]
    fn screens_plugged_into_a_card_are_not_more_cards() {
        let drm = a_list_of_what_draws();
        drm.holding("card0", "0x8086", Some("i915"), None);
        for connector in ["card0-eDP-1", "card0-DP-2", "renderD128"] {
            std::fs::create_dir_all(drm.at().join(connector)).expect("a connector");
        }
        let here = WhatDrawsHere::among(drm.at());
        assert_eq!(here.cards().len(), 1, "{here:?}");
        assert_eq!(
            here.cards().first().map(ACard::made_by),
            Some(Vendor::Intel)
        );
    }

    /// **A card with no driver is a card.** The whole of this file's reason.
    #[test]
    fn a_card_nothing_can_reach_is_reported_as_a_card_with_no_driver() {
        let drm = a_list_of_what_draws();
        drm.holding("card0", "0x10de", None, None);
        let here = WhatDrawsHere::among(drm.at());
        assert_ne!(
            here,
            WhatDrawsHere::NoCard,
            "an NVIDIA card read as no card"
        );
        let card = here.cards().first().expect("the card that is there");
        assert_eq!(card.made_by(), Vendor::Nvidia);
        assert_eq!(card.driver(), None);
        assert!(!card.is_reachable());
        assert_eq!(card.where_it_is(), drm.at().join("card0"));
    }

    /// **A vendor nobody here has seen is named by its number**, not dropped.
    #[test]
    fn a_device_from_somebody_else_is_still_a_device() {
        let drm = a_list_of_what_draws();
        drm.holding("card0", "0x1af4", Some("virtio_gpu"), None);
        let here = WhatDrawsHere::among(drm.at());
        let card = here.cards().first().expect("the card that is there");
        assert_eq!(card.made_by(), Vendor::Someone(0x1af4));
        assert_eq!(card.made_by().named(), "0x1af4");
        assert_eq!(card.made_by().identifier(), 0x1af4);
        assert_eq!(card.driver(), Some("virtio_gpu"));
    }

    /// **A card that would not say how much memory it has says [`None`]**, and
    /// a card that says is read as it wrote it.
    #[test]
    fn memory_is_read_where_it_is_published_and_never_guessed() {
        let drm = a_list_of_what_draws();
        drm.holding("card0", "0x1002", Some("amdgpu"), Some(8_573_157_888));
        drm.holding("card1", "0x10de", Some("nvidia"), None);
        let here = WhatDrawsHere::among(drm.at());
        assert_eq!(here.cards().len(), 2, "{here:?}");
        let mut memory = here.cards().iter().map(ACard::memory_bytes);
        assert_eq!(memory.next(), Some(Some(8_573_157_888)));
        assert_eq!(memory.next(), Some(None));
    }

    /// **A folder that says nothing a vendor could be read from is not a
    /// card.** A reading that invented one would be the assumption ADR 0007
    /// rejected.
    #[test]
    fn a_folder_with_no_vendor_in_it_is_not_counted() {
        let drm = a_list_of_what_draws();
        std::fs::create_dir_all(drm.at().join("card0").join("device"))
            .expect("a folder with nothing in it");
        assert_eq!(WhatDrawsHere::among(drm.at()), WhatDrawsHere::NoCard);

        let drm = a_list_of_what_draws();
        let device = drm.at().join("card0").join("device");
        std::fs::create_dir_all(&device).expect("a folder");
        std::fs::write(device.join("vendor"), "not a number\n").expect("a vendor that is not one");
        assert_eq!(WhatDrawsHere::among(drm.at()), WhatDrawsHere::NoCard);
    }

    /// **A kernel whose `uevent` is silent still names a bound driver**, from
    /// the link beside it — on a machine that has links.
    #[cfg(unix)]
    #[test]
    fn a_driver_named_only_by_the_link_is_still_a_driver() {
        let drm = a_list_of_what_draws();
        let device = drm.at().join("card0").join("device");
        std::fs::create_dir_all(device.join("amdgpu")).expect("a folder for the driver");
        std::fs::write(device.join("vendor"), "0x1002\n").expect("a vendor");
        std::os::unix::fs::symlink(device.join("amdgpu"), device.join("driver"))
            .expect("a driver link");
        let here = WhatDrawsHere::among(drm.at());
        let card = here.cards().first().expect("the card that is there");
        assert_eq!(card.driver(), Some("amdgpu"));
        assert!(card.is_reachable());
    }

    /// The three named vendors are the three the bus identifies.
    #[test]
    fn the_three_named_vendors_read_back_as_themselves() {
        for (identifier, vendor) in [
            (Vendor::NVIDIA, Vendor::Nvidia),
            (Vendor::AMD, Vendor::Amd),
            (Vendor::INTEL, Vendor::Intel),
        ] {
            assert_eq!(Vendor::on_the_bus(identifier), vendor);
            assert_eq!(vendor.identifier(), identifier);
            assert!(!vendor.named().is_empty());
        }
    }
}
