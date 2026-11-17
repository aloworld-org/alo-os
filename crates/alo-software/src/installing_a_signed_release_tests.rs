//! Installing a maker's signed release, and — mostly — every way it is refused.
//!
//! The fetcher, the verifier and the tool are stand-ins, so each refusal is the
//! one thing wrong. The test that runs the real gpgv against a real Craft
//! signature is `tests/a_signed_release_is_checked_by_gpgv.rs`.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_egress::Indicator;

use super::*;
use crate::bound::SetBy;
use crate::source::{Configured, SourceName};
use crate::tool::Failed;

const PINNED_KEY: &str = "2B76741B414AAD6CD33E631213A005C0B9D7C3ED";
const BUNDLE: &str = "https://github.com/storytold/photocraft/releases/download/v0.23.1/\
                      photocraft-0.23.1-linux-x86_64.flatpak";
const THE_BYTES: &[u8] = b"a bundle, standing in";

fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_794_484_800)
}

fn photocraft() -> Application {
    Application::identified("ai.storyteller.photocraft").unwrap()
}

/// The SHA-256 of [`THE_BYTES`], worked out here rather than copied.
fn the_checksum() -> String {
    use sha2::Digest as _;
    sha2::Sha256::digest(THE_BYTES)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn the_release() -> SignedRelease {
    SignedRelease::pinned(photocraft(), "storytold", BUNDLE, &the_checksum()).unwrap()
}

/// A network holding these files, by address, and nothing else.
struct Holding(BTreeMap<String, Vec<u8>>);

impl Holding {
    fn the_bundle_and_its_signature() -> Self {
        let mut files = BTreeMap::new();
        files.insert(BUNDLE.to_owned(), THE_BYTES.to_vec());
        files.insert(format!("{BUNDLE}.asc"), b"a signature".to_vec());
        Self(files)
    }

    fn without(mut self, address: &str) -> Self {
        self.0.remove(address);
        self
    }

    fn with(mut self, address: &str, bytes: &[u8]) -> Self {
        self.0.insert(address.to_owned(), bytes.to_vec());
        self
    }
}

impl Fetcher for Holding {
    fn fetch(&self, address: &str, into: &Path, _: u64) -> Result<(), Failed> {
        let bytes = self.0.get(address).ok_or_else(|| Failed::DidNotAnswer {
            said: format!("{address} answered 404"),
        })?;
        std::fs::write(into, bytes).unwrap();
        Ok(())
    }
}

/// A verifier that names this signer, and remembers what it was shown.
struct Naming {
    signer: Option<&'static str>,
    shown: RefCell<Vec<PathBuf>>,
}

impl Naming {
    fn the_pinned_key() -> Self {
        Self::signed_by(Some(PINNED_KEY))
    }

    fn signed_by(signer: Option<&'static str>) -> Self {
        Self {
            signer,
            shown: RefCell::new(Vec::new()),
        }
    }
}

impl Verifier for Naming {
    fn signer(&self, file: &Path, _: &Path, keyring: &Path) -> Result<Option<String>, Failed> {
        // The keyring is the pinned key, with its armour off, every time.
        assert_eq!(
            std::fs::read(keyring).unwrap(),
            Maker::named("storytold").unwrap().keyring().unwrap()
        );
        self.shown.borrow_mut().push(file.to_owned());
        Ok(self.signer.map(str::to_owned))
    }
}

/// A tool that records each file it is handed, and has these applications.
#[derive(Default)]
struct Recording {
    has: Vec<String>,
    handed: RefCell<Vec<(String, Vec<u8>)>>,
}

impl Tool for Recording {
    fn sources(&self) -> Result<Vec<Configured>, Failed> {
        Ok(Vec::new())
    }
    fn installed(&self) -> Result<Vec<String>, Failed> {
        Ok(self.has.clone())
    }
    fn open(&self) -> Result<Vec<String>, Failed> {
        Ok(Vec::new())
    }
    fn install(&self, _: &SourceName, _: &Application) -> Result<(), Failed> {
        Err(Failed::NotOffered)
    }
    fn updates(&self, _: &SourceName) -> Result<Vec<String>, Failed> {
        Ok(Vec::new())
    }
    fn update(&self, _: &Application) -> Result<(), Failed> {
        Err(Failed::NotInstalled)
    }
    fn remove(&self, _: &Application) -> Result<(), Failed> {
        Err(Failed::NotInstalled)
    }
    fn install_a_file(&self, application: &Application, file: &Path) -> Result<(), Failed> {
        self.handed.borrow_mut().push((
            application.identifier().to_owned(),
            std::fs::read(file).unwrap(),
        ));
        Ok(())
    }
}

/// Both steps, on an indicator, into a folder of its own.
fn the_whole_way(
    bound: &Bound,
    fetcher: &impl Fetcher,
    verifier: &impl Verifier,
    tool: &Recording,
) -> (Result<Installed, Stopped>, tempfile::TempDir) {
    let folder = tempfile::tempdir().unwrap();
    let mut indicator = Indicator::default();
    let errand = installing_a_signed_release(bound, &the_release(), THE_ARCHITECTURE, tool)
        .map_err(Stopped::from);
    let answer = errand.and_then(|errand| {
        let underway = indicator.beginning_on_its_own(errand, noon());
        let answer = install_a_signed_release(
            &underway,
            bound,
            &the_release(),
            THE_ARCHITECTURE,
            fetcher,
            verifier,
            tool,
            folder.path(),
        );
        indicator.ended_on_its_own(underway);
        answer
    });
    (answer, folder)
}

fn not_shown() -> Stopped {
    Stopped::Refused(NotDone::SignatureNotShown {
        source: "storytold".to_owned(),
        application: "ai.storyteller.photocraft".to_owned(),
    })
}

/// **A release whose file hashes to its pin and whose signature names the
/// pinned key is installed**, from exactly the bytes that were checked, and
/// nothing fetched is kept.
#[test]
fn a_release_shown_to_come_from_its_maker_is_installed() {
    let tool = Recording::default();
    let verifier = Naming::the_pinned_key();
    let (answer, folder) = the_whole_way(
        &Bound::Nobodys,
        &Holding::the_bundle_and_its_signature(),
        &verifier,
        &tool,
    );
    let installed = answer.unwrap();
    assert_eq!(installed.application(), &photocraft());
    assert_eq!(
        tool.handed.borrow().as_slice(),
        &[("ai.storyteller.photocraft".to_owned(), THE_BYTES.to_vec())]
    );
    assert_eq!(
        verifier.shown.borrow().len(),
        1,
        "the signature was checked once"
    );
    assert_eq!(
        std::fs::read_dir(folder.path()).unwrap().count(),
        0,
        "something fetched was kept"
    );
}

/// **A missing signature is refused**, and the file is never installed on the
/// strength of its checksum alone.
#[test]
fn a_missing_signature_is_refused() {
    let tool = Recording::default();
    let verifier = Naming::the_pinned_key();
    let (answer, folder) = the_whole_way(
        &Bound::Nobodys,
        &Holding::the_bundle_and_its_signature().without(&format!("{BUNDLE}.asc")),
        &verifier,
        &tool,
    );
    assert_eq!(answer.unwrap_err(), not_shown());
    assert!(tool.handed.borrow().is_empty(), "it was installed");
    assert!(verifier.shown.borrow().is_empty());
    assert_eq!(std::fs::read_dir(folder.path()).unwrap().count(), 0);
}

/// **A bad signature is refused** — the verifier names nobody.
#[test]
fn a_bad_signature_is_refused() {
    let tool = Recording::default();
    let (answer, folder) = the_whole_way(
        &Bound::Nobodys,
        &Holding::the_bundle_and_its_signature(),
        &Naming::signed_by(None),
        &tool,
    );
    assert_eq!(answer.unwrap_err(), not_shown());
    assert!(tool.handed.borrow().is_empty(), "it was installed");
    assert_eq!(std::fs::read_dir(folder.path()).unwrap().count(), 0);
}

/// **A good signature by another key is refused**, so a keyring swapped for
/// another maker's, or a release re-signed by somebody else, installs nothing.
#[test]
fn a_signature_by_another_key_is_refused() {
    let tool = Recording::default();
    let (answer, _folder) = the_whole_way(
        &Bound::Nobodys,
        &Holding::the_bundle_and_its_signature(),
        &Naming::signed_by(Some("0000000000000000000000000000000000000000")),
        &tool,
    );
    assert_eq!(answer.unwrap_err(), not_shown());
    assert!(tool.handed.borrow().is_empty(), "it was installed");
}

/// **A file that does not hash to its pin is refused before its signature is
/// read**, even with a signature that would have passed.
#[test]
fn a_file_that_does_not_hash_to_its_pin_is_refused() {
    let tool = Recording::default();
    let verifier = Naming::the_pinned_key();
    let (answer, folder) = the_whole_way(
        &Bound::Nobodys,
        &Holding::the_bundle_and_its_signature().with(BUNDLE, b"a bundle, replaced"),
        &verifier,
        &tool,
    );
    assert_eq!(answer.unwrap_err(), not_shown());
    assert!(tool.handed.borrow().is_empty(), "it was installed");
    assert!(verifier.shown.borrow().is_empty(), "its signature was read");
    assert_eq!(std::fs::read_dir(folder.path()).unwrap().count(), 0);
}

/// **A file that could not be fetched installs nothing**, and says the
/// machine did not answer rather than that the maker's file was bad.
#[test]
fn a_file_that_could_not_be_fetched_installs_nothing() {
    let tool = Recording::default();
    let (answer, _folder) = the_whole_way(
        &Bound::Nobodys,
        &Holding::the_bundle_and_its_signature().without(BUNDLE),
        &Naming::the_pinned_key(),
        &tool,
    );
    assert!(
        matches!(answer, Err(Stopped::Refused(NotDone::DidNotAnswer { .. }))),
        "{answer:?}"
    );
    assert!(tool.handed.borrow().is_empty());
}

/// **An organisation's rule reaches a maker as it reaches any place** —
/// refused before anything leaves.
#[test]
fn a_rule_that_keeps_the_maker_out_is_kept_before_anything_leaves() {
    let tool = Recording::default();
    let only_flathub = Bound::only(
        [SourceName::checked("flathub").unwrap()],
        SetBy::AnAdministrator,
    );
    let refused =
        installing_a_signed_release(&only_flathub, &the_release(), THE_ARCHITECTURE, &tool)
            .unwrap_err();
    assert_eq!(
        refused,
        NotDone::OutsideTheBound {
            source: "storytold".to_owned(),
            set_by: SetBy::AnAdministrator
        }
    );
}

/// **Already here, and another architecture, are refused before anything
/// leaves.**
#[test]
fn already_here_and_another_architecture_are_refused_first() {
    let has_it = Recording {
        has: vec!["ai.storyteller.photocraft".to_owned()],
        ..Recording::default()
    };
    assert_eq!(
        installing_a_signed_release(&Bound::Nobodys, &the_release(), THE_ARCHITECTURE, &has_it)
            .unwrap_err(),
        NotDone::AlreadyInstalled {
            application: "ai.storyteller.photocraft".to_owned()
        }
    );
    assert!(matches!(
        installing_a_signed_release(
            &Bound::Nobodys,
            &the_release(),
            "aarch64",
            &Recording::default()
        ),
        Err(NotDone::NotOffered { .. })
    ));
}

/// The errand shown is an installation, to the maker's release host.
#[test]
fn the_errand_is_an_installation_to_the_makers_host() {
    let errand = installing_a_signed_release(
        &Bound::Nobodys,
        &the_release(),
        THE_ARCHITECTURE,
        &Recording::default(),
    )
    .unwrap();
    assert_eq!(errand.errand(), Errand::InstallingAnApplication);
    assert_eq!(
        errand.destination(),
        &Destination::at("github.com").unwrap()
    );
}

/// **A tool that does not say it installs files installs none** — the default
/// every older tool and stand-in gets.
#[test]
fn a_tool_that_does_not_say_it_installs_files_refuses_one() {
    struct Older;
    impl Tool for Older {
        fn sources(&self) -> Result<Vec<Configured>, Failed> {
            Ok(Vec::new())
        }
        fn installed(&self) -> Result<Vec<String>, Failed> {
            Ok(Vec::new())
        }
        fn open(&self) -> Result<Vec<String>, Failed> {
            Ok(Vec::new())
        }
        fn install(&self, _: &SourceName, _: &Application) -> Result<(), Failed> {
            Ok(())
        }
        fn updates(&self, _: &SourceName) -> Result<Vec<String>, Failed> {
            Ok(Vec::new())
        }
        fn update(&self, _: &Application) -> Result<(), Failed> {
            Ok(())
        }
        fn remove(&self, _: &Application) -> Result<(), Failed> {
            Ok(())
        }
    }
    assert!(matches!(
        Older.install_a_file(&photocraft(), Path::new("/x.flatpak")),
        Err(Failed::DidNotAnswer { .. })
    ));
}

/// **What a pinned release may not be**: another maker, another host, and a
/// checksum that is not one.
#[test]
fn what_a_pinned_release_may_not_be() {
    let sum = the_checksum();
    assert_eq!(
        SignedRelease::pinned(photocraft(), "someone", BUNDLE, &sum).unwrap_err(),
        NotASignedRelease::NoSuchMaker
    );
    assert_eq!(
        SignedRelease::pinned(
            photocraft(),
            "storytold",
            "https://example.org/storytold/photocraft/releases/download/v1/a.flatpak",
            &sum
        )
        .unwrap_err(),
        NotASignedRelease::NotTheMakers
    );
    for no in ["", "abc", &sum[1..], &format!("{}g", &sum[1..])] {
        assert_eq!(
            SignedRelease::pinned(photocraft(), "storytold", BUNDLE, no).unwrap_err(),
            NotASignedRelease::NotAChecksum,
            "{no:?}"
        );
    }
    assert_eq!(the_release().signature(), format!("{BUNDLE}.asc"));
}
