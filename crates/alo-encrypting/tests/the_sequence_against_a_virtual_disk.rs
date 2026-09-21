//! The enrolment sequence, run against a real LUKS2 volume rather than argued
//! about.
//!
//! This is the acceptance
//! [ADR 0056](../../../docs/decisions/0056-a-sealed-disks-promise-is-shown-on-a-machine-with-a-chip.md)
//! decided, option C: **everything on the road that is about LUKS runs here, on
//! a virtual disk, and the three facts that are about a chip are shown on a
//! machine with a chip and nowhere else** (the plan's task 9). What runs is the
//! sequence `alo-encrypting` hands the installer — the same [`TheSequence`] the
//! installer plan is given, argument for argument — in the base alo OS ships,
//! under `podman`.
//!
//! # What it shows, and each of these is one of ADR 0054's own claims
//!
//! 1. the six steps happen in [`alo_encrypting::THE_ROAD`]'s order, and the
//!    recovery key is made, shown and typed back before anything the person will
//!    unlock with exists;
//! 2. the recovery key the rented tool really printed is the shape this crate
//!    reads, and a person typing it back is what makes the [`Enrolment`];
//! 3. the installer's own first key opens nothing once the install has finished;
//! 4. the person's secret opens the volume, **and so does the recovery key**;
//! 5. a secret that is not this disk's opens nothing, and neither does a near
//!    miss;
//! 6. changing what the person unlocks with leaves the new secret opening it and
//!    the old one opening nothing;
//! 7. the recovery key is not in the bytes of the disk it recovers.
//!
//! The passphrase road — ADR 0054's option B, which is what every machine with
//! no usable chip gets — is therefore shown **end to end**. Of the chip road's
//! six runs, five are these same runs; the sixth is the one this cannot run, and
//! saying so is the point of the split rather than a gap in it.
//!
//! # What it needs, and why it is not in the workspace suite
//!
//! `podman`, root, loop devices, and the pinned base pulled. It is `#[ignore]`d
//! and run by name, the way every other test in this repository that needs a
//! container is, and it **fails** rather than skips when something it needs is
//! missing — a test that quietly passes on a machine that could not run it is
//! the thing this whole decision was written to avoid.
//!
//! ```text
//! cargo test -p alo-encrypting --test the_sequence_against_a_virtual_disk -- --ignored --nocapture
//! ```
//!
//! # What is not claimed here
//!
//! Nothing about a chip, and nothing about a machine. `docs/features.md`'s v0.5
//! *Full-disk encryption, enrolled at install* is **not** ticked by this test
//! passing: an install that has never happened on a machine with a chip has
//! enrolled nothing, and ADR 0056 point 2 is that said out loud rather than left
//! to a reader of a green suite.

#![cfg(target_os = "linux")]
#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::Command;

use alo_encrypting::{
    ASecretOnItsWay, Enrolment, HowItUnlocks, ONLY_ITS_OWNER_MAY_READ_IT, Passphrase, RecoveryKey,
    Run, THE_ONE_PLACE, TheChip, TheDisk, TheDiskRefused, TheSequence, TheVolume, WhatToAskFor,
};

/// The base alo OS is built on, exactly as `image/Containerfile` pins it.
const THE_BASE: &str = "quay.io/fedora/fedora-bootc:42@sha256:077182b6ba853b3348d0bede602ac30b9e6568c6422bcf0654de5af96f19b9c3";

/// The disk the virtual one answers to, by the identity udev would have given
/// it.
const THE_DISK: &str = "virtio-alo-target";

/// Which partition of it holds the encrypted volume.
const THE_PARTITION: u8 = 4;

/// How big the virtual disk is. LUKS2's header wants sixteen mebibytes of
/// keyslot area and a little more; sixty-four is comfortable and still made in
/// no time.
const HOW_BIG: u64 = 64 * 1024 * 1024;

/// What the installer's own first key is: bytes nobody is shown, the length of
/// a volume key.
const THE_INSTALLERS_FIRST_KEY: &[u8] =
    b"an installer key no person is ever shown 0123456789abcdef";

/// What this person opens their machine with.
const THE_PERSONS_SECRET: &str = "a quiet harbour in november";

/// What they change it to.
const THE_PERSONS_NEW_SECRET: &str = "a louder harbour in december";

/// A secret that was never this disk's.
const A_STRANGERS_SECRET: &str = "this was some other machine's secret";

/// How a rented tool ended.
struct Ended {
    /// What it answered, or `None` if a signal stopped it.
    code: Option<i32>,
    /// What it wrote to standard output — a secret on exactly one run of the
    /// road, and nothing of the kind on any other.
    printed: String,
    /// What it wrote to standard error, which is where both tools put their
    /// English.
    said: String,
}

impl Ended {
    /// What the disk refused, if anything.
    fn refusal(&self) -> Option<TheDiskRefused> {
        TheDiskRefused::from_how_it_ended(self.code)
    }
}

/// A virtual disk, with the directories the sequence expects around it.
struct AVirtualDisk {
    /// Where its working directory is on the host.
    at: PathBuf,
}

impl AVirtualDisk {
    /// A fresh one, named so that two tests do not share a disk.
    fn made(named: &str) -> Self {
        let at = std::env::temp_dir()
            .join("alo-encrypting-acceptance")
            .join(named);
        if at.exists()
            && let Err(why) = fs::remove_dir_all(&at)
        {
            panic!(
                "the last run's {} could not be removed: {why}",
                at.display()
            );
        }
        for under in ["by-id", "secrets"] {
            if let Err(why) = fs::create_dir_all(at.join(under)) {
                panic!("{} could not be made: {why}", at.join(under).display());
            }
        }
        let image = at.join("disk.img");
        match fs::File::create(&image).and_then(|file| file.set_len(HOW_BIG)) {
            Ok(()) => {}
            Err(why) => panic!("{} could not be made: {why}", image.display()),
        }
        let disk = Self { at };
        let named_the_way_it_stays_named = disk
            .at
            .join("by-id")
            .join(format!("{THE_DISK}-part{THE_PARTITION}"));
        // Inside the container the image is at `/w/disk.img`, which is what the
        // symlink in the mounted `by-id` directory has to point at.
        if let Err(why) = symlink("/w/disk.img", &named_the_way_it_stays_named) {
            panic!(
                "{} could not be named: {why}",
                named_the_way_it_stays_named.display()
            );
        }
        disk
    }

    /// The volume, as the rest of alo OS names it.
    fn volume(&self) -> TheVolume {
        let Ok(disk) = TheDisk::named(THE_DISK) else {
            panic!("{THE_DISK} is a disk's own name")
        };
        match TheVolume::the_partition_of(&disk, THE_PARTITION) {
            Ok(volume) => volume,
            Err(why) => panic!("{THE_PARTITION} is a partition: {why}"),
        }
    }

    /// Put a secret in the file the sequence will name, readable by nobody but
    /// its owner.
    fn put(&self, secret: ASecretOnItsWay, bytes: &[u8]) {
        let at = self.where_the_secret_is(secret);
        if let Err(why) = fs::write(&at, bytes) {
            panic!("{} could not be written: {why}", at.display());
        }
        if let Err(why) =
            fs::set_permissions(&at, fs::Permissions::from_mode(ONLY_ITS_OWNER_MAY_READ_IT))
        {
            panic!(
                "{} could not be made the owner's alone: {why}",
                at.display()
            );
        }
    }

    /// Where on the host a named secret's file is.
    fn where_the_secret_is(&self, secret: ASecretOnItsWay) -> PathBuf {
        let Some(named) = secret
            .where_it_is()
            .strip_prefix(&format!("{THE_ONE_PLACE}/"))
        else {
            panic!("{secret} is not in the one place")
        };
        self.at.join("secrets").join(named)
    }

    /// The disk's own bytes, as anybody who took the disk would read them.
    fn bytes(&self) -> Vec<u8> {
        match fs::read(self.at.join("disk.img")) {
            Ok(bytes) => bytes,
            Err(why) => panic!("the virtual disk could not be read: {why}"),
        }
    }

    /// One run of one rented tool, in the pinned base.
    fn run(&self, run: &Run) -> Ended {
        let at = self.at.display();
        let mut asking = Command::new("podman");
        asking
            .arg("run")
            .arg("--rm")
            .arg("--privileged")
            .arg("-v")
            .arg(format!("{at}:/w"))
            .arg("-v")
            .arg(format!("{at}/by-id:/dev/disk/by-id"))
            .arg("-v")
            .arg(format!("{at}/secrets:{THE_ONE_PLACE}"))
            .arg(THE_BASE)
            .arg(run.tool().where_it_is())
            .args(run.arguments());
        let answered = match asking.output() {
            Ok(answered) => answered,
            Err(why) => panic!(
                "podman could not be run, and this test fails rather than skipping on a machine \
                 that cannot run it: {why}"
            ),
        };
        Ended {
            code: answered.status.code(),
            printed: String::from_utf8_lossy(&answered.stdout).into_owned(),
            said: String::from_utf8_lossy(&answered.stderr).into_owned(),
        }
    }

    /// Every run of a sequence, in order, and each of them has to finish.
    fn walk(&self, sequence: &TheSequence) -> Vec<Ended> {
        sequence
            .runs()
            .iter()
            .map(|run| {
                let ended = self.run(run);
                if let Some(refused) = ended.refusal() {
                    panic!(
                        "{:?} was refused ({refused}): {}",
                        run.at(),
                        ended.said.trim()
                    );
                }
                ended
            })
            .collect()
    }
}

/// Enrol the passphrase road on a fresh virtual disk, and hand back the disk,
/// the recovery key as the tool printed it, and the enrolment the person's
/// typing made.
fn enrolled(named: &str) -> (AVirtualDisk, String, Enrolment) {
    let disk = AVirtualDisk::made(named);
    disk.put(
        ASecretOnItsWay::TheInstallersFirstKey,
        THE_INSTALLERS_FIRST_KEY,
    );
    disk.put(
        ASecretOnItsWay::ThePersonsSecret,
        THE_PERSONS_SECRET.as_bytes(),
    );

    let volume = disk.volume();
    let sequence = TheSequence::enrolling_at_install(&volume, WhatToAskFor::APassphrase);
    let ended = disk.walk(&sequence);

    // The one run whose standard output is a secret, and nothing else prints
    // anything of the kind.
    let mut printed = None;
    for (run, ended) in sequence.runs().iter().zip(&ended) {
        if run.what_it_prints_is_a_secret() {
            printed = Some(ended.printed.clone());
        } else {
            assert!(
                ended.printed.trim().is_empty(),
                "{:?} printed {}",
                run.at(),
                ended.printed
            );
        }
    }
    let Some(printed) = printed else {
        panic!("no run of the enrolment made the recovery key")
    };

    // The person is shown it, copies it onto paper, and types it back.
    let key = match RecoveryKey::as_printed(&printed) {
        Ok(key) => key,
        Err(why) => panic!("what the rented tool printed was not a recovery key: {why}"),
    };
    let on_the_screen = key.as_it_is_shown().to_owned();
    let kept = match key.written_back(&on_the_screen) {
        Ok(kept) => kept,
        Err(again) => panic!("the key typed back is the key: {:?}", again.why()),
    };
    let passphrase = match Passphrase::typed(THE_PERSONS_SECRET, THE_PERSONS_SECRET) {
        Ok(passphrase) => passphrase,
        Err(why) => panic!("that is a passphrase: {why}"),
    };
    disk.put(ASecretOnItsWay::TheRecoveryKey, on_the_screen.as_bytes());
    (
        disk,
        on_the_screen,
        Enrolment::made(HowItUnlocks::a_passphrase(passphrase), kept),
    )
}

/// Open the volume with a named secret, and close it again if it opened.
fn opening_with(disk: &AVirtualDisk, secret: ASecretOnItsWay) -> Option<TheDiskRefused> {
    let volume = disk.volume();
    let opening = TheSequence::opening(&volume, secret);
    let Some(run) = opening.runs().first() else {
        panic!("opening is one run")
    };
    let ended = disk.run(run);
    if ended.refusal().is_none() {
        let closing = TheSequence::closing();
        let Some(run) = closing.runs().first() else {
            panic!("closing is one run")
        };
        let ended = disk.run(run);
        if let Some(refused) = ended.refusal() {
            panic!(
                "the volume would not close ({refused}): {}",
                ended.said.trim()
            );
        }
    }
    ended.refusal()
}

/// **The whole road, against a real LUKS2 volume**: enrolled, opened by the
/// person, opened by the recovery key, opened by nothing else, and its secret
/// changed.
#[test]
#[ignore = "runs the pinned base under podman against a real LUKS2 volume; run by name"]
fn the_whole_sequence_runs_against_a_real_virtual_disk() {
    let (disk, key, enrolment) = enrolled("the-whole-sequence");

    // 2. The key the tool really printed is the shape this crate reads, and the
    //    enrolment could not have been written without the person's typing.
    assert_eq!(key.len(), 71, "{key}");
    assert!(!enrolment.how_it_unlocks().is_sealed_to_the_chip());
    assert_eq!(
        WhatToAskFor::on_a_machine_with(TheChip::Absent),
        WhatToAskFor::APassphrase
    );

    // 4. The person's secret opens it, and so does the recovery key.
    assert_eq!(
        opening_with(&disk, ASecretOnItsWay::ThePersonsSecret),
        None,
        "the person's own secret did not open their disk"
    );
    assert_eq!(
        opening_with(&disk, ASecretOnItsWay::TheRecoveryKey),
        None,
        "the recovery key did not recover the disk"
    );

    // 3. The installer's own first key opens nothing now the install is over.
    assert_eq!(
        opening_with(&disk, ASecretOnItsWay::TheInstallersFirstKey),
        Some(TheDiskRefused::WhatWasGivenDoesNotOpenIt)
    );

    // 5. A secret that was never this disk's opens nothing, and neither does a
    //    near miss — one character of the recovery key changed.
    disk.put(
        ASecretOnItsWay::ThePersonsNewSecret,
        A_STRANGERS_SECRET.as_bytes(),
    );
    assert_eq!(
        opening_with(&disk, ASecretOnItsWay::ThePersonsNewSecret),
        Some(TheDiskRefused::WhatWasGivenDoesNotOpenIt)
    );
    let nearly = a_near_miss(&key);
    disk.put(ASecretOnItsWay::ThePersonsNewSecret, nearly.as_bytes());
    assert_eq!(
        opening_with(&disk, ASecretOnItsWay::ThePersonsNewSecret),
        Some(TheDiskRefused::WhatWasGivenDoesNotOpenIt),
        "a recovery key one character wrong opened the disk"
    );

    // 6. Changing what the person unlocks with: the new one opens it, the old
    //    one stops opening it, and the recovery key still recovers it.
    disk.put(
        ASecretOnItsWay::ThePersonsNewSecret,
        THE_PERSONS_NEW_SECRET.as_bytes(),
    );
    let volume = disk.volume();
    disk.walk(&TheSequence::changing_what_the_person_unlocks_with(
        &volume,
        WhatToAskFor::APassphrase,
    ));
    assert_eq!(
        opening_with(&disk, ASecretOnItsWay::ThePersonsNewSecret),
        None,
        "the secret they changed it to did not open it"
    );
    assert_eq!(
        opening_with(&disk, ASecretOnItsWay::ThePersonsSecret),
        Some(TheDiskRefused::WhatWasGivenDoesNotOpenIt),
        "the secret they changed away from still opened it"
    );
    assert_eq!(
        opening_with(&disk, ASecretOnItsWay::TheRecoveryKey),
        None,
        "changing the secret took the recovery key away with it"
    );
}

/// **The recovery key is not on the disk it recovers**, read off the disk's own
/// bytes the way somebody who took the disk would read them.
#[test]
#[ignore = "runs the pinned base under podman against a real LUKS2 volume; run by name"]
fn the_recovery_key_is_not_on_the_disk_it_recovers() {
    let (disk, key, _) = enrolled("the-key-is-not-on-the-disk");
    let bytes = disk.bytes();
    assert_eq!(bytes.len() as u64, HOW_BIG);
    for looked_for in [
        key.clone(),
        key.replace('-', ""),
        THE_PERSONS_SECRET.to_owned(),
    ] {
        assert!(
            !contains(&bytes, looked_for.as_bytes()),
            "a secret is in the bytes of the disk it opens"
        );
    }
    // And the thing that is on the disk — the LUKS2 header — is there, so that
    // the test above is not passing because it read the wrong file.
    assert!(contains(&bytes, b"LUKS"), "that is not a LUKS volume");
}

/// The recovery key with one character changed to another of the sixteen.
fn a_near_miss(key: &str) -> String {
    let changed = if key.starts_with('c') { 'b' } else { 'c' };
    let mut letters: Vec<char> = key.chars().collect();
    match letters.first_mut() {
        Some(first) => *first = changed,
        None => panic!("the recovery key is not empty"),
    }
    letters.into_iter().collect()
}

/// Whether a run of bytes is anywhere in another.
fn contains(inside: &[u8], looked_for: &[u8]) -> bool {
    if looked_for.is_empty() || looked_for.len() > inside.len() {
        return false;
    }
    inside
        .windows(looked_for.len())
        .any(|window| window == looked_for)
}

/// The base is pinned here exactly as the image pins it, so that this runs in
/// the base alo OS ships rather than in whatever was newest today.
#[test]
fn the_base_this_runs_in_is_the_base_the_image_pins() {
    let containerfile = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../image/Containerfile");
    let text = fs::read_to_string(&containerfile).unwrap_or_default();
    assert!(
        !text.is_empty(),
        "{} could not be read",
        containerfile.display()
    );
    assert!(
        text.contains(THE_BASE),
        "this test pins a base the image does not"
    );
}
