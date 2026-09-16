//! The machine Settings' tests are written against: a real home directory on a
//! real disk, the machine's grants and the daemon's pairings in real files,
//! and a daemon's door that counts what it was asked.
//!
//! The person has added a provider and chosen one of its models; the
//! machine holds one grant, of a folder to `@files`; and this machine is paired
//! with the one at reception, which may ask its models. Nothing else is set up:
//! appearance, the dock and shortcuts have no file, which is every machine on
//! its first morning.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only,
//! and it lives outside `src/` because it writes a pairings file to stand the
//! machine up — `/var/lib/alo/pairings.toml` has one writer in shipped source,
//! the daemon (`alo-changing/tests/the_pairings_file_has_one_writer.rs`), and a
//! fixture is not shipped.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::time::Duration;

use alo_appearance::{Scheme, TextScale};
use alo_capability::{Grant, Grants, Reach};
use alo_changing::{Knocking, RevokingPairings, Stood, Unpaired};
use alo_choosing::{Choosing, Picked};
use alo_models::{Provider, Region};
use alo_nearby::{Deliberating, Keying, MachineId, MayAskIts, Pairings, Proposal, Side};
use alo_strings::Direction;

use crate::approval_testing::hour;
pub(crate) use crate::approval_testing::{noon, words};
use crate::{SettingsDoors, SettingsLook, SettingsPlaces, SettingsWindow};

/// The machine at reception, which this one is paired with.
pub(crate) const RECEPTION: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f0";

/// This machine.
const STUDIO: &str = "aaaabbbbccccddddeeeeffff00001111";

/// The provider the person added.
pub(crate) const PROVIDER: &str = "Harbour AI";

/// The ordinary light look, read left to right.
pub(crate) fn light() -> SettingsLook {
    SettingsLook {
        scheme: Scheme::Light,
        scale: TextScale::ordinary(),
        reading: Direction::LeftToRight,
    }
}

/// A person's machine, in a folder of the test's own.
pub(crate) struct Machine {
    /// The folder everything is in, removed when the test is done.
    _held: tempfile::TempDir,
    /// The person's home directory.
    home: PathBuf,
    /// Where Settings finds everything.
    pub(crate) places: SettingsPlaces,
}

impl Machine {
    /// Settings, opened by hand on this machine at noon.
    pub(crate) fn opened(&self) -> SettingsWindow {
        let mut window = SettingsWindow::closed();
        let opened = window.opened_by_hand(&self.places, noon());
        assert!(opened.not_remembered.is_empty(), "{opened:?}");
        window
    }

    /// Where Settings finds everything in a session on this machine with no
    /// home directory: the machine's own grants and pairings, and no folder.
    pub(crate) fn with_no_folder(&self) -> SettingsPlaces {
        assert!(self.home.is_dir());
        SettingsPlaces::of(None, None, self.places.grants(), self.places.pairings())
    }

    /// The person's folder.
    pub(crate) fn folder(&self) -> PathBuf {
        self.places.folder().unwrap().to_owned()
    }

    /// A file in the person's folder, by the name its keeper declares.
    pub(crate) fn kept(&self, file: &str) -> PathBuf {
        self.folder().join(file)
    }

    /// The person's `settings.toml`, where `alo-choosing` says it is.
    pub(crate) fn choosing(&self) -> PathBuf {
        self.places.choosing().unwrap().to_owned()
    }

    /// The machine's grants file.
    pub(crate) fn grants(&self) -> PathBuf {
        self.places.grants().to_owned()
    }

    /// The daemon's pairings file.
    pub(crate) fn pairings(&self) -> PathBuf {
        self.places.pairings().to_owned()
    }
}

/// A person's machine as the module header describes it.
pub(crate) fn a_persons_machine(what: &str) -> Machine {
    let held = tempfile::Builder::new()
        .prefix(&format!("alo-shell-settings-{what}-"))
        .tempdir()
        .unwrap();
    let home = held.path().join("home");
    let invoices = home.join("Invoices");
    std::fs::create_dir_all(&invoices).unwrap();
    let machine_files = held.path().join("var");
    std::fs::create_dir_all(&machine_files).unwrap();
    let places = SettingsPlaces::of(
        None,
        Some(home.as_os_str()),
        &machine_files.join("grants"),
        &machine_files.join("pairings"),
    );

    let mut grants = Grants::default();
    grants.grant(Grant::checked("@files", Reach::Folder(invoices), noon(), hour()).unwrap());
    alo_remembering::kept(places.grants(), &grants, noon()).unwrap();
    alo_remembering::pairings_kept(places.pairings(), &the_pairings(), noon()).unwrap();

    let mut choosing = Choosing::at(places.choosing().unwrap()).unwrap();
    choosing
        .adding(
            Provider::checked(PROVIDER, "https://harbour.example", Region::Unknown, None).unwrap(),
        )
        .unwrap();
    choosing
        .answered_by(Some(
            Picked::from_a_provider(PROVIDER, "harbour-small").unwrap(),
        ))
        .unwrap();

    Machine {
        _held: held,
        home,
        places,
    }
}

/// This machine's pairing with reception, made the way two machines make one:
/// reception may ask this machine's models, for a day from noon.
pub(crate) fn the_pairings() -> Pairings {
    let reception = MachineId::read(RECEPTION).unwrap();
    let studio = MachineId::read(STUDIO).unwrap();
    let proposal = Proposal::checked(
        reception,
        studio,
        &[MayAskIts::Models],
        Duration::from_secs(86_400),
        Keying::fresh().unwrap().offer().clone(),
    )
    .unwrap();
    let agreed = Deliberating::asked(proposal, Keying::fresh().unwrap())
        .agreed_at(Side::TheOneAsking)
        .agreed_at(Side::TheOneAsked)
        .agreed(noon())
        .unwrap();
    let mut pairings = Pairings::none();
    pairings.keep(agreed);
    pairings
}

/// A daemon's door that counts both conversations and answers a pairing's
/// revocation the way it is told to.
#[derive(Debug)]
pub(crate) struct CountingDoor {
    /// How many knocks arrived.
    pub(crate) knocks: Cell<u32>,
    /// Every machine it was asked to unpair.
    pub(crate) unpaired: RefCell<Vec<String>>,
    /// What it answers a pairing's revocation with.
    answer: Unpaired,
}

impl CountingDoor {
    /// A door whose daemon revokes a pairing and writes it down.
    pub(crate) fn revoking() -> Self {
        Self::answering(Unpaired::Revoked)
    }

    /// A door whose daemon answers a pairing's revocation with `answer`.
    pub(crate) fn answering(answer: Unpaired) -> Self {
        Self {
            knocks: Cell::new(0),
            unpaired: RefCell::new(Vec::new()),
            answer,
        }
    }
}

impl Knocking for CountingDoor {
    fn knock(&self) -> Stood {
        self.knocks.set(self.knocks.get() + 1);
        Stood::Heard { holding: 0 }
    }
}

impl RevokingPairings for CountingDoor {
    fn revoke_pairing(&self, with: &MachineId) -> Unpaired {
        self.unpaired.borrow_mut().push(with.as_str().to_owned());
        self.answer.clone()
    }
}

/// The doors a press reaches, at noon.
pub(crate) fn doors<'a>(
    strings: &'a alo_strings::Strings,
    daemon: &'a CountingDoor,
) -> SettingsDoors<'a> {
    SettingsDoors {
        strings,
        daemon,
        now: noon(),
    }
}

/// The bytes of a file, or [`None`] when there is none.
pub(crate) fn bytes(at: &Path) -> Option<Vec<u8>> {
    std::fs::read(at).ok()
}
