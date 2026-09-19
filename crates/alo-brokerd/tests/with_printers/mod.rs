//! A composite broker whose non-printing services fail if printer work reaches them.

#![expect(
    clippy::panic,
    reason = "an unrelated service being reached fails the test"
)]

use std::path::Path;

use alo_brokerd::{Carriers, Network, PrintService, Printers, Proxy, Storage};
use alo_drives::{Drive, DriveService, Drives, Filesystem, LoginName, TheDrives};
use alo_networks::{NetworkService, Networks, Saved, TheNetworks, Visible};

/// A service a printer change must never reach.
#[derive(Debug)]
pub struct Nothing;

impl Networks for Nothing {
    fn now(&self) -> Result<TheNetworks, alo_networks::NotAnswering> {
        panic!("a printer change asked the network manager")
    }
}

impl NetworkService for Nothing {
    fn join(&self, _: &Visible) -> Result<(), alo_networks::NotDone> {
        panic!("a printer change joined a network")
    }

    fn forget(&self, _: &Saved) -> Result<(), alo_networks::NotDone> {
        panic!("a printer change forgot a network")
    }

    fn switch_wireless(&self, _: bool) -> Result<(), alo_networks::NotDone> {
        panic!("a printer change switched wireless")
    }
}

impl Drives for Nothing {
    fn now(&self) -> Result<TheDrives, alo_drives::NotAnswering> {
        panic!("a printer change asked the disk service")
    }
}

impl DriveService for Nothing {
    fn mount(&self, _: &Drive, _: &Filesystem, _: &LoginName) -> Result<(), alo_drives::NotDone> {
        panic!("a printer change mounted a drive")
    }

    fn eject(&self, _: &Drive) -> Result<(), alo_drives::NotDone> {
        panic!("a printer change ejected a drive")
    }
}

/// The published constructor and two-parameter type still work unchanged.
pub fn without_printers() -> Carriers<Nothing, Nothing> {
    Carriers::of(
        Network::against(Nothing),
        Proxy::handed_over(
            Path::new("/nonexistent-printer-test/wanted.json"),
            Path::new("/nonexistent-printer-test/proxy.json"),
            alo_broker::our_user(),
        ),
        Storage::against(
            Nothing,
            Path::new("/nonexistent-printer-test/passwd"),
            alo_broker::our_user(),
        ),
    )
}

/// The printer service is the only service these carriers may reach.
pub fn carriers<P: PrintService>(service: P) -> Carriers<Nothing, Nothing, P> {
    without_printers().with_printers(Printers::against(service))
}
