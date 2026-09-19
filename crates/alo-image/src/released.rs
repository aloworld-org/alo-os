//! What the Release and the page a person downloads from say, held to what is
//! pinned and to what the installer actually accepts.
//!
//! `crate::publishing` holds the image's pin to the recipe, the key and
//! `docs/booting.md`. This holds it to the two things a **person** reads before
//! running the program: the Release's notes, and the README's *Try it*. The
//! installer plan's task 5 asks for both — *the Release notes say which image
//! digest it installs and which Secure Boot state it accepts*, and *the README
//! gains a `Try it` section … and nothing else*.
//!
//! # Why a test, and not a proofread
//!
//! Every one of these is a green build and a sentence that has quietly become a
//! lie: notes still naming the release before last, so a person reads that they
//! are getting bytes nothing will pull; notes saying Secure Boot may be on while
//! `alo_installer::decide` refuses it, so a person restarts a computer that was
//! never going to install; a README requiring 16 GB where `docs/hardware.md`
//! says 32, so somebody buys the wrong machine on our word.
//!
//! # Where each fact comes from
//!
//! Nothing here is written twice. The digest, the tag and the registry are
//! `image/pinned.toml`'s, and the requirements are `docs/hardware.md`'s own
//! table. A checker holding a document to a second spelling of the answer is
//! the drift this crate exists to catch.
//!
//! **The one exception is the Secure Boot state**, and it is a dependency
//! rather than a preference: `alo-installing` already depends on this crate, so
//! this crate cannot ask `alo-installer` what it accepts without a cycle. So
//! [`ACCEPTED`] is written here, and the question is asked from the other end —
//! `crates/alo-installer/tests/the_release_says_what_it_accepts.rs` puts a
//! scripted machine in each of the three states to `alo_installer::decide` and
//! holds these notes to what it refuses. The rule has one home; this is the
//! reader of it.

use crate::image::Image;
use crate::notes;
use crate::trying::{Requirement, TheTryIt, what_to_buy};
use crate::wrong::Wrong;

/// What a document says where it says nothing, in a sentence somebody reads.
const NOTHING: &str = "-";

/// The rows of `docs/hardware.md`'s table a person needs before they download.
///
/// Not every row: the README's rule since 2026-09-13 is no noise, and wireless
/// chipsets and graphics are what `docs/hardware.md` is for. These five are the
/// ones that decide whether the program will run at all on the computer in
/// front of them.
const WHAT_A_PERSON_NEEDS: [&str; 5] =
    ["Memory", "Storage", "Processor", "Firmware", "Arrives with"];

/// What the README's one Secure Boot sentence has to say the state is.
const OFF: &str = "off";

/// The Secure Boot state this release's installer accepts, as the notes must
/// state it.
///
/// One state today: ADR 0033 §4 refuses *on*, and *could not be found out* is
/// never read as *off*. The plan's task 9 is what would change it, and the test
/// named at the top of this file is what makes that change arrive here.
pub const ACCEPTED: &str = OFF;

/// The name of the setting, wherever it is written.
const SECURE_BOOT: &str = "Secure Boot";

/// What Windows stays is said as.
const WINDOWS_STAYS: &str = "Windows stays";

/// Every way of telling somebody to change the setting, which ADR 0033 §4
/// forbids however helpfully it is phrased.
const ADVICE: [&str; 5] = [
    "turn it off",
    "turn off",
    "switch it off",
    "switch off",
    "disable",
];

/// Everything the Release's notes disagree with.
pub(crate) fn everything_wrong_with_the_notes(image: &Image, wrong: &mut Vec<Wrong>) {
    let pin = image.pin();
    let notes = image.notes();

    for (fact, pinned) in [
        (notes::THE_REGISTRY, pin.registry()),
        (notes::THE_TAG, pin.version()),
        (notes::THE_DIGEST, pin.digest()),
    ] {
        let said = notes.says(fact);
        if said != Some(pinned) {
            wrong.push(Wrong::TheNotesDoNotSayWhatIsPinned {
                fact: fact.to_owned(),
                said: said.unwrap_or(NOTHING).to_owned(),
                pinned: pinned.to_owned(),
            });
        }
    }

    if notes.digests().is_empty() {
        wrong.push(Wrong::TheNotesNameAnotherDigest {
            digest: NOTHING.to_owned(),
            pinned: pin.digest().to_owned(),
        });
    }
    for digest in notes.digests() {
        if digest != pin.digest() {
            wrong.push(Wrong::TheNotesNameAnotherDigest {
                digest: digest.clone(),
                pinned: pin.digest().to_owned(),
            });
        }
    }

    let said = notes.says(notes::THE_SECURE_BOOT);
    if said != Some(ACCEPTED) {
        wrong.push(Wrong::TheNotesDoNotSayWhichSecureBootStateIsAccepted {
            said: said.unwrap_or(NOTHING).to_owned(),
            accepted: ACCEPTED.to_owned(),
        });
    }
}

/// Everything wrong with the README's *Try it*, against the hardware document.
///
/// Both are texts rather than paths, for `crate::image`'s reason: a rule about
/// a document is only a rule with a test if a test can run it against a
/// document it may write.
#[must_use]
pub fn everything_wrong_with_the_try_it(readme: &str, hardware: &str) -> Vec<Wrong> {
    let mut wrong = Vec::new();
    let read = TheTryIt::read(readme);

    if !read.is_there() {
        wrong.push(Wrong::TheReadmeDoesNotOfferTheInstaller {
            section: crate::trying::THE_SECTION.to_owned(),
        });
        return wrong;
    }

    let buy = what_to_buy(hardware);
    for needed in WHAT_A_PERSON_NEEDS {
        if !read.requirements().iter().any(|it| it.named == needed) {
            wrong.push(Wrong::TheReadmeIsMissingARequirement {
                named: needed.to_owned(),
            });
        }
    }
    for said in read.requirements() {
        if !is_what_the_document_says(said, &buy) {
            wrong.push(Wrong::TheReadmeRequiresSomethingElse {
                named: said.named.clone(),
                said: said.said.clone(),
                document: buy
                    .iter()
                    .find(|it| it.named == said.named)
                    .map_or(NOTHING, |it| it.said.as_str())
                    .to_owned(),
            });
        }
    }

    if !read.says_nothing_else() {
        wrong.push(Wrong::TheReadmeSaysMore);
    }
    if read.paragraphs_naming(WINDOWS_STAYS).is_empty() {
        wrong.push(Wrong::TheReadmeDoesNotSayWindowsStays);
    }

    match read.paragraphs_naming(SECURE_BOOT).as_slice() {
        [only] => {
            let lowered = only.to_lowercase();
            if !lowered.contains(OFF) || ADVICE.iter().any(|advice| lowered.contains(advice)) {
                wrong.push(Wrong::TheReadmeArguesWithSecureBoot {
                    paragraph: (*only).to_owned(),
                });
            }
        }
        said => wrong.push(Wrong::TheReadmeSaysSecureBootTwice { times: said.len() }),
    }
    wrong
}

/// Whether the document's table says what the README says it does.
///
/// Compared piece by piece rather than word for word: the README says *UEFI,
/// TPM 2.0* where the table's row says *UEFI, Secure Boot, TPM 2.0*, because
/// this release does not install where Secure Boot is on and a requirement it
/// does not have is noise. What may never happen is the README naming something
/// the table does not.
fn is_what_the_document_says(said: &Requirement, buy: &[Requirement]) -> bool {
    buy.iter()
        .find(|it| it.named == said.named)
        .is_some_and(|it| said.said.split(", ").all(|piece| it.said.contains(piece)))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::checking::everything_wrong_with;
    use crate::notes::THE_NOTES;
    use crate::testing::{a_copy_of_the_image, edited, image_at, the_tag_line};
    use crate::trying::{THE_HARDWARE, THE_README};

    /// The digest the owner signed, as the shipped pin and notes state it.
    const THE_DIGEST: &str =
        "sha256:8f9c36e0d608eb13d8ba7746b9c549438a939bcbd51e90e2b5fcd5103be90bf9";

    /// A file this repository ships, as text.
    fn text(at: &str) -> String {
        std::fs::read_to_string(at).unwrap()
    }

    /// Everything wrong with the image at this root.
    fn wrong_at(root: &std::path::Path) -> Vec<Wrong> {
        everything_wrong_with(&image_at(root))
    }

    /// Everything wrong with the shipped README, with one thing changed in it.
    fn the_readme_with(from: &str, to: &str) -> Vec<Wrong> {
        let readme = text(THE_README);
        assert!(readme.contains(from), "the README has no `{from}`");
        everything_wrong_with_the_try_it(&readme.replace(from, to), &text(THE_HARDWARE))
    }

    /// **The notes this repository ships agree with its pin**, and say the state
    /// the installer accepts.
    #[test]
    fn the_shipped_notes_agree_with_the_pin() {
        let mut wrong = Vec::new();
        everything_wrong_with_the_notes(
            &image_at(std::path::Path::new(crate::THE_IMAGE)),
            &mut wrong,
        );
        assert!(wrong.is_empty(), "{wrong:?}");
    }

    /// **Notes naming a digest the pin does not are refused.** This is the
    /// plan's acceptance in as many words.
    #[test]
    fn notes_naming_another_digest_are_refused() {
        let other = format!("sha256:{}", "0".repeat(64));
        let root = a_copy_of_the_image("notes-another-digest");
        edited(&root, THE_NOTES, THE_DIGEST, &other);

        let wrong = wrong_at(&root);

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheNotesDoNotSayWhatIsPinned { fact, .. } if fact == "digest"
            )),
            "{wrong:?}"
        );
        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheNotesNameAnotherDigest { digest, .. } if digest == &other
            )),
            "{wrong:?}"
        );
    }

    /// **And so is a second digest in a sentence**, with the stated fact left
    /// alone: a person reading notes that name two does not know which they are
    /// getting.
    #[test]
    fn notes_naming_a_second_digest_in_prose_are_refused() {
        let other = format!("sha256:{}", "a".repeat(64));
        let root = a_copy_of_the_image("notes-second-digest");
        edited(
            &root,
            THE_NOTES,
            "Nothing but that digest is pulled",
            &format!("The release before it was {other}. Nothing but that digest is pulled"),
        );

        let wrong = wrong_at(&root);

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheNotesNameAnotherDigest { digest, .. } if digest == &other
            )),
            "{wrong:?}"
        );
    }

    /// **Notes naming no digest at all are refused**, which is the notes
    /// somebody rewrote as a paragraph of welcome.
    #[test]
    fn notes_naming_no_digest_are_refused() {
        let root = a_copy_of_the_image("notes-no-digest");
        edited(&root, THE_NOTES, THE_DIGEST, "the one we published");

        let wrong = wrong_at(&root);

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheNotesNameAnotherDigest { digest, .. } if digest == NOTHING
            )),
            "{wrong:?}"
        );
    }

    /// **Notes naming another release are refused**, so the tag in them cannot
    /// be the one before.
    #[test]
    fn notes_naming_another_release_are_refused() {
        let root = a_copy_of_the_image("notes-another-tag");
        edited(
            &root,
            THE_NOTES,
            &format!("    {}", the_tag_line()),
            "    tag: 0.0.0",
        );

        let wrong = wrong_at(&root);

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheNotesDoNotSayWhatIsPinned { fact, said, .. }
                    if fact == "tag" && said == "0.0.0"
            )),
            "{wrong:?}"
        );
    }

    /// **Notes promising a Secure Boot state the installer refuses are
    /// refused**, whichever way they are wrong — and so are notes that stopped
    /// saying which state they accept.
    #[test]
    fn notes_promising_the_wrong_secure_boot_state_are_refused() {
        for (what, instead) in [
            ("on", "    secure boot: on"),
            ("either", "    secure boot: on or off"),
            ("nothing", "Secure Boot is somewhere else now"),
        ] {
            let root = a_copy_of_the_image(&format!("notes-secure-boot-{what}"));
            edited(&root, THE_NOTES, "    secure boot: off", instead);

            let wrong = wrong_at(&root);

            assert!(
                wrong.iter().any(|it| matches!(
                    it,
                    Wrong::TheNotesDoNotSayWhichSecureBootStateIsAccepted { accepted, .. }
                        if accepted == OFF
                )),
                "{what}: {wrong:?}"
            );
        }
    }

    /// **The README this repository ships offers the installer and says nothing
    /// the hardware document does not.**
    #[test]
    fn the_shipped_readme_offers_it_and_says_nothing_else() {
        let wrong = everything_wrong_with_the_try_it(&text(THE_README), &text(THE_HARDWARE));

        assert!(wrong.is_empty(), "{wrong:?}");
    }

    /// **A README with no `Try it` is caught**, and nothing else is said about
    /// it: there is no section to have anything wrong with.
    #[test]
    fn a_readme_without_the_section_is_caught() {
        let wrong = the_readme_with("## Try it", "## Somewhere else");

        assert_eq!(
            wrong,
            [Wrong::TheReadmeDoesNotOfferTheInstaller {
                section: "## Try it".to_owned(),
            }]
        );
    }

    /// **A requirement that drifted from the hardware document is caught** —
    /// which is somebody buying the wrong machine on our word.
    #[test]
    fn a_requirement_the_document_does_not_make_is_caught() {
        let wrong = the_readme_with("- **Memory** — 32 GB", "- **Memory** — 16 GB");

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheReadmeRequiresSomethingElse { named, said, .. }
                    if named == "Memory" && said == "16 GB"
            )),
            "{wrong:?}"
        );
    }

    /// **A requirement that went missing is caught**, so the list cannot quietly
    /// shrink to the two that are easy to meet.
    #[test]
    fn a_requirement_that_went_missing_is_caught() {
        let wrong = the_readme_with("- **Storage** — 1 TB NVMe\n", "");

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheReadmeIsMissingARequirement { named } if named == "Storage"
            )),
            "{wrong:?}"
        );
    }

    /// **A README that says the computer needs something nobody wrote down is
    /// caught**, because a requirement with no measurement behind it is a
    /// preference.
    #[test]
    fn a_requirement_from_nowhere_is_caught() {
        let wrong = the_readme_with(
            "- **Memory** — 32 GB",
            "- **Memory** — 32 GB\n- **Screen** — a good one",
        );

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheReadmeRequiresSomethingElse { named, document, .. }
                    if named == "Screen" && document == NOTHING
            )),
            "{wrong:?}"
        );
    }

    /// **A section that grew is caught.** The README's rule is no noise, and
    /// this is the section every future task will want one more sentence in.
    #[test]
    fn a_section_that_grew_is_caught() {
        let wrong = the_readme_with(
            "`docs/hardware.md` is where each of those comes from",
            "And another thing entirely.\n\nAnd one more.\n\n`docs/hardware.md` is where each of \
             those comes from",
        );

        assert!(wrong.contains(&Wrong::TheReadmeSaysMore), "{wrong:?}");
    }

    /// **A README that stopped saying Windows stays is caught**: it is the one
    /// thing a person is afraid of before they run the program.
    #[test]
    fn a_readme_that_stopped_saying_windows_stays_is_caught() {
        let wrong = the_readme_with("Windows stays:", "It is installed:");

        assert!(
            wrong.contains(&Wrong::TheReadmeDoesNotSayWindowsStays),
            "{wrong:?}"
        );
    }

    /// **A README that tells somebody to switch Secure Boot off is caught**,
    /// however helpfully it is phrased. ADR 0033 §4 is absolute, and this is
    /// the sentence somebody will one day write to be kind.
    #[test]
    fn a_readme_that_argues_with_secure_boot_is_caught() {
        for helpful in [
            "This release installs only where Secure Boot is off; you can turn it off in the \
             firmware menu.",
            "This release installs only where Secure Boot is off, so disable it first.",
            "Secure Boot is fine either way.",
        ] {
            let wrong = the_readme_with(
                "This release installs only where Secure Boot is already off; where it is on, the\n\
                 installer stops, says why, and changes nothing.",
                helpful,
            );

            assert!(
                wrong
                    .iter()
                    .any(|it| matches!(it, Wrong::TheReadmeArguesWithSecureBoot { .. })),
                "{helpful}: {wrong:?}"
            );
        }
    }

    /// **A README naming Secure Boot in two places is caught**: one sentence is
    /// what the plan asks for, and a second one is where the advice appears.
    #[test]
    fn a_readme_naming_secure_boot_twice_is_caught() {
        let wrong = the_readme_with(
            "Windows stays:",
            "Secure Boot is worth reading about. Windows stays:",
        );

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheReadmeSaysSecureBootTwice { times } if *times == 2
            )),
            "{wrong:?}"
        );
    }
}
