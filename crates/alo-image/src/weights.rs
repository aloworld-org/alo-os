//! What the image's recipe says about the weights a machine arrives with.
//!
//! [ADR 0025](../../../docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md)
//! was accepted as Option D, and the expensive half of what it took on is this:
//! **a model on the disk of every machine we ship, sized for that machine**
//! ([ADR 0007](../../../docs/decisions/0007-the-cpu-is-the-default.md)). A
//! runtime with nothing to load answers exactly as little as no runtime at all,
//! so until the recipe carries weights, *the local model is what the machine
//! arrives ready to run* is a sentence about a machine nobody has.
//!
//! [`crate::runtime`] is the same shape one artefact over, and the two are
//! separate files because they are two different promises that go wrong in two
//! different ways — the runtime by floating its version, the weights by naming a
//! model nobody measured.
//!
//! # Carried, not fetched — and what that cost
//!
//! ADR 0025 left one thing open: *whether the image carries the weights or
//! fetches them at setup*. It is carried. A machine that fetches at setup has
//! not arrived ready when it is offline at setup, and the promise is about what
//! is in the box rather than about the network somebody unpacks it beside. The
//! price is an image 2.23 GiB larger and an update channel that moves those
//! bytes whenever this pin moves, which is a cost paid by us; the other answer's
//! price is paid by a person on their first morning, in a place where we cannot
//! help them. `docs/quirks.md` carries the measurement.
//!
//! # Carried once — asserted, because the first build carried them twice
//!
//! Building the recipe measured what the runtime's import really does
//! (`docs/quirks.md`, 2026-09-11): it copies the checked file into its store
//! under that file's own digest, writes a second blob of the same length under
//! another, and names only the second in its manifest. A store left as the
//! runtime left it is 4.5 GiB for 2.23 GiB of model, on the read-only half of
//! every machine we ship, where nothing can ever prune it. So the weights stage
//! removes the source blob **after** the import — the digest check has done its
//! work by then and is untouched — and holds every blob left in the store to the
//! manifest before the store leaves the stage. [`TheWeights::drops_the_source`]
//! and [`TheWeights::holds_the_store_to_its_manifest`] read those two lines, so
//! a recipe that quietly went back to carrying the weights twice is a red test
//! rather than an image 2.23 GiB larger than its own comment.
//!
//! # It is not enough for the recipe to name *a* model
//!
//! `docs/features.md` promises a catalogue *measured by us, not claimed by the
//! publisher*, one line above the promise this file is about. So a recipe may
//! not name weights the catalogue has never heard of, or an entry nobody has put
//! to `alo-driving` — the check is `crate::checking`'s, against
//! [`alo_models::Catalogue`], and this file is only what the recipe says.
//!
//! # Read leniently, judged strictly
//!
//! [`TheWeights::read`] never refuses, for [`crate::runtime`]'s reason: the
//! interesting states are the wrong ones, and a reader that refused them could
//! never report them.

use crate::recipe::{argument, copies_from_a_stage_to, in_stage, is_a_whole_digest};

/// Where the weights land on the machine, as the runtime's own model store.
///
/// Ours rather than the runtime's default, because the default is a home
/// directory belonging to a login this image does not make. What points the
/// runtime at it is the unit that starts the runtime, and this image has none
/// yet — which is said in as many words beside the `COPY` line, so that nobody
/// reads a store on the disk as a model a machine is serving.
pub const THE_WEIGHTS: &str = "/usr/share/alo/models/";

/// The build stage that fetches the weights and imports them.
const THE_STAGE: &str = "weights";

/// The build argument that names the catalogue entry these weights are.
const THE_MODEL_ARG: &str = "ARG THE_MODEL=";

/// The build argument that names the quantisation.
const THE_QUANTISATION_ARG: &str = "ARG THE_MODELS_QUANTISATION=";

/// The build argument that names what the runtime on the machine answers to.
const THE_ARTEFACT_ARG: &str = "ARG THE_MODELS_ARTEFACT=";

/// The build argument that names where the weights are fetched from.
const THE_SOURCE_ARG: &str = "ARG THE_MODELS_WEIGHTS=";

/// The build argument that holds the weights' digest.
const THE_DIGEST_ARG: &str = "ARG THE_MODELS_SHA256=";

/// The name of the digest argument alone, for finding the line that checks it.
const THE_DIGEST_NAME: &str = "THE_MODELS_SHA256";

/// What the checking of a digest looks like in a build step.
const A_DIGEST_CHECKED: &str = "sha256sum --check";

/// How a fetch says where it is putting what it fetched.
const FETCHED_TO: &str = "--output";

/// How the weights are imported into the runtime's store, which is the line the
/// store exists after and the line the source blob may only be removed after.
const IMPORTED: &str = "ollama create";

/// The blob the runtime leaves the checked file under: the file's own digest,
/// in the store, named by no manifest once the import has written the blob it
/// names instead.
const THE_SOURCE_BLOB: &str = "blobs/sha256-${THE_MODELS_SHA256}";

/// How a line removes something.
const REMOVED: &str = "rm ";

/// How the stage walks the store, which is where holding every blob in it to
/// the manifest begins.
const EVERY_BLOB: &str = "for blob in /models/blobs/*";

/// How a blob is looked for in the manifest.
const LOOKED_UP: &str = "grep";

/// What it is looked for in.
const THE_MANIFEST: &str = "manifest";

/// What a blob the manifest does not name does to the build.
const REFUSED: &str = "exit 1";

/// Names that mean *whatever is there today*.
///
/// A digest says what arrived and this says what was asked for, and the two are
/// not the same question: a moving name with a pinned digest is a recipe that
/// stops building the morning the publisher pushes a commit, which is a broken
/// release rather than a caught mistake.
const MOVING: [&str; 4] = ["/main/", "/master/", "/HEAD/", ":latest"];

/// What the image's recipe says about the weights it carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheWeights {
    /// The catalogue entry the recipe names, or [`None`] where it names none.
    model: Option<String>,
    /// The quantisation the recipe names, or [`None`] where it names none.
    quantisation: Option<String>,
    /// What the runtime on the machine answers to, or [`None`].
    artefact: Option<String>,
    /// Where the weights are fetched from, or [`None`].
    from: Option<String>,
    /// The digest the recipe pins, or [`None`] where it pins none.
    digest: Option<String>,
    /// Whether that digest is checked before anything else reads the file.
    checked_first: bool,
    /// Whether the weights are copied onto the machine.
    lands: bool,
    /// Whether the blob the runtime left the checked file under is removed
    /// after the import.
    source_dropped: bool,
    /// Whether every blob left in the store is held to the manifest.
    held_to_manifest: bool,
}

impl TheWeights {
    /// What this Containerfile says, read off its text.
    ///
    /// Never a refusal: a recipe that says nothing about weights reads as one
    /// that carries none, which is `checking.rs`'s to report against the
    /// decisions it breaks.
    #[must_use]
    pub fn read(containerfile: &str) -> Self {
        let mut weights = Self {
            model: None,
            quantisation: None,
            artefact: None,
            from: None,
            digest: None,
            checked_first: false,
            lands: false,
            source_dropped: false,
            held_to_manifest: false,
        };

        for line in containerfile.lines() {
            let line = line.trim();
            if line.starts_with('#') {
                continue;
            }
            for (named, into) in [
                (THE_MODEL_ARG, &mut weights.model),
                (THE_QUANTISATION_ARG, &mut weights.quantisation),
                (THE_ARTEFACT_ARG, &mut weights.artefact),
                (THE_SOURCE_ARG, &mut weights.from),
                (THE_DIGEST_ARG, &mut weights.digest),
            ] {
                if let Some(said) = argument(line, named) {
                    *into = Some(said.to_owned());
                }
            }
            if copies_from_a_stage_to(line, THE_WEIGHTS) {
                weights.lands = true;
            }
        }

        weights.checked_first = checked_before_anything_read_it(containerfile);
        let (source_dropped, held_to_manifest) = carried_once(containerfile);
        weights.source_dropped = source_dropped;
        weights.held_to_manifest = held_to_manifest;
        weights
    }

    /// Whether the stage removes the blob the runtime left the checked file
    /// under, **after** the import.
    ///
    /// After, because before it there is nothing to import from; and the digest
    /// check is not this line's business — it ran on the file before the import
    /// read it, and removing the runtime's copy of that file afterwards changes
    /// nothing about what was checked.
    #[must_use]
    pub const fn drops_the_source(&self) -> bool {
        self.source_dropped
    }

    /// Whether every blob left in the store is held to the manifest that names
    /// it before the store leaves the stage.
    ///
    /// This is the line that makes *carried once* a build that goes red rather
    /// than a comment: a runtime update that left a second copy behind under
    /// some other name would be caught here, where [`Self::drops_the_source`]
    /// would not see it.
    #[must_use]
    pub const fn holds_the_store_to_its_manifest(&self) -> bool {
        self.held_to_manifest
    }

    /// The catalogue entry the recipe names, or [`None`] where it names none.
    #[must_use]
    pub fn model(&self) -> Option<&str> {
        said(&self.model)
    }

    /// The quantisation the recipe names, or [`None`] where it names none.
    #[must_use]
    pub fn quantisation(&self) -> Option<&str> {
        said(&self.quantisation)
    }

    /// What the runtime on the machine answers to, or [`None`] where the recipe
    /// says nothing.
    #[must_use]
    pub fn artefact(&self) -> Option<&str> {
        said(&self.artefact)
    }

    /// Where the weights are fetched from, or [`None`] where the recipe says
    /// nothing.
    #[must_use]
    pub fn from(&self) -> Option<&str> {
        said(&self.from)
    }

    /// The digest the recipe names, or [`None`] where it names none.
    #[must_use]
    pub fn digest(&self) -> Option<&str> {
        said(&self.digest)
    }

    /// Whether the weights are copied onto the machine at all.
    #[must_use]
    pub const fn land(&self) -> bool {
        self.lands
    }

    /// Whether what is fetched is one exact thing rather than whatever is
    /// published under a moving name today.
    ///
    /// The runtime's version pin, said about a file instead of a release: a
    /// branch is not a revision, and `latest` is not a version.
    #[must_use]
    pub fn is_pinned(&self) -> bool {
        self.from()
            .is_some_and(|from| !MOVING.iter().any(|moving| from.contains(moving)))
    }

    /// Whether the weights are held to a whole digest that the build checks
    /// **before anything reads them**.
    ///
    /// Stricter than [`crate::TheRuntime::is_verified`] on purpose, and the
    /// difference is the failure this one can have: weights are imported by the
    /// runtime rather than unpacked by `tar`, so a recipe that fetched, imported
    /// and then checked would produce a model store built out of whatever
    /// arrived and a build that went red afterwards — with the wrong bytes
    /// already in a layer.
    #[must_use]
    pub fn is_verified(&self) -> bool {
        self.checked_first && self.digest().is_some_and(is_a_whole_digest)
    }
}

/// Whatever a recipe said, where it said anything.
fn said(what: &Option<String>) -> Option<&str> {
    what.as_deref().map(str::trim).filter(|it| !it.is_empty())
}

/// Whether the stage that fetches the weights checks their digest before any
/// other line in it touches the file it fetched.
///
/// Order is the whole question, so it is asked inside one stage: which file was
/// fetched is what the fetch itself says, and every later line naming that file
/// is something reading it.
fn checked_before_anything_read_it(containerfile: &str) -> bool {
    let lines = in_stage(containerfile, THE_STAGE);

    let Some(fetched) = lines.iter().copied().find_map(fetched_to) else {
        return false;
    };

    let mut checked = false;
    for line in lines {
        if line.contains(THE_DIGEST_NAME) && line.contains(A_DIGEST_CHECKED) {
            checked = true;
            continue;
        }
        if fetched_to(line).is_some() {
            continue;
        }
        if line.contains(fetched) {
            return checked;
        }
    }
    checked
}

/// Whether the weights stage, after the import, removes the source blob and
/// holds every blob left in the store to the manifest — as two answers, because
/// they are two lines that go wrong separately.
///
/// Read off the text the way the rest of this crate reads a recipe: a line that
/// removes the source blob is one naming `rm` and the blob by the pinned
/// digest's own argument; the store is held when the stage walks every blob,
/// looks each up in the manifest, and refuses the build for one it does not
/// find. Order is part of the question — a removal before the import would
/// remove nothing and then import nothing — so it is asked inside the stage,
/// after the line that imports.
fn carried_once(containerfile: &str) -> (bool, bool) {
    let lines = in_stage(containerfile, THE_STAGE);
    let Some(imported) = lines.iter().position(|line| line.contains(IMPORTED)) else {
        return (false, false);
    };
    let after: Vec<&str> = lines.into_iter().skip(imported + 1).collect();

    let dropped = after
        .iter()
        .any(|line| line.contains(REMOVED) && line.contains(THE_SOURCE_BLOB));

    // Three lines in order: the walk, the lookup, the refusal. Each is looked
    // for after the one before it, by skipping rather than slicing.
    let walked = after.iter().position(|line| line.contains(EVERY_BLOB));
    let looked = walked.and_then(|walked| {
        after
            .iter()
            .skip(walked + 1)
            .position(|line| line.contains(LOOKED_UP) && line.contains(THE_MANIFEST))
            .map(|looked| walked + 1 + looked)
    });
    let held =
        looked.is_some_and(|looked| after.iter().skip(looked).any(|line| line.contains(REFUSED)));

    (dropped, held)
}

/// Where this line puts what it fetched, where it is a fetch.
fn fetched_to(line: &str) -> Option<&str> {
    let mut words = line.split_whitespace();
    while let Some(word) = words.next() {
        if word == FETCHED_TO {
            return words.next();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A recipe holding exactly the lines a test names.
    fn saying(lines: &[&str]) -> TheWeights {
        TheWeights::read(&lines.join("\n"))
    }

    /// The shape a correct recipe has, as a fixture the refusals below break one
    /// line of.
    fn a_correct_recipe() -> Vec<String> {
        vec![
            "ARG THE_MODEL=phi-3-mini-instruct".to_owned(),
            "ARG THE_MODELS_QUANTISATION=Q4_K_M".to_owned(),
            "ARG THE_MODELS_ARTEFACT=phi3:3.8b-mini-4k-instruct-q4_K_M".to_owned(),
            "ARG THE_MODELS_WEIGHTS=https://example.test/resolve/a64113/it.gguf".to_owned(),
            format!("ARG THE_MODELS_SHA256={}", "ab".repeat(32)),
            "FROM builder AS weights".to_owned(),
            "RUN curl --output /weights.gguf \"${THE_MODELS_WEIGHTS}\" \\".to_owned(),
            " && echo \"${THE_MODELS_SHA256}  /weights.gguf\" | sha256sum --check -".to_owned(),
            "RUN ollama create it -f /weights.gguf \\".to_owned(),
            " && rm -f /models/blobs/sha256-${THE_MODELS_SHA256} \\".to_owned(),
            " && for blob in /models/blobs/*; do \\".to_owned(),
            "      grep -q \"sha256:${blob##*/sha256-}\" \"${manifest}\" || exit 1; \\".to_owned(),
            "    done".to_owned(),
            "FROM base".to_owned(),
            "COPY --from=weights /models/ /usr/share/alo/models/".to_owned(),
        ]
    }

    /// The same recipe with one line changed, the way a person edits a
    /// Containerfile.
    fn with(from: &str, to: &str) -> TheWeights {
        let lines: Vec<String> = a_correct_recipe()
            .into_iter()
            .map(|line| line.replace(from, to))
            .collect();
        TheWeights::read(&lines.join("\n"))
    }

    /// **The whole shape reads off a recipe that carries it**, so the refusals
    /// below are about what is missing rather than about a reader that never
    /// worked.
    #[test]
    fn a_recipe_that_carries_the_weights_pinned_reads_as_one() {
        let read = TheWeights::read(&a_correct_recipe().join("\n"));

        assert_eq!(read.model(), Some("phi-3-mini-instruct"));
        assert_eq!(read.quantisation(), Some("Q4_K_M"));
        assert_eq!(read.artefact(), Some("phi3:3.8b-mini-4k-instruct-q4_K_M"));
        assert!(read.land());
        assert!(read.is_pinned());
        assert!(read.is_verified());
        assert!(read.drops_the_source());
        assert!(read.holds_the_store_to_its_manifest());
    }

    /// **A stage that leaves the runtime's copy of the checked file in the
    /// store reads as carrying the weights twice**, which is exactly what the
    /// first build of the real recipe did — and the store is still held to its
    /// manifest, because the two lines are two answers.
    #[test]
    fn a_stage_that_leaves_the_source_blob_reads_as_carrying_the_weights_twice() {
        let left = with(
            " && rm -f /models/blobs/sha256-${THE_MODELS_SHA256} \\",
            " && true \\",
        );

        assert!(!left.drops_the_source());
        assert!(left.holds_the_store_to_its_manifest());
    }

    /// **A store nothing holds to its manifest reads as held to nothing**, and
    /// dropping the source blob does not stand in for it: a runtime update
    /// leaving a second copy under some other name is caught by the walk, never
    /// by the removal.
    #[test]
    fn a_store_nothing_holds_to_its_manifest_reads_as_held_to_nothing() {
        let unwalked = with("for blob in /models/blobs/*", "for blob in /nowhere/*");
        assert!(unwalked.drops_the_source());
        assert!(!unwalked.holds_the_store_to_its_manifest());

        let unrefused = with("|| exit 1", "|| true");
        assert!(!unrefused.holds_the_store_to_its_manifest());

        let unlooked = with("grep -q", "test -f");
        assert!(!unlooked.holds_the_store_to_its_manifest());
    }

    /// **A removal before the import removes nothing**, because there is
    /// nothing in the store yet — and a reader that swept the stage for an `rm`
    /// would have counted it.
    #[test]
    fn a_removal_before_the_import_does_not_count() {
        let early = saying(&[
            &format!("ARG THE_MODELS_SHA256={}", "ab".repeat(32)),
            "FROM builder AS weights",
            "RUN curl --output /weights.gguf https://example.test/resolve/a64113/it.gguf",
            "RUN rm -f /models/blobs/sha256-${THE_MODELS_SHA256}",
            "RUN ollama create it -f /weights.gguf",
        ]);
        assert!(!early.drops_the_source());

        let never_imported = saying(&[
            "FROM builder AS weights",
            "RUN rm -f /models/blobs/sha256-${THE_MODELS_SHA256}",
        ]);
        assert!(!never_imported.drops_the_source());
        assert!(!never_imported.holds_the_store_to_its_manifest());
    }

    /// **A recipe that says nothing reads as one that carries nothing**, rather
    /// than as a refusal — the wrong states are the ones the checker has to be
    /// able to see.
    #[test]
    fn a_recipe_with_no_weights_in_it_carries_nothing_and_pins_nothing() {
        let read = saying(&["FROM somewhere", "COPY image/etc/alo/ /etc/alo/"]);

        assert_eq!(read.model(), None);
        assert_eq!(read.quantisation(), None);
        assert_eq!(read.artefact(), None);
        assert_eq!(read.digest(), None);
        assert!(!read.land());
        assert!(!read.is_pinned());
        assert!(!read.is_verified());
    }

    /// **A name that moves is not a pin.** A branch is a different file on two
    /// builds of one image, and the digest beside it turns that into a broken
    /// build rather than a caught mistake.
    #[test]
    fn weights_fetched_from_a_moving_name_are_not_pinned() {
        for moving in [
            "https://example.test/resolve/main/it.gguf",
            "https://example.test/resolve/master/it.gguf",
            "https://example.test/resolve/HEAD/it.gguf",
            "registry.test/it:latest",
        ] {
            let read = with("https://example.test/resolve/a64113/it.gguf", moving);
            assert!(!read.is_pinned(), "`{moving}` was read as pinned");
        }
        assert!(!saying(&["FROM base"]).is_pinned());
    }

    /// **A digest nothing checks is prose**, and so is half a digest.
    #[test]
    fn a_digest_that_is_not_checked_or_not_whole_verifies_nothing() {
        let half = with(&"ab".repeat(32), &"ab".repeat(16));
        assert!(!half.is_verified());

        let not_hex = with(&"ab".repeat(32), &"zz".repeat(32));
        assert!(!not_hex.is_verified());

        let unchecked = with("sha256sum --check -", "true");
        assert!(!unchecked.is_verified());
    }

    /// **And a digest checked after the weights have already been read verifies
    /// nothing either**, which is the mistake that looks like the correct
    /// recipe with two lines swapped: the model store is built out of whatever
    /// arrived, and the build goes red with those bytes already in a layer.
    #[test]
    fn a_digest_checked_after_the_weights_were_read_is_not_a_check() {
        let afterwards = saying(&[
            &format!("ARG THE_MODELS_SHA256={}", "ab".repeat(32)),
            "FROM builder AS weights",
            "RUN curl --output /weights.gguf https://example.test/resolve/a64113/it.gguf",
            "RUN ollama create it -f /weights.gguf",
            "RUN echo \"${THE_MODELS_SHA256}  /weights.gguf\" | sha256sum --check -",
        ]);
        assert!(!afterwards.is_verified());
    }

    /// **A check in some other stage is not this stage's check.** The runtime's
    /// own digest is checked three stages away, and a reader that swept the
    /// whole recipe would have counted it.
    #[test]
    fn a_check_in_another_stage_does_not_verify_these_weights() {
        let elsewhere = saying(&[
            &format!("ARG THE_MODELS_SHA256={}", "ab".repeat(32)),
            "FROM builder AS runtime",
            "RUN echo \"${THE_MODELS_SHA256}  /it\" | sha256sum --check -",
            "FROM builder AS weights",
            "RUN curl --output /weights.gguf https://example.test/resolve/a64113/it.gguf",
            "RUN ollama create it -f /weights.gguf",
        ]);
        assert!(!elsewhere.is_verified());
    }

    /// **Weights copied out of the build context are not weights landing.** A
    /// multi-gigabyte artefact committed into this repository is the source-tree
    /// shape ADR 0006 refuses, and a commented-out line lands nothing at all.
    #[test]
    fn a_copy_that_is_not_from_a_stage_does_not_count() {
        assert!(!saying(&["COPY vendored/models/ /usr/share/alo/models/"]).land());
        assert!(!saying(&["# COPY --from=weights /models/ /usr/share/alo/models/"]).land());
        assert!(!saying(&["COPY --from=weights /models/ /usr/share/alo/elsewhere/"]).land());
    }

    /// An argument that is there and says nothing is not a statement — the rule
    /// `alo_models::Catalogue` holds a curator to, one file over.
    #[test]
    fn an_argument_that_says_nothing_names_nothing() {
        let blank = with("phi-3-mini-instruct", "   ");
        assert_eq!(blank.model(), None);
    }
}
