//! What the boot environment's recipe says, held to the image and the pin.
//!
//! `image/installing/Containerfile` builds the environment the installer
//! restarts into (the installer plan's task 2). It is the second recipe in
//! `image/`, and it has exactly the ways to go wrong this crate exists for — a
//! green build of a machine that is wrong:
//!
//! - **another base.** The environment writes the disk with the `bootc` inside
//!   its base, and `docs/booting.md` says the version of that tool is the image's
//!   base digest. An environment on a different base is installing with a tool
//!   nobody pinned;
//! - **another toolchain.** The same builder, compiler and target as the image,
//!   so the one program of ours in it is built the way every other is;
//! - **another release, or another key.** The environment carries what it
//!   installs and whose signature it accepts. A recipe that copied some other
//!   pin, or a key from somewhere else, is an installer that trusts a file this
//!   repository does not hold to anything;
//! - **a checker nobody checked.** The signature checker is upstream's own
//!   binary. A floating version, half a digest, or a digest nobody runs
//!   `sha256sum --check` against is the one program deciding *genuine* arriving
//!   unverified.
//!
//! # Read leniently, judged strictly
//!
//! [`TheEnvironment::read`] never refuses, for `crate::runtime`'s reason: a
//! recipe that says nothing reads as one that pins nothing, and
//! `crate::checking` turns each absence into a sentence.

use crate::recipe::{argument, copies_from_a_stage_to, in_stage, is_a_whole_digest};

/// The environment's recipe, beneath the image's directory.
pub const THE_ENVIRONMENT: &str = "installing/Containerfile";

/// Where the environment carries the pin.
pub const THE_PIN_ABOARD: &str = "/usr/lib/alo/installing/pinned.toml";

/// Where the environment carries its programs' key and pin, beneath which the
/// pin's own relative key path is kept.
pub const ABOARD: &str = "/usr/lib/alo/installing/";

/// Where the checker lands in the environment.
pub const THE_CHECKER_ABOARD: &str = "/usr/bin/cosign";

/// The build arguments both recipes must give the same value.
pub const SHARED: [&str; 4] = ["THE_BASE", "THE_BUILDER", "THE_STABLE", "THE_TARGET"];

/// The build argument naming the checker's version.
const THE_CHECKER: &str = "ARG THE_CHECKER=";

/// The build argument naming the checker's digest.
const THE_CHECKERS_DIGEST: &str = "ARG THE_CHECKERS_SHA256=";

/// The stage the checker is fetched in.
const THE_CHECKERS_STAGE: &str = "checker";

/// What checking a digest looks like in a build step.
const A_DIGEST_CHECKED: &str = "sha256sum --check";

/// What the environment's recipe says, beside what the image's recipe says.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TheEnvironment {
    /// Each shared argument: its name, the environment's value, the image's.
    shared: Vec<(&'static str, Option<String>, Option<String>)>,
    /// Every `COPY` line's source and landing, outside any `--from`.
    copies: Vec<(String, String)>,
    /// The checker's version.
    checker: Option<String>,
    /// The checker's digest.
    checkers_digest: Option<String>,
    /// Whether the checker's stage checks the digest before anything else in
    /// that stage runs it.
    checked: bool,
    /// Whether the checker is copied into the environment out of that stage.
    checker_lands: bool,
}

impl TheEnvironment {
    /// What the environment's recipe says, read beside the image's.
    #[must_use]
    pub fn read(environment: &str, image: &str) -> Self {
        let shared = SHARED
            .iter()
            .map(|name| (*name, stated(environment, name), stated(image, name)))
            .collect();

        let mut copies = Vec::new();
        let mut checker = None;
        let mut checkers_digest = None;
        let mut checker_lands = false;
        for line in environment.lines().map(str::trim) {
            if line.starts_with('#') {
                continue;
            }
            if let Some(named) = argument(line, THE_CHECKER) {
                checker = Some(named.to_owned());
            }
            if let Some(named) = argument(line, THE_CHECKERS_DIGEST) {
                checkers_digest = Some(named.to_owned());
            }
            if copies_from_a_stage_to(line, THE_CHECKER_ABOARD)
                && line.contains(&format!("--from={THE_CHECKERS_STAGE}"))
            {
                checker_lands = true;
            }
            if let Some(rest) = line.strip_prefix("COPY ")
                && !rest.contains("--from=")
            {
                let words: Vec<&str> = rest.split_whitespace().collect();
                if let [source, landing] = words.as_slice() {
                    copies.push(((*source).to_owned(), (*landing).to_owned()));
                }
            }
        }

        let stage = in_stage(environment, THE_CHECKERS_STAGE);
        let checked = stage
            .iter()
            .any(|line| line.contains(A_DIGEST_CHECKED) && line.contains("THE_CHECKERS_SHA256"));

        Self {
            shared,
            copies,
            checker,
            checkers_digest,
            checked,
            checker_lands,
        }
    }

    /// Every shared argument the two recipes give different values, as its
    /// name, the environment's value and the image's.
    #[must_use]
    pub fn differs_from_the_image(&self) -> Vec<(&'static str, &str, &str)> {
        self.shared
            .iter()
            .filter(|(_, environment, image)| environment.is_none() || environment != image)
            .map(|(name, environment, image)| {
                (
                    *name,
                    environment.as_deref().unwrap_or("-"),
                    image.as_deref().unwrap_or("-"),
                )
            })
            .collect()
    }

    /// Whether the recipe copies this file of the repository to exactly there.
    #[must_use]
    pub fn copies(&self, source: &str, landing: &str) -> bool {
        self.copies
            .iter()
            .any(|(from, to)| from == source && to == landing)
    }

    /// The checker's version, as the recipe names it.
    #[must_use]
    pub fn checker(&self) -> Option<&str> {
        self.checker.as_deref()
    }

    /// Whether the checker is one exact release, a whole digest, checked in its
    /// own stage before it is copied, and copied into the environment.
    #[must_use]
    pub fn the_checker_is_pinned_and_checked(&self) -> bool {
        self.checker
            .as_deref()
            .is_some_and(crate::version::is_a_release)
            && self
                .checkers_digest
                .as_deref()
                .is_some_and(is_a_whole_digest)
            && self.checked
            && self.checker_lands
    }
}

/// The value a recipe gives an argument, where it gives one.
fn stated(recipe: &str, name: &str) -> Option<String> {
    let prefix = format!("ARG {name}=");
    recipe
        .lines()
        .map(str::trim)
        .find_map(|line| argument(line, &prefix))
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shipped recipes, as text.
    fn the_recipes() -> (String, String) {
        let at = std::path::Path::new(crate::THE_IMAGE);
        (
            std::fs::read_to_string(at.join(THE_ENVIRONMENT)).unwrap_or_default(),
            std::fs::read_to_string(at.join("Containerfile")).unwrap_or_default(),
        )
    }

    /// The shipped environment is on the image's base and toolchain, carries the
    /// pin and the key, and pins and checks its checker.
    #[test]
    fn the_shipped_environment_agrees_with_the_image() {
        let (environment, image) = the_recipes();
        let read = TheEnvironment::read(&environment, &image);
        assert_eq!(read.differs_from_the_image(), Vec::new());
        assert!(read.copies("image/pinned.toml", THE_PIN_ABOARD));
        assert!(read.copies(
            "image/signing/alo-os.pub",
            "/usr/lib/alo/installing/signing/alo-os.pub"
        ));
        assert!(read.the_checker_is_pinned_and_checked());
        assert_eq!(read.checker(), Some("3.1.3"));
    }

    /// **A different base, or a missing argument, is a difference.**
    #[test]
    fn another_base_is_caught() {
        let (environment, image) = the_recipes();
        let moved = environment.replace(
            "fedora-bootc:42@sha256:077182",
            "fedora-bootc:43@sha256:077182",
        );
        let read = TheEnvironment::read(&moved, &image);
        let differs = read.differs_from_the_image();
        assert_eq!(differs.len(), 1);
        assert_eq!(differs.first().map(|(name, _, _)| *name), Some("THE_BASE"));

        let gone = environment.replace("ARG THE_STABLE=", "ARG SOMETHING_ELSE=");
        let read = TheEnvironment::read(&gone, &image);
        assert_eq!(
            read.differs_from_the_image(),
            vec![("THE_STABLE", "-", "1.97.0")]
        );
    }

    /// **A checker that floats, is half-pinned, is never checked, or never
    /// lands is not pinned and checked.**
    #[test]
    fn a_checker_nobody_checked_is_caught() {
        let (environment, image) = the_recipes();
        for (from, to) in [
            ("ARG THE_CHECKER=3.1.3", "ARG THE_CHECKER=latest"),
            (
                "ARG THE_CHECKERS_SHA256=4629c757b7618056f8ddd7e2625ae9fdd94c0372a65049520bc7d9df9efc7f71",
                "ARG THE_CHECKERS_SHA256=4629c757b7618056",
            ),
            ("| sha256sum --check -", "| cat"),
            (
                "COPY --from=checker /cosign /usr/bin/cosign",
                "COPY --from=checker /cosign /usr/bin/something-else",
            ),
        ] {
            assert!(
                environment.contains(from),
                "the recipe no longer says {from}"
            );
            let read = TheEnvironment::read(&environment.replace(from, to), &image);
            assert!(!read.the_checker_is_pinned_and_checked(), "{to}");
        }
    }

    /// **A copy of another file, or to another place, is not the pin aboard.**
    #[test]
    fn a_different_pin_or_key_is_not_carried() {
        let (environment, image) = the_recipes();
        let other = environment.replace("COPY image/pinned.toml", "COPY elsewhere/pinned.toml");
        assert!(!TheEnvironment::read(&other, &image).copies("image/pinned.toml", THE_PIN_ABOARD));

        let other = environment.replace(
            "/usr/lib/alo/installing/signing/alo-os.pub",
            "/usr/lib/alo/installing/signing/other.pub",
        );
        assert!(!TheEnvironment::read(&other, &image).copies(
            "image/signing/alo-os.pub",
            "/usr/lib/alo/installing/signing/alo-os.pub"
        ));
    }
}
