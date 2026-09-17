//! Every promise the published image makes, checked against the recipe, the
//! key and the document a person follows.
//!
//! `crate::checking` asks whether the image's files agree with each other. This
//! asks the same question about the one fact that lives outside them: which
//! bytes in the registry *are* this release. [ADR 0033](../../../docs/decisions/0033-the-certified-laptop-is-installed-the-way-a-customer-installs.md)
//! §3 pins that digest in `image/pinned.toml`;
//! [ADR 0036](../../../docs/decisions/0036-the-image-is-signed-by-a-key-a-person-holds.md)
//! says it was signed by a person, by digest; and
//! [ADR 0023](../../../docs/decisions/0023-installed-from-the-machine-it-replaces.md)
//! §3 says the signature is verified before anything is written.
//!
//! Each of those can go wrong silently, in a file nobody reviews twice: the
//! recipe moves to a release the pin never heard of, the key file becomes
//! something other than a public key, or `docs/booting.md` keeps telling a
//! person to pull the release before last — or pulls by a tag, or installs
//! before it verifies. Every one of those is a green build.
//!
//! # What this cannot check
//!
//! That the digest is really in the registry, and that its signature really
//! verifies. Both need the network, and a test in this crate reaches nothing
//! off the machine. They were checked by the owner when the digest was handed
//! over, and they are checked again by whatever pulls it — which is the point
//! of the signature.

use crate::booting;
use crate::image::Image;
use crate::pinned::THE_REGISTRY;
use crate::wrong::Wrong;

/// The section of `docs/booting.md` that installs what was published.
pub const THE_SECTION: &str = "Installing the published image";

/// What a document runs to install.
const INSTALL: &str = "bootc install";

/// What a document runs to verify a signature.
const VERIFY: &str = "cosign verify";

/// Where the image's directory is, as a path in this repository, which is how
/// a person at the root of a checkout names the key.
const THE_IMAGES_DIRECTORY: &str = "image/";

/// What a PEM public key begins with.
const BEGIN_PUBLIC: &str = "-----BEGIN PUBLIC KEY-----";

/// What it ends with.
const END_PUBLIC: &str = "-----END PUBLIC KEY-----";

/// What every PEM block begins with.
const BEGIN_ANY: &str = "-----BEGIN";

/// What a document says where it says nothing, in a sentence somebody reads.
const NOTHING: &str = "-";

/// Everything the published image's pin disagrees with.
pub(crate) fn everything_wrong_with_the_publish(image: &Image, wrong: &mut Vec<Wrong>) {
    the_pin_is_in_the_decided_registry(image, wrong);
    the_pin_is_the_recipes_release(image, wrong);
    the_key_is_one_public_key(image, wrong);
    the_document_installs_what_is_pinned(image, wrong);
}

/// **The pin names the registry ADR 0033 publishes to.**
fn the_pin_is_in_the_decided_registry(image: &Image, wrong: &mut Vec<Wrong>) {
    let pinned = image.pin().registry();
    if pinned != THE_REGISTRY {
        wrong.push(Wrong::ThePinIsNotTheDecidedRegistry {
            pinned: pinned.to_owned(),
        });
    }
}

/// **The recipe builds the release that is pinned, or the one declared next.**
///
/// The strict half is the plan's acceptance: a recipe naming one release and a
/// pin naming another is a test that fails. The declared half exists because
/// the next release has to be built from a commit of main whose recipe already
/// names it (ADR 0036, step 1) — and that commit cannot exist if the recipe and
/// the pin must be equal at every commit. So a newer release is *said*, in the
/// pin, as `next`, rather than tolerated in silence; it has to come after what
/// was signed, and the recipe has to name exactly it.
fn the_pin_is_the_recipes_release(image: &Image, wrong: &mut Vec<Wrong>) {
    let pin = image.pin();
    let recipe = image.version().said();

    if let Some(next) = pin.next()
        && !comes_after(next, pin.version())
    {
        wrong.push(Wrong::TheNextReleaseDoesNotFollowThePin {
            pinned: pin.version().to_owned(),
            next: next.to_owned(),
        });
    }

    let expected = pin.next().unwrap_or(pin.version());
    if recipe != Some(expected) {
        wrong.push(Wrong::ThePinIsNotTheRecipesRelease {
            recipe: recipe.unwrap_or(NOTHING).to_owned(),
            pinned: pin.version().to_owned(),
            next: pin
                .next()
                .map(|next| format!(", declaring `{next}` next"))
                .unwrap_or_default(),
        });
    }
}

/// Whether this release comes after that one, number by number.
fn comes_after(later: &str, earlier: &str) -> bool {
    matches!((numbers(later), numbers(earlier)), (Some(later), Some(earlier)) if later > earlier)
}

/// A release as three numbers, where it is one.
fn numbers(release: &str) -> Option<(u64, u64, u64)> {
    let mut parts = release.split('.').map(str::parse::<u64>);
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(Ok(major)), Some(Ok(minor)), Some(Ok(patch)), None) => Some((major, minor, patch)),
        _ => None,
    }
}

/// **The key the pin names is one public key, and nothing else.**
///
/// A puller verifies against it, so it has to be there; and a private half in
/// its place would hand the root of trust for every installed machine to
/// anybody who can read the repository.
fn the_key_is_one_public_key(image: &Image, wrong: &mut Vec<Wrong>) {
    let key = image.key().trim();
    let one_public_key = key.starts_with(BEGIN_PUBLIC)
        && key.ends_with(END_PUBLIC)
        && key.matches(BEGIN_ANY).count() == 1
        && !key.contains("PRIVATE");
    if !one_public_key {
        wrong.push(Wrong::ThePinsKeyIsNotOnePublicKey {
            key: image.pin().key().to_owned(),
        });
    }
}

/// **`docs/booting.md` verifies, then installs, exactly what is pinned.**
///
/// The registry, the tag and the digest it states are the pin's; the one
/// `bootc install` that pulls from the registry names the digest; nothing in it
/// names the registry by anything else; and the first command that touches the
/// pinned image is the one verifying its signature with the committed key.
fn the_document_installs_what_is_pinned(image: &Image, wrong: &mut Vec<Wrong>) {
    let pin = image.pin();
    let document = image.document();

    if !document.has_a_section(THE_SECTION) {
        wrong.push(Wrong::TheDocumentIsMissingASection {
            heading: THE_SECTION.to_owned(),
        });
    }

    for (fact, pinned) in [
        (booting::THE_REGISTRY, pin.registry()),
        (booting::THE_TAG, pin.version()),
        (booting::THE_DIGEST, pin.digest()),
    ] {
        let said = document.says(fact);
        if said != Some(pinned) {
            wrong.push(Wrong::TheDocumentDoesNotSayWhatIsPinned {
                fact: fact.to_owned(),
                said: said.unwrap_or(NOTHING).to_owned(),
                pinned: pinned.to_owned(),
            });
        }
    }

    let reference = pin.reference();
    let commands = document.commands();

    if !commands
        .iter()
        .any(|command| command.contains(INSTALL) && command.contains(&reference))
    {
        wrong.push(Wrong::TheDocumentDoesNotInstallWhatIsPinned {
            reference: reference.clone(),
        });
    }

    for command in commands {
        if names_the_registry_otherwise(command, pin.registry(), pin.digest()) {
            wrong.push(Wrong::TheDocumentPullsByAName {
                command: command.clone(),
            });
        }
    }

    let key = format!("{THE_IMAGES_DIRECTORY}{}", pin.key());
    let first_touch = commands
        .iter()
        .position(|command| command.contains(&reference));
    let verifies = first_touch
        .and_then(|at| commands.get(at))
        .is_some_and(|command| {
            command.contains(VERIFY) && command.contains(&format!("--key {key}"))
        });
    if !verifies {
        wrong.push(Wrong::TheDocumentWritesBeforeItVerifies { reference, key });
    }
}

/// Whether this command names the registry anywhere other than as the pinned
/// digest — by a tag, by another digest, or bare, which a tool reads as
/// `latest`.
fn names_the_registry_otherwise(command: &str, registry: &str, digest: &str) -> bool {
    let pinned = format!("@{digest}");
    command.match_indices(registry).any(|(at, _)| {
        !command
            .get(at + registry.len()..)
            .is_some_and(|rest| rest.starts_with(&pinned))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checking::everything_wrong_with;
    use crate::pinned::THE_PIN;
    use crate::testing::{
        THE_BOOTING_DOCUMENT, THE_CONTAINERFILE, a_copy_of_the_image, edited, image_at,
    };

    /// The digest the owner signed, as the shipped pin and document state it.
    const THE_DIGEST: &str =
        "sha256:d3f05b60975edcff51a44c1f21e764a32b286677e306ba24631bad6a00b6a13c";

    /// The recipe's release line, as the repository ships it.
    const THE_RELEASE_LABEL: &str = "LABEL org.opencontainers.image.version=\"0.0.2\"";

    /// The release line the pinned digest was built from.
    const THE_PINNED_RELEASE_LABEL: &str = "LABEL org.opencontainers.image.version=\"0.0.1\"";

    /// The candidate the shipped pin declares.
    const THE_DECLARED_NEXT: &str = "next = \"0.0.2\"";

    /// A copy of the image with **no release in flight**.
    ///
    /// Every scenario below is about the recipe and the pin disagreeing, and
    /// since 2026-09-17 the shipped files disagree *legitimately*: 0.0.2 is
    /// declared and being prepared, so the recipe names it while the pin still
    /// holds the digest of 0.0.1. A fixture built straight from that would be
    /// arranging two disagreements and asserting about one, and would go on
    /// passing if the thing it meant to catch stopped being caught.
    ///
    /// So each starts from the state this repository is in *between* releases:
    /// the recipe naming exactly what the pin holds, and nothing declared.
    fn a_copy_between_releases(what: &str) -> std::path::PathBuf {
        let root = a_copy_of_the_image(what);
        edited(&root, THE_PIN, THE_DECLARED_NEXT, "");
        edited(
            &root,
            THE_CONTAINERFILE,
            THE_RELEASE_LABEL,
            THE_PINNED_RELEASE_LABEL,
        );
        root
    }

    /// Everything wrong with the image at this root.
    fn wrong_at(root: &std::path::Path) -> Vec<Wrong> {
        everything_wrong_with(&image_at(root))
    }

    /// **The image this repository ships agrees with its pin**: the recipe
    /// names what was signed, the key is one public key, and the document
    /// verifies then installs the pinned digest.
    #[test]
    fn the_shipped_pin_agrees_with_the_recipe_the_key_and_the_document() {
        let mut wrong = Vec::new();
        everything_wrong_with_the_publish(
            &image_at(std::path::Path::new(crate::THE_IMAGE)),
            &mut wrong,
        );
        assert!(wrong.is_empty(), "{wrong:?}");
    }

    /// **A recipe that moved to another release without saying so is caught.**
    /// This is the acceptance in as many words: the recipe's version and the
    /// pinned digest disagree, and a test fails.
    #[test]
    fn a_recipe_naming_a_release_the_pin_does_not_is_caught() {
        let root = a_copy_between_releases("recipe-moved-on");
        edited(
            &root,
            THE_CONTAINERFILE,
            THE_PINNED_RELEASE_LABEL,
            "LABEL org.opencontainers.image.version=\"0.0.2\"",
        );

        let wrong = wrong_at(&root);

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::ThePinIsNotTheRecipesRelease { recipe, pinned, .. }
                    if recipe == "0.0.2" && pinned == "0.0.1"
            )),
            "{wrong:?}"
        );
    }

    /// **And so is a pin moved to a release the recipe never named**, which is
    /// a digest written against a version nothing built.
    #[test]
    fn a_pin_naming_a_release_the_recipe_does_not_is_caught() {
        let root = a_copy_between_releases("pin-moved-on");
        edited(&root, THE_PIN, "version = \"0.0.1\"", "version = \"0.0.3\"");

        let wrong = wrong_at(&root);

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::ThePinIsNotTheRecipesRelease { recipe, pinned, .. }
                    if recipe == "0.0.1" && pinned == "0.0.3"
            )),
            "{wrong:?}"
        );
    }

    /// **A recipe with no release at all disagrees with every pin.**
    #[test]
    fn a_recipe_naming_no_release_disagrees_with_the_pin() {
        let root = a_copy_between_releases("recipe-no-release");
        edited(&root, THE_CONTAINERFILE, THE_PINNED_RELEASE_LABEL, "");

        let wrong = wrong_at(&root);

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::ThePinIsNotTheRecipesRelease { recipe, .. } if recipe == "-"
            )),
            "{wrong:?}"
        );
    }

    /// **The next release, declared, is the one legitimate disagreement** — and
    /// only while the recipe names exactly it.
    #[test]
    fn a_declared_next_release_the_recipe_names_is_a_candidate() {
        let root = a_copy_between_releases("declared-next");
        edited(
            &root,
            THE_CONTAINERFILE,
            THE_PINNED_RELEASE_LABEL,
            "LABEL org.opencontainers.image.version=\"0.1.0\"",
        );
        edited(
            &root,
            THE_PIN,
            "version = \"0.0.1\"",
            "version = \"0.0.1\"\nnext = \"0.1.0\"",
        );

        let wrong = wrong_at(&root);

        assert!(
            !wrong.iter().any(|it| matches!(
                it,
                Wrong::ThePinIsNotTheRecipesRelease { .. }
                    | Wrong::TheNextReleaseDoesNotFollowThePin { .. }
            )),
            "{wrong:?}"
        );
    }

    /// **A declared next release the recipe has not reached is caught**: once
    /// `next` is written, the recipe names it, or the declaration is a promise
    /// about a build nobody is making.
    #[test]
    fn a_declared_next_release_the_recipe_does_not_name_is_caught() {
        let root = a_copy_between_releases("next-not-built");
        edited(
            &root,
            THE_PIN,
            "version = \"0.0.1\"",
            "version = \"0.0.1\"\nnext = \"0.0.2\"",
        );

        let wrong = wrong_at(&root);

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::ThePinIsNotTheRecipesRelease { recipe, next, .. }
                    if recipe == "0.0.1" && next.contains("0.0.2")
            )),
            "{wrong:?}"
        );
    }

    /// **A next release that does not come after the signed one is caught** —
    /// the same release again, or an older one, compared as numbers so that
    /// `0.0.10` comes after `0.0.9`.
    #[test]
    fn a_next_release_that_does_not_follow_is_caught() {
        for (pinned, next, follows) in [
            ("0.0.1", "0.0.1", false),
            ("0.1.0", "0.0.9", false),
            ("0.0.9", "0.0.10", true),
            ("0.9.9", "1.0.0", true),
        ] {
            let root = a_copy_between_releases(&format!("next-{pinned}-{next}"));
            edited(
                &root,
                THE_CONTAINERFILE,
                THE_PINNED_RELEASE_LABEL,
                &format!("LABEL org.opencontainers.image.version=\"{next}\""),
            );
            edited(
                &root,
                THE_PIN,
                "version = \"0.0.1\"",
                &format!("version = \"{pinned}\"\nnext = \"{next}\""),
            );
            edited(
                &root,
                THE_BOOTING_DOCUMENT,
                "tag: 0.0.1",
                &format!("tag: {pinned}"),
            );

            let wrong = wrong_at(&root);
            let caught = wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheNextReleaseDoesNotFollowThePin { .. }));

            assert_eq!(caught, !follows, "{pinned} then {next}: {wrong:?}");
        }
    }

    /// **A pin naming another registry is caught.**
    #[test]
    fn a_pin_in_another_registry_is_caught() {
        let root = a_copy_of_the_image("another-registry");
        edited(
            &root,
            THE_PIN,
            "registry = \"ghcr.io/aloworld-org/alo-os\"",
            "registry = \"docker.io/somebody/alo-os\"",
        );

        let wrong = wrong_at(&root);

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::ThePinIsNotTheDecidedRegistry { .. })),
            "{wrong:?}"
        );
    }

    /// **A key file that is not one public key is caught**: a private half in
    /// its place, two keys, or something that is not a key at all.
    #[test]
    fn a_key_that_is_not_one_public_key_is_caught() {
        for (what, instead) in [
            (
                "private",
                "-----BEGIN ENCRYPTED SIGSTORE PRIVATE KEY-----\nabc\n-----END ENCRYPTED SIGSTORE PRIVATE KEY-----\n",
            ),
            (
                "two",
                "-----BEGIN PUBLIC KEY-----\nabc\n-----END PUBLIC KEY-----\n-----BEGIN PUBLIC KEY-----\ndef\n-----END PUBLIC KEY-----\n",
            ),
            ("nothing", "not a key\n"),
        ] {
            let root = a_copy_of_the_image(&format!("key-{what}"));
            assert!(std::fs::write(root.join("signing/alo-os.pub"), instead).is_ok());

            let wrong = wrong_at(&root);

            assert!(
                wrong
                    .iter()
                    .any(|it| matches!(it, Wrong::ThePinsKeyIsNotOnePublicKey { .. })),
                "{what}: {wrong:?}"
            );
        }
    }

    /// **A document naming another digest is caught**, both as the fact and as
    /// the install that no longer pulls what is pinned.
    #[test]
    fn a_document_pulling_another_digest_is_caught() {
        let root = a_copy_of_the_image("document-another-digest");
        let other = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
        edited(&root, THE_BOOTING_DOCUMENT, THE_DIGEST, other);

        let wrong = wrong_at(&root);

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheDocumentDoesNotSayWhatIsPinned { fact, .. } if fact == "digest"
            )),
            "{wrong:?}"
        );
        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheDocumentDoesNotInstallWhatIsPinned { .. })),
            "{wrong:?}"
        );
        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheDocumentPullsByAName { .. })),
            "{wrong:?}"
        );
    }

    /// **A document still telling somebody the release before is caught.**
    #[test]
    fn a_document_naming_another_tag_is_caught() {
        let root = a_copy_of_the_image("document-another-tag");
        edited(&root, THE_BOOTING_DOCUMENT, "tag: 0.0.1", "tag: 0.0.0");

        let wrong = wrong_at(&root);

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheDocumentDoesNotSayWhatIsPinned { fact, said, .. }
                    if fact == "tag" && said == "0.0.0"
            )),
            "{wrong:?}"
        );
    }

    /// **A document that pulls by the tag is caught**, even with the digest
    /// still named beside it: a tag can be moved after the owner signed.
    #[test]
    fn a_document_pulling_by_the_tag_is_caught() {
        let root = a_copy_of_the_image("document-by-tag");
        edited(
            &root,
            THE_BOOTING_DOCUMENT,
            &format!("    podman pull ghcr.io/aloworld-org/alo-os@{THE_DIGEST}"),
            "    podman pull ghcr.io/aloworld-org/alo-os:0.0.1",
        );

        let wrong = wrong_at(&root);

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheDocumentPullsByAName { command } if command.contains(":0.0.1")
            )),
            "{wrong:?}"
        );
    }

    /// **A document that installs before it verifies is caught**, and so is one
    /// that verifies against something other than the committed key, or not at
    /// all.
    #[test]
    fn a_document_that_writes_before_it_verifies_is_caught() {
        let verify = "    cosign verify --key image/signing/alo-os.pub";
        for (what, instead) in [
            ("no-verify", "    echo".to_owned()),
            (
                "another-key",
                "    cosign verify --key somebody.pub".to_owned(),
            ),
        ] {
            let root = a_copy_of_the_image(&format!("document-{what}"));
            edited(&root, THE_BOOTING_DOCUMENT, verify, &instead);

            let wrong = wrong_at(&root);

            assert!(
                wrong
                    .iter()
                    .any(|it| matches!(it, Wrong::TheDocumentWritesBeforeItVerifies { .. })),
                "{what}: {wrong:?}"
            );
        }

        let root = a_copy_of_the_image("document-verify-after");
        let document = root.join(THE_BOOTING_DOCUMENT);
        let text = std::fs::read_to_string(&document).unwrap_or_default();
        let pull = format!("    podman pull ghcr.io/aloworld-org/alo-os@{THE_DIGEST}\n");
        assert!(
            text.contains(&pull),
            "the document does not pull the digest"
        );
        let moved = text.replacen(&pull, "", 1).replacen(
            "## Installing the published image\n",
            &format!("## Installing the published image\n\n{pull}\n"),
            1,
        );
        assert!(std::fs::write(&document, moved).is_ok());

        let wrong = wrong_at(&root);

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheDocumentWritesBeforeItVerifies { .. })),
            "{wrong:?}"
        );
    }

    /// **A document without the section is caught.**
    #[test]
    fn a_document_without_the_published_section_is_caught() {
        let root = a_copy_of_the_image("document-no-published-section");
        edited(
            &root,
            THE_BOOTING_DOCUMENT,
            "## Installing the published image",
            "## Somewhere else",
        );

        let wrong = wrong_at(&root);

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheDocumentIsMissingASection { heading } if heading == THE_SECTION
            )),
            "{wrong:?}"
        );
    }

    /// Releases compare as numbers, and anything that is not one follows
    /// nothing.
    #[test]
    fn releases_compare_as_numbers() {
        assert!(comes_after("0.0.10", "0.0.9"));
        assert!(comes_after("1.0.0", "0.99.99"));
        assert!(!comes_after("0.0.1", "0.0.1"));
        assert!(!comes_after("latest", "0.0.1"));
        assert!(!comes_after("0.0.1.1", "0.0.1"));
    }
}
