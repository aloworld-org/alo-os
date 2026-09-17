//! A Bluetooth service to decide against, for tests.
//!
//! Every decision this crate makes is made against [`BluetoothService`], so all
//! of them are tested on machines with no radio in them — which is most machines
//! this gate runs on, and every virtual one.
//!
//! `#[doc(hidden)]` and in `src/` rather than in a test file, because the
//! crate's own tests and the tests that hold this crate against `alo-nearby` and
//! `alo-capability` both need a service, and two copies of a fixture drift until
//! one of them is testing something nobody meant.

#![doc(hidden)]

use std::cell::RefCell;

use crate::choosing::Chosen;
use crate::confirming::{Answer, Asked};
use crate::radio::Radio;
use crate::reported::{Address, DeviceName, Found, Kind, TheDevices};
use crate::service::{BluetoothService, Devices, NotAnswering, NotDone};

/// A device, for a test that needs one.
///
/// # Panics
/// On an address that is not one, which in a fixture is a typo rather than a
/// machine.
#[doc(hidden)]
#[must_use]
#[expect(
    clippy::panic,
    reason = "a fixture built on an address that is not one is a typo in a test, and a panic \
              naming it is how the test says so"
)]
pub fn a_device(address: &str, called: &str, kind: Kind, paired: bool) -> Found {
    let address = Address::reported(address).unwrap_or_else(|why| panic!("{why}"));
    let called = DeviceName::reported(called, &address);
    Found::reported(address, called, kind, paired)
}

/// **What a fake service was told**, one line each.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Told {
    /// Pair with this device.
    Pair(String),
    /// Forget this device.
    Forget(String),
    /// Turn the radio on or off.
    Radio(Radio),
    /// Start or stop looking.
    Look(bool),
}

/// A Bluetooth service with no radio behind it, which remembers what it was
/// told.
#[doc(hidden)]
#[derive(Debug)]
pub struct AService {
    /// What it says when it is asked.
    devices: TheDevices,
    /// What each device will ask for.
    will_ask: Asked,
    /// What the person at this machine says when they are asked.
    the_person_says: Answer,
    /// What it has been told, in order.
    told: RefCell<Vec<Told>>,
}

impl AService {
    /// A service reporting these devices, with the radio on.
    #[doc(hidden)]
    #[must_use]
    pub fn with(found: Vec<Found>) -> Self {
        Self {
            devices: TheDevices::reported(found, Radio::On, true),
            will_ask: Asked::Nothing,
            the_person_says: Answer::Yes,
            told: RefCell::new(Vec::new()),
        }
    }

    /// The same, where every device asks for this before it will pair.
    #[doc(hidden)]
    #[must_use]
    pub fn asking(mut self, asked: Asked) -> Self {
        self.will_ask = asked;
        self
    }

    /// The same, where the person at this machine says no to whatever they are
    /// shown.
    #[doc(hidden)]
    #[must_use]
    pub fn where_the_person_says_no(mut self) -> Self {
        self.the_person_says = Answer::No;
        self
    }

    /// What every device here will ask for before it pairs.
    #[doc(hidden)]
    #[must_use]
    pub const fn will_ask(&self) -> &Asked {
        &self.will_ask
    }

    /// The same, with the radio off.
    #[doc(hidden)]
    #[must_use]
    pub fn with_the_radio_off(mut self) -> Self {
        self.devices = TheDevices::reported(self.devices.every().to_vec(), Radio::Off, true);
        self
    }

    /// A machine with no Bluetooth in it at all.
    #[doc(hidden)]
    #[must_use]
    pub fn with_no_radio() -> Self {
        Self {
            devices: TheDevices::without_a_radio(),
            will_ask: Asked::Nothing,
            the_person_says: Answer::Yes,
            told: RefCell::new(Vec::new()),
        }
    }

    /// What it has been told, in order.
    #[doc(hidden)]
    #[must_use]
    pub fn told(&self) -> Vec<Told> {
        self.told.borrow().clone()
    }
}

impl Devices for AService {
    fn now(&self) -> Result<TheDevices, NotAnswering> {
        Ok(self.devices.clone())
    }
}

impl BluetoothService for AService {
    fn pair(&self, chosen: &Chosen) -> Result<(), NotDone> {
        if !self.the_person_says.is_yes() {
            return Err(NotDone::NobodySaidYes);
        }
        self.told
            .borrow_mut()
            .push(Told::Pair(chosen.address().as_str().to_owned()));
        Ok(())
    }

    fn forget(&self, address: &Address) -> Result<(), NotDone> {
        self.told
            .borrow_mut()
            .push(Told::Forget(address.as_str().to_owned()));
        Ok(())
    }

    fn switch_radio(&self, to: Radio) -> Result<(), NotDone> {
        self.told.borrow_mut().push(Told::Radio(to));
        Ok(())
    }

    fn look(&self, looking: bool) -> Result<(), NotDone> {
        self.told.borrow_mut().push(Told::Look(looking));
        Ok(())
    }
}
