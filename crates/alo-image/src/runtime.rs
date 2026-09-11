//! What the image's recipe says about the model runtime it carries.
//!
//! [ADR 0025](../../../docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md)
//! made *the local model is what the machine arrives ready to run* the promise,
//! and the expensive half of it is an artefact on the disk of every machine we
//! ship. [ADR 0006](../../../docs/decisions/0006-the-pinned-model-runtime.md)
//! settled how it gets there: a pinned upstream artefact, configured, never
//! patched. Both sentences are about `image/Containerfile`, which no other
//! declaration the image ships can see — a build that quietly dropped the
//! runtime, or floated its version, is green everywhere except here.
//!
//! # This is not a second file that knows the runtime's API
//!
//! ADR 0006's one-file rule is about how the runtime is *spoken to*: its
//! endpoint, its naming convention, its response fields, and — since ADR 0019 —
//! its address. All of that stays in `crates/alo-models/src/ollama.rs`, and
//! none of it appears here. Where the runtime's files land on the image is a
//! different kind of fact: it is the image's own, the Containerfile has to
//! spell it to put the artefact there at all, and this module is the image's
//! reader. A checker that could not name what the image carries could not
//! check that it carries it.
//!
//! # Read leniently, judged strictly
//!
//! [`TheRuntime::read`] never refuses: a Containerfile with no runtime in it
//! reads as one that lands nothing and pins nothing, and `crate::checking` is
//! what turns each absence into a [`Wrong`](crate::Wrong) with the decision it
//! breaks. That is `crate::accounts`' shape — the interesting states are the
//! wrong ones, and a reader that refused them could never report them.

/// Where the runtime's binary lands on the machine.
pub const THE_RUNTIMES_BINARY: &str = "/usr/bin/ollama";

/// Where the runtime's own libraries land on the machine, as the artefact lays
/// them out.
pub const THE_RUNTIMES_LIBRARIES: &str = "/usr/lib/ollama/";

/// The build argument that names the runtime's version.
const THE_VERSION_ARG: &str = "ARG THE_RUNTIME=";

/// The build argument that holds the artefact's digest.
const THE_DIGEST_ARG: &str = "ARG THE_RUNTIME_SHA256=";

/// The name of the digest argument alone, for finding the line that checks it.
const THE_DIGEST_NAME: &str = "THE_RUNTIME_SHA256";

/// What the checking of a digest looks like in a build step.
const A_DIGEST_CHECKED: &str = "sha256sum --check";

/// How many hexadecimal characters a sha256 digest has.
const A_WHOLE_DIGEST: usize = 64;

/// What the image's recipe says about the model runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheRuntime {
    /// The version the recipe pins, or [`None`] where it pins none.
    version: Option<String>,
    /// The digest the recipe pins, or [`None`] where it pins none.
    digest: Option<String>,
    /// Whether any build step actually checks that digest before unpacking.
    checked: bool,
    /// Whether the runtime's binary is copied onto the machine.
    binary_lands: bool,
    /// Whether the runtime's libraries are copied onto the machine.
    libraries_land: bool,
}

impl TheRuntime {
    /// What this Containerfile says, read off its text.
    ///
    /// Never a refusal: a recipe that says nothing about the runtime reads as
    /// one that lands nothing and pins nothing, which is `checking.rs`'s to
    /// report against the decisions it breaks.
    #[must_use]
    pub fn read(containerfile: &str) -> Self {
        let mut version = None;
        let mut digest = None;
        let mut checked = false;
        let mut binary_lands = false;
        let mut libraries_land = false;

        for line in containerfile.lines() {
            let line = line.trim();
            if line.starts_with('#') {
                continue;
            }
            if let Some(named) = line.strip_prefix(THE_VERSION_ARG) {
                version = Some(named.trim().to_owned());
            }
            if let Some(named) = line.strip_prefix(THE_DIGEST_ARG) {
                digest = Some(named.trim().to_owned());
            }
            if line.contains(THE_DIGEST_NAME) && line.contains(A_DIGEST_CHECKED) {
                checked = true;
            }
            if copies_from_a_stage_to(line, THE_RUNTIMES_BINARY) {
                binary_lands = true;
            }
            if copies_from_a_stage_to(line, THE_RUNTIMES_LIBRARIES) {
                libraries_land = true;
            }
        }

        Self {
            version,
            digest,
            checked,
            binary_lands,
            libraries_land,
        }
    }

    /// The version the recipe names, or [`None`] where it names none.
    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// The digest the recipe names, or [`None`] where it names none.
    #[must_use]
    pub fn digest(&self) -> Option<&str> {
        self.digest.as_deref()
    }

    /// Whether the runtime's binary is copied onto the machine.
    #[must_use]
    pub const fn lands_its_binary(&self) -> bool {
        self.binary_lands
    }

    /// Whether the runtime's libraries are copied onto the machine.
    #[must_use]
    pub const fn lands_its_libraries(&self) -> bool {
        self.libraries_land
    }

    /// Whether the version is one exact release — `major.minor.patch`, all
    /// numbers.
    ///
    /// `latest` is not a version, and neither is `0.34`: a floating or partial
    /// name is a different runtime on two builds of one image, moved by nobody
    /// and tested beside nothing, which is what ADR 0006's pin exists to
    /// refuse.
    #[must_use]
    pub fn is_pinned(&self) -> bool {
        self.version().is_some_and(|version| {
            let parts: Vec<&str> = version.split('.').collect();
            parts.len() == 3
                && parts
                    .iter()
                    .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
        })
    }

    /// Whether the artefact is held to a whole digest that a build step
    /// actually checks.
    ///
    /// A version names what was asked for; only a digest, checked before
    /// anything is unpacked, says what arrived. A digest nothing checks is
    /// prose.
    #[must_use]
    pub fn is_verified(&self) -> bool {
        self.checked
            && self.digest().is_some_and(|digest| {
                digest.len() == A_WHOLE_DIGEST && digest.bytes().all(|b| b.is_ascii_hexdigit())
            })
    }
}

/// Whether this line copies something out of another build stage to exactly
/// this place on the machine.
///
/// `--from=` is part of the question on purpose: an artefact copied out of the
/// build context would be a runtime committed into this repository, which is
/// the *source tree* shape ADR 0006 refuses — pinned upstream artefacts arrive
/// through a stage that fetched and checked them.
fn copies_from_a_stage_to(line: &str, landing: &str) -> bool {
    line.starts_with("COPY")
        && line.contains("--from=")
        && line.split_whitespace().next_back() == Some(landing)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A recipe holding exactly the lines a test names.
    fn saying(lines: &[&str]) -> TheRuntime {
        TheRuntime::read(&lines.join("\n"))
    }

    /// **The whole shape reads off a recipe that carries it**, so the refusals
    /// below are about what is missing rather than about a reader that never
    /// worked.
    #[test]
    fn a_recipe_that_carries_the_runtime_pinned_reads_as_one() {
        let read = saying(&[
            "ARG THE_RUNTIME=0.34.0",
            &format!("ARG THE_RUNTIME_SHA256={}", "ab".repeat(32)),
            "RUN echo \"${THE_RUNTIME_SHA256}  /it.tar.zst\" | sha256sum --check -",
            "COPY --from=runtime /runtime/bin/ollama /usr/bin/ollama",
            "COPY --from=runtime /runtime/lib/ollama/ /usr/lib/ollama/",
        ]);

        assert_eq!(read.version(), Some("0.34.0"));
        assert!(read.lands_its_binary());
        assert!(read.lands_its_libraries());
        assert!(read.is_pinned());
        assert!(read.is_verified());
    }

    /// **A recipe that says nothing reads as one that lands nothing**, rather
    /// than as a refusal — the wrong states are the ones the checker has to be
    /// able to see.
    #[test]
    fn a_recipe_with_no_runtime_in_it_lands_nothing_and_pins_nothing() {
        let read = saying(&["FROM somewhere", "COPY image/etc/alo/ /etc/alo/"]);

        assert_eq!(read.version(), None);
        assert_eq!(read.digest(), None);
        assert!(!read.lands_its_binary());
        assert!(!read.lands_its_libraries());
        assert!(!read.is_pinned());
        assert!(!read.is_verified());
    }

    /// **A floating or partial version is not a pin.** Each of these is a
    /// different runtime on two builds of one image.
    #[test]
    fn a_version_that_floats_is_not_pinned() {
        for floating in ["latest", "0.34", "v0.34.0", "0.34.0-rc1", "", "0..0"] {
            let read = saying(&[&format!("ARG THE_RUNTIME={floating}")]);
            assert!(!read.is_pinned(), "`{floating}` was read as pinned");
        }
    }

    /// **A digest nothing checks is prose**, and so is half a digest: both are
    /// a pin that stopped meaning what arrived.
    #[test]
    fn a_digest_that_is_not_checked_or_not_whole_does_not_verify_anything() {
        let unchecked = saying(&[&format!("ARG THE_RUNTIME_SHA256={}", "ab".repeat(32))]);
        assert!(!unchecked.is_verified());

        let half = saying(&[
            &format!("ARG THE_RUNTIME_SHA256={}", "ab".repeat(16)),
            "RUN echo \"${THE_RUNTIME_SHA256}  /it\" | sha256sum --check -",
        ]);
        assert!(!half.is_verified());

        let not_hex = saying(&[
            &format!("ARG THE_RUNTIME_SHA256={}", "zz".repeat(32)),
            "RUN echo \"${THE_RUNTIME_SHA256}  /it\" | sha256sum --check -",
        ]);
        assert!(!not_hex.is_verified());
    }

    /// **An artefact out of the build context is not the runtime landing.** A
    /// binary committed into this repository is the source-tree shape ADR 0006
    /// refuses, and a commented-out line lands nothing at all.
    #[test]
    fn a_copy_that_is_not_from_a_stage_does_not_count() {
        let from_the_context = saying(&["COPY vendored/ollama /usr/bin/ollama"]);
        assert!(!from_the_context.lands_its_binary());

        let commented = saying(&["# COPY --from=runtime /runtime/bin/ollama /usr/bin/ollama"]);
        assert!(!commented.lands_its_binary());
    }
}
