//! Every promise the boot environment's recipe makes, checked against the image
//! and the pin.
//!
//! `crate::installing` reads the recipe; this says what is wrong with it, one
//! [`Wrong`] per broken promise. The promises are the installer plan's task 2
//! and ADR 0023 §3 — the environment installs **the** signed release, with the
//! tool the image's own base carries, and checks the signature with a checker
//! that was itself checked.

use crate::image::Image;
use crate::installing::{ABOARD, THE_PIN_ABOARD};
use crate::pinned::THE_PIN;
use crate::wrong::Wrong;

/// Where the image's directory is, as a path in this repository.
const THE_IMAGES_DIRECTORY: &str = "image/";

/// Everything the boot environment's recipe disagrees with.
pub(crate) fn everything_wrong_with_the_environment(image: &Image, wrong: &mut Vec<Wrong>) {
    the_environment_is_the_images_base_and_toolchain(image, wrong);
    the_environment_carries_the_pin_and_its_key(image, wrong);
    the_checker_is_pinned_and_checked(image, wrong);
}

/// **The environment is built on the image's base, by the image's toolchain.**
fn the_environment_is_the_images_base_and_toolchain(image: &Image, wrong: &mut Vec<Wrong>) {
    for (argument, environment, image) in image.environment().differs_from_the_image() {
        wrong.push(Wrong::TheEnvironmentIsNotBuiltLikeTheImage {
            argument: argument.to_owned(),
            environment: environment.to_owned(),
            image: image.to_owned(),
        });
    }
}

/// **The environment carries this repository's pin, and the key the pin names.**
fn the_environment_carries_the_pin_and_its_key(image: &Image, wrong: &mut Vec<Wrong>) {
    let environment = image.environment();
    let pin = format!("{THE_IMAGES_DIRECTORY}{THE_PIN}");
    if !environment.copies(&pin, THE_PIN_ABOARD) {
        wrong.push(Wrong::TheEnvironmentDoesNotCarry {
            file: pin,
            landing: THE_PIN_ABOARD.to_owned(),
        });
    }
    let key = format!("{THE_IMAGES_DIRECTORY}{}", image.pin().key());
    let landing = format!("{ABOARD}{}", image.pin().key());
    if !environment.copies(&key, &landing) {
        wrong.push(Wrong::TheEnvironmentDoesNotCarry { file: key, landing });
    }
}

/// **The signature checker is one exact release, checked by digest before the
/// environment carries it.**
fn the_checker_is_pinned_and_checked(image: &Image, wrong: &mut Vec<Wrong>) {
    let environment = image.environment();
    if !environment.the_checker_is_pinned_and_checked() {
        wrong.push(Wrong::TheCheckerArrivesUnverified {
            version: environment.checker().unwrap_or("-").to_owned(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{a_copy_of_the_image, edited, image_at};

    /// What this crate finds wrong with an image, about the environment only.
    fn wrong_with_the_environment(root: &std::path::Path) -> Vec<Wrong> {
        let mut wrong = Vec::new();
        everything_wrong_with_the_environment(&image_at(root), &mut wrong);
        wrong
    }

    /// The shipped environment keeps every promise.
    #[test]
    fn the_shipped_environment_keeps_every_promise() {
        assert_eq!(
            wrong_with_the_environment(std::path::Path::new(crate::THE_IMAGE)),
            Vec::new()
        );
    }

    /// **An environment on another base is caught**, naming both values.
    #[test]
    fn an_environment_on_another_base_is_caught() {
        let root = a_copy_of_the_image("environment-base");
        edited(
            &root,
            crate::installing::THE_ENVIRONMENT,
            "ARG THE_BASE=quay.io/fedora/fedora-bootc:42@",
            "ARG THE_BASE=quay.io/fedora/fedora-bootc:41@",
        );
        let wrong = wrong_with_the_environment(&root);
        assert!(
            matches!(
                wrong.as_slice(),
                [Wrong::TheEnvironmentIsNotBuiltLikeTheImage { argument, .. }] if argument == "THE_BASE"
            ),
            "{wrong:?}"
        );
    }

    /// **An environment carrying another pin or another key is caught.**
    #[test]
    fn an_environment_carrying_another_pin_or_key_is_caught() {
        let root = a_copy_of_the_image("environment-pin");
        edited(
            &root,
            crate::installing::THE_ENVIRONMENT,
            "COPY image/pinned.toml /usr/lib/alo/installing/pinned.toml",
            "COPY somewhere/pinned.toml /usr/lib/alo/installing/pinned.toml",
        );
        edited(
            &root,
            crate::installing::THE_ENVIRONMENT,
            "COPY image/signing/alo-os.pub",
            "COPY image/signing/someone-else.pub",
        );
        let wrong = wrong_with_the_environment(&root);
        assert_eq!(wrong.len(), 2, "{wrong:?}");
        assert!(
            wrong
                .iter()
                .all(|one| matches!(one, Wrong::TheEnvironmentDoesNotCarry { .. })),
            "{wrong:?}"
        );
    }

    /// **A checker that is not checked is caught.**
    #[test]
    fn a_checker_that_is_not_checked_is_caught() {
        let root = a_copy_of_the_image("environment-checker");
        edited(
            &root,
            crate::installing::THE_ENVIRONMENT,
            "| sha256sum --check -",
            "| cat",
        );
        let wrong = wrong_with_the_environment(&root);
        assert_eq!(
            wrong,
            vec![Wrong::TheCheckerArrivesUnverified {
                version: "3.1.3".to_owned()
            }]
        );
    }
}
