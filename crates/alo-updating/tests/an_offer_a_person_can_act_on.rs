//! An offer a person can act on — task 7 of
//! `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`, one criterion at a
//! time.
//!
//! | Criterion | Test |
//! |---|---|
//! | the offer carries that it has not been vouched for, and says so **before** the person chooses | [`an_update_nothing_vouches_for_says_so_before_the_person_chooses`] |
//! | a build nothing vouches for is still offered rather than hidden | [`a_build_nothing_vouches_for_is_told_about_rather_than_hidden`] |
//! | the signature policy's refusal is its own sentence, not *it could not be prepared* | [`a_build_the_signature_policy_refuses_is_its_own_sentence`] |
//! | every refusal added is reachable and is in the vocabulary with a note | [`every_refusal_added_is_reachable_and_in_the_vocabulary_with_a_note`] |
//! | nothing weakens `--enforce-container-sigpolicy`, whatever is vouched for | [`nothing_about_an_unvouched_offer_weakens_the_signature_policy`] |
//! | what a machine is offered today, and what its base does with it | [`what_the_real_base_says_about_the_real_places_builds`] (`#[ignore]`d), and `alo-looking`'s `what_this_machine_is_offered_today_and_what_it_is_told_about_it` |
//!
//! # The decision this task made, and the one it discarded
//!
//! The plan left two ways open and asked for the reason in writing.
//!
//! **Chosen: the offer carries the doubt.** A build nothing vouches for is
//! still offered — a newer version that exists is told about — and the
//! sentence a person reads says the machine cannot confirm it came from alo
//! OS. Acting on it is not blocked here; the machine's signature policy is
//! still the only authority, and when it refuses, the refusal is its own
//! sentence rather than the one a stalled download reads as.
//!
//! **Discarded: narrowing the offer to what the place vouches for.** It reads
//! safer and is worse. A machine running an old release would be told *this
//! machine is up to date* while a newer one sat at the place, which is the one
//! thing task 1 of this plan forbids: *the choice is when, never whether to be
//! told*. It would also hide the window between a release being pushed and
//! being signed from the only people who could close it, and a fleet quietly
//! sitting on an old build because a signature went missing is a worse failure
//! than a cautious sentence.
//!
//! What is **not** done either way: nothing here decides that a build may be
//! staged. Vouching is read from a name at a place; it verifies nothing.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::time::{Duration, SystemTime};

use alo_egress::{Destination, Indicator};
use alo_keeping_up::{
    Digest, Offered, Ready, Running, Source, Standing, Vouching, WhenItApplies, a_check_at, words,
};
use alo_saying::everything_this_machine_can_say;
use alo_strings::{Filling, Said, Strings};
use alo_updating::{Base, NotAnswered, NotApplied, apply, refused_for_its_signature};

/// What the base printed on 2026-09-19 when its signature policy refused a
/// build of alo OS — see the `#[ignore]`d measurement at the bottom of this
/// file, and `crates/alo-updating/src/genuine.rs`.
const WHAT_THE_BASE_SAID: &str = "time=\"2026-09-19T22:53:00Z\" level=fatal msg=\"Source image \
                                  rejected: A signature was required, but no signature exists\"";

/// A stand-in for the base that stages nothing and says why in the base's own
/// words.
struct ABaseThatRefuses {
    /// What it says when told to switch.
    said: String,
    /// Every question it was asked.
    asked: RefCell<Vec<Vec<String>>>,
}

impl ABaseThatRefuses {
    fn saying(said: &str) -> Self {
        Self {
            said: said.to_owned(),
            asked: RefCell::new(Vec::new()),
        }
    }

    /// Everything it was told to do, other than being asked its status.
    fn told_to_do(&self) -> Vec<Vec<String>> {
        self.asked
            .borrow()
            .iter()
            .filter(|asked| asked.first().is_some_and(|verb| verb != "status"))
            .cloned()
            .collect()
    }
}

impl Base for ABaseThatRefuses {
    fn asked(&self, arguments: &[String]) -> Result<Vec<u8>, NotAnswered> {
        self.asked.borrow_mut().push(arguments.to_vec());
        match arguments.first().map(String::as_str) {
            Some("status") => Ok(a_status(&build("aa")).into_bytes()),
            _ => Err(NotAnswered::SaidNo {
                program: "bootc".to_owned(),
                code: Some(1),
                said: self.said.clone(),
            }),
        }
    }
}

/// The base's status document, with one build booted and nothing waiting.
fn a_status(booted: &str) -> String {
    format!(
        r#"{{"apiVersion":"org.containers.bootc/v1","kind":"BootcHost","status":{{"booted":{{"image":{{"image":{{"image":"ghcr.io/aloworld-org/alo-os","transport":"registry"}},"imageDigest":"{booted}"}},"pinned":false}},"staged":null,"rollback":null}}}}"#
    )
}

fn build(pair: &str) -> String {
    format!("sha256:{}", pair.repeat(32))
}

fn digest(pair: &str) -> Digest {
    Digest::read(&build(pair)).unwrap()
}

fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * 12)
}

fn the_source() -> Source {
    Source::named("ghcr.io/aloworld-org/alo-os").unwrap()
}

fn the_machine() -> Strings {
    Strings::of(everything_this_machine_can_say().unwrap())
}

/// One word, said by this machine with nothing put into it.
fn plainly(word: alo_strings::Word, strings: &Strings) -> Said {
    strings.say(&word.key(), &Filling::nothing())
}

/// Where a machine stands, found the only way an offer can be: heard during a
/// check that was on the indicator.
fn standing(running: &str, offered: &str, vouching: Vouching) -> Standing {
    let mut indicator = Indicator::default();
    let underway =
        indicator.beginning_on_its_own(a_check_at(Destination::at("ghcr.io").unwrap()), noon());
    let offer = Offered::heard(&underway, digest(offered), vouching).unwrap();
    assert!(indicator.ended_on_its_own(underway));
    Standing::between(&Running::reported(digest(running)), &offer)
}

/// The update itself, for a test that wants to act on one.
fn ready(running: &str, offered: &str, vouching: Vouching) -> Ready {
    match standing(running, offered, vouching) {
        Standing::Ready(ready) => ready,
        Standing::UpToDate => panic!("{running} and {offered} are the same build"),
    }
}

/// **An update nothing vouches for says so before the person chooses.**
///
/// The sentence is the offer's own — it is what a surface shows *instead of*
/// an update being ready — so a person meets it while the choice is still in
/// front of them, rather than as a refusal after they made it.
#[test]
fn an_update_nothing_vouches_for_says_so_before_the_person_chooses() {
    let strings = the_machine();

    let vouched = standing("aa", "bb", Vouching::ThePlaceVouchesForIt).said(&strings);
    let not = standing("aa", "bb", Vouching::NobodyHasVouchedForIt).said(&strings);

    assert!(!not.is_a_bug(), "{not}");
    assert!(not.unfilled().is_empty(), "{not}");
    assert_ne!(vouched.text(), not.text(), "both offers read the same");
    assert_eq!(
        not.text(),
        "A new version of this machine's system is available, but this machine cannot confirm \
         that it came from alo OS. It will not be applied unless it can be confirmed"
    );

    // It is the *offer's* sentence and not a refusal's: nothing has been
    // chosen, tried or refused at the moment a person reads it.
    assert!(
        standing("aa", "bb", Vouching::NobodyHasVouchedForIt).is_ready(),
        "an offer nothing vouches for stopped being an offer"
    );

    // And it does not name the machinery a person never learns.
    for machinery in [
        "signature",
        "registry",
        "digest",
        "sha256",
        "image",
        "bootc",
    ] {
        assert!(
            !not.text().to_lowercase().contains(machinery),
            "{not} says \"{machinery}\""
        );
    }
}

/// **A build nothing vouches for is told about rather than hidden.**
///
/// The discarded alternative, as a test: narrowing the offer would leave this
/// machine saying *up to date* while a newer version sat at the place.
#[test]
fn a_build_nothing_vouches_for_is_told_about_rather_than_hidden() {
    let strings = the_machine();
    let standing = standing("aa", "bb", Vouching::NobodyHasVouchedForIt);

    assert!(standing.is_ready());
    assert_ne!(
        standing.said(&strings).text(),
        plainly(words::UP_TO_DATE, &strings).text(),
        "a version nothing vouches for was hidden behind *up to date*"
    );
    match &standing {
        Standing::Ready(ready) => {
            assert_eq!(ready.offered(), &digest("bb"));
            assert_eq!(ready.vouching(), Vouching::NobodyHasVouchedForIt);
        }
        Standing::UpToDate => panic!("a newer build was not offered"),
    }
}

/// **A build the signature policy refuses is its own sentence**, and the
/// machine is as it was.
///
/// Until this task both of these read as *the update could not be prepared*,
/// which is true of a download that stopped and true of a build the machine
/// would not trust, and tells a person neither.
#[test]
fn a_build_the_signature_policy_refuses_is_its_own_sentence() {
    let strings = the_machine();

    let base = ABaseThatRefuses::saying(WHAT_THE_BASE_SAID);
    let refused = apply(
        &base,
        &ready("aa", "bb", Vouching::NobodyHasVouchedForIt),
        &the_source(),
        WhenItApplies::AtTheNextRestart,
    )
    .unwrap_err();

    assert!(
        matches!(refused, NotApplied::TheBuildWasNotGenuine(_)),
        "{refused:?}"
    );
    let said = refused.said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert_eq!(
        said.text(),
        "This machine could not confirm that the new version came from alo OS, so it was not \
         installed and nothing was changed. The next restart starts this machine as it is now"
    );
    assert_ne!(
        said.text(),
        plainly(words::NOT_PREPARED, &strings).text(),
        "the two refusals still read alike"
    );
    assert!(said.text().contains("nothing was changed"), "{said}");

    // **The machine is as it was**: the base was told to switch once, and
    // nothing else was asked of it afterwards.
    assert_eq!(base.told_to_do().len(), 1);
    assert_eq!(
        base.told_to_do().first().unwrap().first().unwrap(),
        "switch"
    );

    // And a failure that is not about a signature keeps the sentence it had.
    let stalled = ABaseThatRefuses::saying("error: Pulling: reading blob: unexpected EOF");
    let refused = apply(
        &stalled,
        &ready("aa", "bb", Vouching::ThePlaceVouchesForIt),
        &the_source(),
        WhenItApplies::AtTheNextRestart,
    )
    .unwrap_err();
    assert!(
        matches!(refused, NotApplied::TheBaseDidNotStageIt(_)),
        "{refused:?}"
    );
    assert_eq!(
        refused.said(&strings).text(),
        plainly(words::NOT_PREPARED, &strings).text()
    );
}

/// **Every refusal added is reachable from a test and is in the machine's
/// vocabulary with a translator's note.**
#[test]
fn every_refusal_added_is_reachable_and_in_the_vocabulary_with_a_note() {
    let vocabulary = everything_this_machine_can_say().unwrap();
    let strings = Strings::of(vocabulary.clone());

    for word in [words::READY_NOT_VOUCHED_FOR, words::NOT_GENUINE] {
        let phrase = vocabulary
            .phrase(&word.key())
            .unwrap_or_else(|| panic!("{} is not in the machine's vocabulary", word.named()));
        assert_eq!(phrase.source().as_written(), word.says());
        assert!(
            phrase.note().is_some_and(|note| note.len() > 30),
            "{} has no note a translator could work from",
            word.named()
        );
    }

    // Reached, each through the one public road to it.
    let offered = standing("aa", "bb", Vouching::NobodyHasVouchedForIt).said(&strings);
    let refused = apply(
        &ABaseThatRefuses::saying(WHAT_THE_BASE_SAID),
        &ready("aa", "bb", Vouching::NobodyHasVouchedForIt),
        &the_source(),
        WhenItApplies::AtTheNextRestart,
    )
    .unwrap_err()
    .said(&strings);

    assert_eq!(offered.text(), words::READY_NOT_VOUCHED_FOR.says());
    assert_eq!(refused.text(), words::NOT_GENUINE.says());
    assert_ne!(offered.text(), refused.text());
}

/// **Nothing about an offer nobody vouched for weakens the signature policy.**
///
/// The instruction is the same instruction, `--enforce-container-sigpolicy`
/// and all, under either reading and under both of the person's choices. There
/// is no road here to staging a build the policy refuses, which is the whole of
/// what makes an offer safe to act on.
#[test]
fn nothing_about_an_unvouched_offer_weakens_the_signature_policy() {
    for vouching in Vouching::EVERY {
        for when in WhenItApplies::EVERY {
            let base = ABaseThatRefuses::saying(WHAT_THE_BASE_SAID);
            let refused = apply(&base, &ready("aa", "bb", vouching), &the_source(), when)
                .expect_err("the stand-in stages nothing");
            assert!(
                matches!(refused, NotApplied::TheBuildWasNotGenuine(_)),
                "{refused:?}"
            );
            let told = base.told_to_do();
            let arguments = told.first().expect("the base was told to switch");
            assert!(
                arguments.contains(&"--enforce-container-sigpolicy".to_owned()),
                "{arguments:?}"
            );
            assert_eq!(arguments.first().unwrap(), "switch");
        }
    }
}

/// **What the real base's signature policy says about the real place's
/// builds** — the measurement half of task 7's first criterion.
///
/// It runs the base's **own** `skopeo` and `containers-common`, out of the
/// image `image/Containerfile` pins, against
/// `ghcr.io/aloworld-org/alo-os`, under a policy requiring the key
/// `image/signing/alo-os.pub`. That library is the one the base's `bootc`
/// stages a build with, so its answer is the answer a person's machine would
/// get — closer than a version of it that happens to be installed on the host,
/// and far cheaper than a boot.
///
/// **What it does not show.** It is not `bootc` itself and it is not a
/// certified machine: what it measures is the *policy decision*, which is
/// containers/image's, not the staging, which is the base's. A real boot is
/// `tests/an_update_keeps_the_persons_things.rs`'s shape and is owed here too
/// once a release exists that the policy accepts.
///
/// Run it by hand with:
///
/// ```text
/// cargo test -p alo-updating --test an_offer_a_person_can_act_on -- --ignored --nocapture
/// ```
#[test]
#[ignore = "runs the pinned base under podman against the real place"]
fn what_the_real_base_says_about_the_real_places_builds() {
    let pinned = the_pin().digest().to_owned();
    let said = the_base_asked_about(&["0.0.4", &pinned]);
    println!("{said}");

    // **The refusal this machine reads is the one the base actually prints.**
    let refusals: Vec<&str> = said
        .lines()
        .filter(|line| line.contains("level=fatal"))
        .collect();
    assert!(
        !refusals.is_empty(),
        "the base accepted every build, so no refusal was measured:\n{said}"
    );
    for refusal in refusals {
        assert!(
            refused_for_its_signature(&NotAnswered::SaidNo {
                program: "bootc".to_owned(),
                code: Some(1),
                said: refusal.to_owned(),
            }),
            "this machine would not read as a signature refusal: {refusal}"
        );
    }

    // **And the machine is unchanged**: nothing was written, because the
    // policy is decided before a single layer of a build is fetched.
    assert!(
        !said.contains("Copying blob"),
        "a build was fetched before the policy refused it:\n{said}"
    );
}

/// The pin this repository ships, read from the one file that holds it.
fn the_pin() -> alo_image::ThePin {
    let text = std::fs::read_to_string(
        std::path::Path::new(alo_image::THE_IMAGE).join(alo_image::THE_PIN),
    )
    .expect("the pin this repository ships");
    alo_image::ThePin::read(&text).expect("the pin this repository ships")
}

/// The base this image is built on, read out of `image/Containerfile` rather
/// than written down again here — one file, one answer, and a base that moves
/// moves this with it.
fn the_base() -> String {
    let recipe =
        std::fs::read_to_string(std::path::Path::new(alo_image::THE_IMAGE).join("Containerfile"))
            .expect("the recipe this image is built from");
    recipe
        .lines()
        .find_map(|line| line.strip_prefix("ARG THE_BASE="))
        .expect("the recipe names the base it is built on")
        .trim()
        .to_owned()
}

/// Ask the pinned base's own tooling whether its signature policy would have
/// each of these builds, and return everything it said.
#[cfg(unix)]
fn the_base_asked_about(references: &[&str]) -> String {
    use std::process::Command;

    let pin = the_pin();
    let folder = std::env::temp_dir().join(format!("alo-offer-{}", std::process::id()));
    std::fs::create_dir_all(folder.join("registries.d")).unwrap();
    std::fs::copy(
        std::path::Path::new(alo_image::THE_IMAGE).join("signing/alo-os.pub"),
        folder.join("alo-os.pub"),
    )
    .unwrap();
    std::fs::write(
        folder.join("policy.json"),
        format!(
            r#"{{"default":[{{"type":"reject"}}],"transports":{{"docker":{{"{}":[{{"type":"sigstoreSigned","keyPath":"/sig/alo-os.pub","signedIdentity":{{"type":"matchRepository"}}}}]}}}}}}"#,
            pin.registry()
        ),
    )
    .unwrap();
    std::fs::write(
        folder.join("registries.d/the-place.yaml"),
        format!(
            "docker:\n  {}:\n    use-sigstore-attachments: true\n",
            pin.registry().split('/').next().unwrap()
        ),
    )
    .unwrap();

    let asking = references
        .iter()
        .map(|reference| {
            let at = if reference.starts_with("sha256:") {
                format!("{}@{reference}", pin.registry())
            } else {
                format!("{}:{reference}", pin.registry())
            };
            format!(
                "echo \"=== {at}\"; timeout 60 skopeo --policy /sig/policy.json \
                 --registries.d /sig/registries.d copy docker://{at} dir:/tmp/asked 2>&1 | \
                 head -3; rm -rf /tmp/asked"
            )
        })
        .collect::<Vec<String>>()
        .join("; ");

    let said = Command::new("podman")
        .args([
            "run",
            "--rm",
            "--pull=never",
            "--network=host",
            "-v",
            &format!("{}:/sig:ro", folder.display()),
            &the_base(),
            "sh",
            "-c",
            &asking,
        ])
        .output()
        .expect("podman, the pinned base pulled, and a way out to the place");
    assert!(
        said.status.success(),
        "the base could not be asked: {}",
        String::from_utf8_lossy(&said.stderr)
    );
    String::from_utf8_lossy(&said.stdout).into_owned()
}

/// Nowhere else: the base is a Linux image, and this machine's Linux side is
/// where the gates run anyway.
#[cfg(not(unix))]
fn the_base_asked_about(_references: &[&str]) -> String {
    panic!("the base can only be asked on the Linux side of this machine")
}
